use rbatis::{
  self, crud_table, executor::RbatisExecutor, html_sql, log::LogPlugin, log::RbatisLogPlugin,
  push_index, py_sql, rb_html, rb_py, rbatis::Rbatis, sql_index, Page, PageRequest,
};
use serde::{Deserialize, Serialize};

#[crud_table(table_name:biominer_indexd_url)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct URL {
  pub id: u64,
  pub url: String,
  pub created_at: rbatis::DateTimeNative,
  pub status: String, // 'pending', 'processing', 'validated', 'failed'
  pub uploader: String,
}

#[crud_table(table_name:biominer_indexd_hash)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hash {
  pub id: u64,
  pub hash_type: String, // Max 128 characters, md5, sha1, sha256, sha512, blake2b, etc.
  pub hash: String,
}

#[crud_table(table_name:biominer_indexd_alias)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Alias {
  pub id: u64,
  pub name: String,
}

#[crud_table(table_name:biominer_indexd_file)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct File {
  pub guid: String,
  pub filename: String,
  pub size: u64,
  pub created_at: rbatis::DateTimeNative,
  pub updated_at: rbatis::DateTimeNative,
  pub status: String,
  pub baseid: String,
  pub uploader: Option<String>,
  pub urls: Option<Vec<URL>>,
  pub hashes: Option<Vec<Hash>>,
  pub aliases: Option<Vec<Alias>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct QueryParamsFile {
    pub guid: Option<String>,
    pub filename: Option<String>,
    pub baseid: Option<String>,
    pub status: Option<String>,
    pub uploader: Option<String>,
    pub hash: Option<String>,
    pub alias: Option<String>,
    pub url: Option<String>,
    pub page_size: Option<u64>,
    pub page: Option<u64>,
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
) -> Page<File> {
  todo!()
}

pub async fn query_file(rb: &mut RbatisExecutor<'_, '_>, guid: &str, hash: &str) -> Option<File> {
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
    )
    .await
    .unwrap();

    assert!(files.total > 0);
  }

  #[tokio::test]
  async fn test_query_file() {
    let rb = init().await;
    let file = query_file(&mut rb.as_executor(), "abcd", "").await.unwrap();

    assert!(file.guid == "abcd");
  }
}
