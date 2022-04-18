use super::util;
use chrono::{self, Utc};
use log::{debug, info, warn};
use poem_openapi::Object;
use rbatis::executor::ExecutorMut;
use rbatis::{
  self, crud_table, executor::RbatisExecutor, html_sql, push_index, rb_html, rbatis::Rbatis, Error,
  Page, PageRequest,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct FileStat {
  pub total_size: u64,
  pub num_of_files: u64,
  pub num_of_baseid: u64,
  // SELECT REPLACE(CONCAT_WS('', 'v', DATE(TO_TIMESTAMP(updated_at / 1000))), '-', '') FROM biominer_indexd_file ORDER BY updated_at DESC LIMIT 1;
  pub version: String,
  pub registry_id: String,
}

impl FileStat {
  pub async fn get_stat(rb: &Rbatis) -> Result<FileStat, Error> {
    let mut executor = rb.as_executor();
    let stat = executor
      .fetch(
        "
        SELECT 
          SUM(size)::BIGINT AS total_size, 
          COUNT (guid)::BIGINT AS num_of_files, 
          COUNT(DISTINCT(baseid))::BIGINT AS num_of_baseid,
          (SELECT REPLACE(CONCAT_WS('', 'v', DATE(TO_TIMESTAMP(updated_at / 1000))), '-', '') 
            FROM biominer_indexd_file ORDER BY updated_at DESC LIMIT 1) AS version,
          (SELECT registry_id FROM biominer_indexd_config LIMIT 1) AS registry_id
        FROM biominer_indexd_file",
        vec![],
      )
      .await;
    stat
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct FileTags {
  pub field_names: Vec<String>,
}

impl FileTags {
  pub async fn get_fields(rb: &Rbatis) -> Result<FileTags, Error> {
    let mut executor = rb.as_executor();
    let field_names = executor
      .fetch(
        "SELECT DISTINCT(field_name) AS field_names FROM biominer_indexd_tag",
        vec![],
      )
      .await;
    field_names
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Object)]
pub struct FilePage {
  /// data
  pub records: Vec<File>,
  /// total num
  pub total: u64,
  /// pages
  pub pages: u64,
  /// current page index
  pub page_no: u64,
  /// default 10
  pub page_size: u64,
  /// is search_count
  pub search_count: bool,
}

impl From<rbatis::Page<File>> for FilePage {
  fn from(page: rbatis::Page<File>) -> Self {
    let serialised = serde_json::to_string(&page).unwrap();
    serde_json::from_str(&serialised).unwrap()
  }
}

#[crud_table(table_name:biominer_indexd_url)]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Object)]
pub struct URL {
  #[oai(read_only)]
  pub id: u64,
  #[oai(validator(max_length = 255))]
  pub url: String,
  pub created_at: i64,
  #[oai(validator(max_length = 16))]
  pub status: String, // 'pending', 'processing', 'validated', 'failed'
  #[oai(validator(max_length = 64))]
  pub uploader: String,
  #[oai(validator(max_length = 64))]
  pub file: Option<String>,
}

#[crud_table(table_name:biominer_indexd_hash)]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Object)]
pub struct Hash {
  #[oai(read_only)]
  pub id: u64,
  #[oai(validator(max_length = 16))]
  pub hash_type: String, // Max 16 characters, md5, sha1, sha256, sha512, crc32, crc64, etag, etc
  #[oai(validator(max_length = 128))]
  pub hash: String, // Max 128 characters
  #[oai(validator(max_length = 64))]
  pub file: Option<String>,
}

#[crud_table(table_name:biominer_indexd_tag)]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Object)]
pub struct Tag {
  #[oai(read_only)]
  pub id: u64,
  #[oai(validator(max_length = 128))]
  pub field_name: String, // Max 128 characters
  #[oai(validator(max_length = 128))]
  pub field_value: String, // Max 128 characters
  #[oai(validator(max_length = 64))]
  pub file: Option<String>,
}

#[crud_table(table_name:biominer_indexd_alias)]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Object)]
pub struct Alias {
  #[oai(read_only)]
  pub id: u64,
  #[oai(validator(max_length = 255))]
  pub name: String,
  #[oai(validator(max_length = 64))]
  pub file: Option<String>,
}

#[crud_table(table_name:biominer_indexd_config)]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Object)]
pub struct Config {
  #[oai(read_only)]
  pub id: u64,
  #[oai(validator(max_length = 16, pattern = "^[0-9a-z-]{16}$"))]
  pub registry_id: String,
}

impl Config {
  pub fn new(registry_id: String) -> Self {
    Config {
      id: 0,
      registry_id: registry_id,
    }
  }

  fn get_registry_id() -> String {
    let registry_id = match std::env::var("BIOMIER_REGISTRY_ID") {
      Ok(v) => v,
      Err(_) => "fudan-pgx".to_string(),
    };

    let registry_id_regex = Regex::new(r"^[0-9a-z-]{16}$").unwrap();
    if !registry_id_regex.is_match(&registry_id) {
      warn!("Environment variable `BIOMIER_REGISTRY_ID` is not valid (Regex: ^[0-9a-z-]{{16}}$), use default value: fudan-pgx");
    }

    return registry_id;
  }

  pub async fn init_config(rb: &Rbatis) -> Config {
    let mut executor = rb.as_executor();
    let registry_id = Config::get_registry_id();
    let configs: serde_json::Value = executor
      .fetch("SELECT * FROM biominer_indexd_config", vec![])
      .await
      .unwrap();

    debug!("Config: {:?}", configs);

    // Configs always be an array, maybe have one or zero record.
    let configs = configs.as_array().unwrap();
    if configs.len() > 0 {
      warn!(
        "Config already exists, if you want to change the registry_id, please rebuild the database first."
      );
      let config: Config = serde_json::from_value(configs[0].clone()).unwrap();
      config
    } else {
      let v = executor
        .exec(
          "INSERT INTO biominer_indexd_config (registry_id) VALUES ($1);",
          vec![rbson::to_bson(&registry_id).unwrap()],
        )
        .await
        .unwrap();
      if v.rows_affected == 1 {
        info!("Set registry_id to {}", registry_id);
      }
      Config::new(registry_id)
    }
  }
}

#[crud_table(table_name:biominer_indexd_file)]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Object)]
pub struct File {
  pub guid: String,
  pub filename: String,
  pub size: u64,
  pub created_at: i64,
  pub updated_at: i64,
  pub status: String,
  pub baseid: String,
  pub rev: String,
  pub version: usize,
  pub uploader: String,
  pub urls: Option<Vec<URL>>,
  pub hashes: Option<Vec<Hash>>,
  pub aliases: Option<Vec<Alias>>,
  pub tags: Option<Vec<Tag>>,
}

impl File {
  pub fn new(
    filename: &str,
    size: u64,
    uploader: &str,
    registry_id: &str,
    urls: Option<Vec<URL>>,
    hashes: Option<Vec<Hash>>,
    aliases: Option<Vec<Alias>>,
    tags: Option<Vec<Tag>>,
  ) -> Self {
    let guid = uuid::Uuid::new_v4().to_string();
    let rev = guid[..8].to_string();
    let baseid = uuid::Uuid::new_v4().to_string();
    let now_ms = Utc::now().timestamp_millis();

    File {
      // biominer.fudan-pgx/3ec4d151-061b-4bcb-ad3a-425c712bfc88
      guid: format!("{}.{}/{}", "biominer", registry_id, guid),
      filename: filename.to_string(),
      size: size,
      created_at: now_ms,
      updated_at: now_ms,
      status: "pending".to_string(),
      baseid: baseid,
      uploader: uploader.to_string(),
      rev: rev,
      version: 1,
      urls: urls,
      hashes: hashes,
      aliases: aliases,
      tags: tags,
    }
  }

  pub async fn check_hash_exists(rb: &Rbatis, hash: &str) -> bool {
    let mut executor = rb.as_executor();
    let v: serde_json::Value = executor
      .fetch(
        "SELECT count(*) as count FROM biominer_indexd_hash WHERE hash = $1",
        vec![rbson::to_bson(hash).unwrap()],
      )
      .await
      .unwrap();
    // TODO: How to deal with it when has error?
    // fetch function will return a array, but it always has one element.
    v[0].get("count").unwrap().as_i64().unwrap() > 0
  }

  pub async fn add_url(
    rb: &Rbatis,
    uuid: &uuid::Uuid,
    url: &str,
    uploader: &str,
    status: &str,
  ) -> rbatis::core::Result<()> {
    let mut executor = rb.as_executor();
    let uuid_str = uuid.to_string();

    let v: serde_json::Value = executor
      .fetch(
        "SELECT * FROM biominer_indexd_url WHERE file = $1",
        vec![rbson::to_bson(&uuid_str).unwrap()],
      )
      .await
      .unwrap();

    let status = if ["pending", "processing", "validated", "failed"].contains(&status) {
      status.to_string()
    } else {
      "pending".to_string()
    };

    if v.as_array().unwrap().len() == 1 {
      let v = executor
        .exec(
          "INSERT INTO biominer_indexd_url (file, url, status, uploader) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING;",
          vec![
            rbson::to_bson(&uuid_str).unwrap(),
            rbson::to_bson(url).unwrap(),
            rbson::to_bson(&status).unwrap(),
            rbson::to_bson(uploader).unwrap(),
          ],
        )
        .await
        .unwrap();

      if v.rows_affected == 1 {
        info!("Add url {} to file {}", url, uuid);
        return Ok(());
      } else {
        info!("Url {} already exists in file {}.", url, uuid);
        return Ok(());
      }
    } else {
      warn!("Cannot find the file {}.", uuid);
      return Err(Error::from(format!("Cannot find the file with {}", uuid)));
    }
  }

  pub async fn add_alias(rb: &Rbatis, uuid: &uuid::Uuid, alias: &str) -> rbatis::core::Result<()> {
    let mut executor = rb.as_executor();
    let uuid_str = uuid.to_string();

    let v: serde_json::Value = executor
      .fetch(
        "SELECT * FROM biominer_indexd_alias WHERE file = $1",
        vec![rbson::to_bson(&uuid_str).unwrap()],
      )
      .await
      .unwrap();

    if v.as_array().unwrap().len() == 1 {
      let v = executor
        .exec(
          "INSERT INTO biominer_indexd_alias (file, name) VALUES ($1, $2) ON CONFLICT DO NOTHING;",
          vec![
            rbson::to_bson(&uuid_str).unwrap(),
            rbson::to_bson(alias).unwrap(),
          ],
        )
        .await
        .unwrap();

      if v.rows_affected == 1 {
        info!("Add alias {} to file {}", alias, uuid);
        return Ok(());
      } else {
        info!("Alias {} already exists in file {}.", alias, uuid);
        return Ok(());
      }
    } else {
      warn!("Cannot find the file {}.", uuid);
      return Err(Error::from(format!("Cannot find the file with {}", uuid)));
    }
  }

  pub async fn add_tag(
    rb: &Rbatis,
    uuid: &uuid::Uuid,
    field_name: &str,
    field_value: &str,
  ) -> rbatis::core::Result<()> {
    let mut executor = rb.as_executor();
    let uuid_str = uuid.to_string();

    let v: serde_json::Value = executor
      .fetch(
        "SELECT * FROM biominer_indexd_alias WHERE file = $1",
        vec![rbson::to_bson(&uuid_str).unwrap()],
      )
      .await
      .unwrap();

    if v.as_array().unwrap().len() == 1 {
      let v = executor
        .exec(
          "INSERT INTO biominer_indexd_tag (file, field_name, field_value) VALUES ($1, $2, $3) ON CONFLICT (file, field_name, field_value) DO NOTHING;",
          vec![
            rbson::to_bson(&uuid_str).unwrap(),
            rbson::to_bson(field_name).unwrap(),
            rbson::to_bson(field_value).unwrap(),
          ],
        )
        .await
        .unwrap();

      if v.rows_affected == 1 {
        info!(
          "Add tag \"{}:{}\" to file {}",
          field_name, field_value, uuid
        );
        return Ok(());
      } else {
        info!("Tag {} already exists in file {}.", field_name, uuid);
        return Ok(());
      }
    } else {
      warn!("Cannot find the file {}.", uuid);
      return Err(Error::from(format!("Cannot find the file with {}", uuid)));
    }
  }

  pub async fn add_hash(rb: &Rbatis, uuid: &uuid::Uuid, hash: &str) -> rbatis::core::Result<()> {
    let mut executor = rb.as_executor();
    let uuid_str = uuid.to_string();

    let v: serde_json::Value = executor
      .fetch(
        "SELECT * FROM biominer_indexd_alias WHERE file = $1",
        vec![rbson::to_bson(&uuid_str).unwrap()],
      )
      .await
      .unwrap();

    let hash_type = match util::which_hash_type(hash) {
      Some(hash_type) => hash_type,
      None => {
        warn!("Cannot find the hash type of {}", hash);
        return Err(Error::from(format!("Cannot find the hash type of {}, only support md5, sha1, sha256, sha512, crc32, etag, crc64.", hash)));
      }
    };

    if v.as_array().unwrap().len() == 1 {
      let v = executor
        .exec(
          "INSERT INTO biominer_indexd_hash (file, hash_type, hash) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING;",
          vec![
            rbson::to_bson(&uuid_str).unwrap(),
            rbson::to_bson(hash_type).unwrap(),
            rbson::to_bson(hash).unwrap(),
          ],
        )
        .await
        .unwrap();

      if v.rows_affected == 1 {
        info!("Add hash {} to file {}", hash, uuid);
        return Ok(());
      } else {
        info!("Alias {} already exists in file {}.", hash, uuid);
        return Ok(());
      }
    } else {
      warn!("Cannot find the file {}.", uuid);
      return Err(Error::from(format!("Cannot find the file with {}", uuid)));
    }
  }

  pub async fn add(&mut self, rb: &Rbatis, hash: &str) -> rbatis::core::Result<()> {
    // tx will be commit.when func end
    let mut tx = rb.acquire_begin().await?.defer_async(|mut tx1| async move {
      if !tx1.is_done() {
        tx1.rollback().await;
        warn!("Commit rollback success!");
      } else {
        info!("Don't need to rollback!")
      }
    });

    let fvalue = vec![
      rbson::to_bson(&self.guid).unwrap(),
      rbson::to_bson(&self.filename).unwrap(),
      rbson::to_bson(&self.size).unwrap(),
      rbson::to_bson(&self.created_at).unwrap(),
      rbson::to_bson(&self.updated_at).unwrap(),
      rbson::to_bson(&self.status).unwrap(),
      rbson::to_bson(&self.baseid).unwrap(),
      rbson::to_bson(&self.uploader).unwrap(),
      rbson::to_bson(&self.rev).unwrap(),
      rbson::to_bson(&self.version).unwrap(),
    ];

    let f = match tx
      .exec(
        "INSERT INTO biominer_indexd_file 
             (guid, filename, size, created_at, updated_at, status, baseid, uploader, rev, version)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             ON CONFLICT DO NOTHING;",
        fvalue,
      )
      .await
    {
      Ok(f) => f,
      Err(e) => {
        warn!("Insert File Error: {:?}", e);
        return Err(Error::from(e.to_string()));
      }
    };

    let hvalue = vec![
      rbson::to_bson(hash).unwrap(),
      rbson::to_bson("md5").unwrap(),
      rbson::to_bson(&self.guid).unwrap(),
    ];

    let h = match tx
      .exec(
        "INSERT INTO biominer_indexd_hash (hash, hash_type, file) VALUES ($1, $2, $3);",
        hvalue,
      )
      .await
    {
      Ok(h) => h,
      Err(e) => {
        warn!("Insert Hash Error: {:?}", e);
        return Err(Error::from(e.to_string()));
      }
    };

    match tx.commit().await {
      Ok(_) => {
        info!("Commit success!");
      }
      Err(e) => {
        warn!("Commit error: {:?}", e);
        return Err(Error::from(e.to_string()));
      }
    };
    return Ok(());
  }
}

// #[py_sql(
//   "
//   SELECT
//     guid, filename, size, updated_at, baseid, rev, version,
//     biominer_indexd_file.created_at          as created_at,
//     biominer_indexd_file.status              as status,
//     biominer_indexd_file.uploader            as uploader,
//     json_agg(DISTINCT biominer_indexd_url)   as urls,
//     json_agg(DISTINCT biominer_indexd_hash)  as hashes,
//     json_agg(DISTINCT biominer_indexd_alias) as aliases
//   FROM
//     biominer_indexd_file
//   JOIN biominer_indexd_url ON biominer_indexd_url.file = biominer_indexd_file.guid
//   JOIN biominer_indexd_hash ON biominer_indexd_hash.file = biominer_indexd_file.guid
//   JOIN biominer_indexd_alias ON biominer_indexd_alias.file = biominer_indexd_file.guid
//   where:
//     if filename != '':
//       filename LIKE CONCAT('%', #{filename}, '%')
//     if guid != '':
//       trim 'AND':
//         AND guid = #{guid}
//     if baseid != '':
//       trim 'AND':
//         AND baseid = #{baseid}
//     if status != '':
//       trim 'AND':
//         AND biominer_indexd_file.status = #{status}
//     if uploader != '':
//       trim 'AND':
//         AND uploader = #{uploader}
//     if hash != '':
//       trim 'AND':
//         AND biominer_indexd_hash.hash = #{hash}
//     if alias != '':
//       trim 'AND':
//         AND biominer_indexd_alias.name = #{alias}
//     if url != '':
//       trim 'AND':
//         AND biominer_indexd_url.url = #{url}
//   ${' '}
//   GROUP BY guid
// "
// )]
// pub async fn query_files_pysql(
//   rb: &mut RbatisExecutor<'_, '_>,
//   page_req: &PageRequest,
//   guid: &str,
//   filename: &str,
//   baseid: &str,
//   status: &str,
//   uploader: &str,
//   hash: &str,
//   alias: &str,
//   url: &str,
// ) -> Page<File> {
//   todo!()
// }

#[html_sql("sql/query_files.xml")]
pub async fn query_files(
  rb: &mut RbatisExecutor<'_, '_>,
  page_req: &PageRequest,
  guid: &str,
  filename: &str,
  baseid: &str,
  status: &str,
  uploader: &str,
  hash: &str,
  alias: &str,
  url: &str,
  field_name: &str,
  field_value: &str,
  contain_alias: &usize,
  contain_url: &usize,
  contain_tag: &usize,
) -> Page<File> {
  todo!()
}

pub async fn query_file(
  rb: &mut RbatisExecutor<'_, '_>,
  guid: &str,
  hash: &str,
  contain_alias: &usize,
  contain_url: &usize,
  contain_tag: &usize,
) -> Option<File> {
  let files = query_files(
    rb,
    &PageRequest::new(1, 10),
    guid,
    "",
    "",
    "",
    "",
    hash,
    "",
    "",
    "",
    "",
    contain_alias,
    contain_url,
    contain_tag,
  )
  .await
  .unwrap();

  if files.total == 1 {
    Some(files.records.get(0).unwrap().clone())
  } else {
    None
  }
}

pub async fn init_rbatis(database_url: &str) -> Rbatis {
  let rb = Rbatis::new();
  rb.link(&database_url).await.unwrap();
  rb
}

#[cfg(test)]
mod tests {
  use super::{init_rbatis, query_file, query_files};
  use rbatis::{rbatis::Rbatis, PageRequest};

  async fn init() -> Rbatis {
    let database_url = match std::env::var("DATABASE_URL") {
      Ok(v) => v,
      Err(msg) => "postgres://postgres:password@localhost:5432/test_biominer_indexd".to_string(),
    };

    return init_rbatis(&database_url).await;
  }

  #[tokio::test]
  async fn test_query_files() {
    let rb = init().await;
    let files = query_files(
      &mut rb.as_executor(),
      &PageRequest::new(1, 10),
      "",
      "",
      "",
      "failed",
      "",
      "",
      "",
      "",
      "",
      "",
      &1,
      &1,
      &1,
    )
    .await
    .unwrap();

    assert!(files.total > 0);
  }

  #[tokio::test]
  async fn test_query_file() {
    let rb = init().await;
    let file = query_file(&mut rb.as_executor(), "abcd", "", &1, &1, &1)
      .await
      .unwrap();

    assert!(file.guid == "abcd");
  }
}
