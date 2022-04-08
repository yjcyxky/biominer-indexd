#[macro_use]
extern crate rbatis;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use biominer_indexd::{api, database::init_rbatis};
use dotenv::dotenv;
use log::{error, LevelFilter};
use log4rs;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use poem::middleware::AddData;
use poem::EndpointExt;
use poem::{listener::TcpListener, middleware::Cors, Route, Server};
use poem_openapi::OpenApiService;
use std::error::Error;
use std::sync::Arc;
use tokio::{self, time::Duration};

use structopt::StructOpt;

fn init_logger(tag_name: &str) -> Result<log4rs::Handle, String> {
  let stdout = ConsoleAppender::builder()
    .encoder(Box::new(PatternEncoder::new(
      &(format!("[{}]", tag_name) + " {d} - {h({l} - {t} - {m}{n})}"),
    )))
    .build();

  let config = Config::builder()
    .appender(Appender::builder().build("stdout", Box::new(stdout)))
    .logger(
      Logger::builder()
        .appender("stdout")
        .additive(false)
        .build("stdout", LevelFilter::Info),
    )
    .build(Root::builder().appender("stdout").build(LevelFilter::Info))
    .unwrap();

  log4rs::init_config(config).map_err(|e| {
    format!(
      "couldn't initialize log configuration. Reason: {}",
      e.description()
    )
  })
}

/// An Index Engine for Omics Data Files.
#[derive(Debug, PartialEq, StructOpt)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="Biominer Indexd", author="Jingcheng Yang <yjcyxky@163.com>")]
struct Opt {
  /// Activate debug mode
  /// short and long flags (-D, --debug) will be deduced from the field's name
  #[structopt(name = "debug", short = "D", long = "debug")]
  debug: bool,

  /// 127.0.0.1 or 0.0.0.0
  #[structopt(name = "host", short = "H", long = "host", possible_values=&["127.0.0.1", "0.0.0.0"], default_value = "127.0.0.1")]
  host: String,

  /// Which port.
  #[structopt(name = "port", short = "p", long = "port", default_value = "3000")]
  port: String,

  /// Database url, such as postgres:://user:pass@host:port/dbname.
  /// You can also set it with env var: DATABASE_URL.
  #[structopt(name = "database-url", short = "d", long = "database-url")]
  database_url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
  dotenv().ok();

  if let Err(log) = init_logger("biominer-indexd") {
    error!(target:"stdout", "Log initialization error, {}", log);
    std::process::exit(1);
  };

  let args = Opt::from_args();

  let host = args.host;
  let port = args.port;
  let database_url = args.database_url;

  let database_url = if database_url.is_none() {
    match std::env::var("DATABASE_URL") {
      Ok(v) => v,
      Err(_) => {
        error!("{}", "DATABASE_URL is not set.");
        std::process::exit(1);
      }
    }
  } else {
    database_url.unwrap()
  };

  let rb = init_rbatis(&database_url).await;
  let arc_rb = Arc::new(rb);
  let shared_rb = AddData::new(arc_rb.clone());

  println!("\n\t\t*** Launch biominer-indexd on {}:{} ***", host, port);

  let api_service = OpenApiService::new(api::files::FilesApi, "Files", "1.0.0")
    .server(format!("http://{}:{}", host, port));
  let ui = api_service.swagger_ui();
  let spec = api_service.spec();
  let route = Route::new()
    .nest("/", api_service)
    .nest("/ui", ui)
    .at("/spec", poem::endpoint::make_sync(move |_| spec.clone()))
    .with(Cors::new())
    .with(shared_rb);

  Server::new(TcpListener::bind(format!("{}:{}", host, port)))
    .run(route)
    .await
  // Server::new(TcpListener::bind(format!("{}:{}", host, port)))
  //   .run_with_graceful_shutdown(
  //     route,
  //     async move {
  //       let _ = tokio::signal::ctrl_c().await;
  //     },
  //     Some(Duration::from_secs(5)),
  //   )
  //   .await
}
