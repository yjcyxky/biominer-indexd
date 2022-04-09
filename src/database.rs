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
  pub id: u64,
  pub url: String,
  pub created_at: i64,
  pub status: String, // 'pending', 'processing', 'validated', 'failed'
  pub uploader: String,
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
}

#[crud_table(table_name:biominer_indexd_alias)]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Object)]
pub struct Alias {
  #[oai(read_only)]
  pub id: u64,
  #[oai(validator(max_length = 255))]
  pub name: String,
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
      warn!("Config already exists, if you want to change the registry_id, 
             please rebuild the database first.");
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
}

impl File {
  pub fn new(filename: &str, size: u64, uploader: &str, registry_id: &str) -> Self {
    let guid = uuid::Uuid::new_v4().to_string();
    let rev = guid[..8].to_string();
    let baseid = uuid::Uuid::new_v4().to_string();
    let now_ms = Utc::now().timestamp_millis();

    File {
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
      urls: None,
      hashes: None,
      aliases: None,
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
  contain_alias: &usize,
  contain_url: &usize,
) -> Page<File> {
  todo!()
}

pub async fn query_file(
  rb: &mut RbatisExecutor<'_, '_>,
  guid: &str,
  hash: &str,
  contain_alias: &usize,
  contain_url: &usize,
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
    contain_alias,
    contain_url,
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
    let file = query_file(&mut rb.as_executor(), "abcd", "", &1, &1)
      .await
      .unwrap();

    assert!(file.guid == "abcd");
  }
}
