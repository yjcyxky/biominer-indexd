#[macro_use]
extern crate rbatis;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use biominer_indexd::database;
use dotenv::dotenv;
use rbatis::rbatis::Rbatis;
use rbatis::PageRequest;

lazy_static! {
  static ref RB: Rbatis = Rbatis::new();
}

#[tokio::main]
async fn main() {
  dotenv().ok();
  let database_url = match std::env::var("DATABASE_URL") {
    Ok(v) => v,
    Err(msg) => "".to_string(),
  };
}
