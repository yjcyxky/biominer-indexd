use rbatis::{
  self, crud_table, executor::RbatisExecutor, html_sql, push_index, py_sql, rb_html, rb_py,
  rbatis::Rbatis, Page, PageRequest,
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
  // pub hash_type: String, // Max 128 characters, md5, sha1, sha256, sha512, blake2b, etc.
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
  pub urls: Vec<URL>,
  pub hashes: Vec<Hash>,
  pub aliases: Vec<Alias>,
}

#[html_sql("sql/query_files.xml")]
async fn query_files(
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

async fn query_file(rb: &mut RbatisExecutor<'_, '_>, guid: &str, hash: &str) -> Option<File> {
  let files = query_files(
    rb,
    &PageRequest::new(1, 1),
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

async fn init_rbatis(database_url: &str) -> Rbatis {
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
      "",
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
