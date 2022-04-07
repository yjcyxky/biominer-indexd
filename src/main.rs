#[macro_use]
extern crate rbatis;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use biominer_indexd::database;
use database::{init_log, init_rbatis, query_file, query_files};
use dotenv::dotenv;
use rbatis::rbatis::Rbatis;
use rbatis::PageRequest;

#[tokio::main]
async fn main() {
  dotenv().ok();
  init_log();
  let database_url = match std::env::var("DATABASE_URL") {
    Ok(v) => v,
    Err(msg) => "".to_string(),
  };
  let rb = init_rbatis(&database_url).await;
  let file = query_file(&mut rb.as_executor(), "abcd", "").await.unwrap();
  assert!(file.guid == "abcd");

  let files = query_files(
    &mut rb.as_executor(),
    &PageRequest::new(1, 10),
    "abcd",
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

  println!("{:?}", files);
}
