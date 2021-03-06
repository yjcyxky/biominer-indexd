#[macro_use]
extern crate rbatis;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use biominer_indexd::{api, database, database::init_rbatis, repo_config::RepoConfig};
use dotenv::dotenv;
use log::{error, LevelFilter};
use log4rs;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use poem::middleware::AddData;
use poem::EndpointExt;
use poem::{
  endpoint::{EmbeddedFileEndpoint, EmbeddedFilesEndpoint},
  listener::TcpListener,
  middleware::Cors,
  Route, Server,
};
use poem_openapi::OpenApiService;
use rust_embed::RustEmbed;
use std::error::Error;
use std::sync::Arc;
use tokio::{self, time::Duration};

use structopt::StructOpt;

fn init_logger(tag_name: &str, level: LevelFilter) -> Result<log4rs::Handle, String> {
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
        .build("stdout", level),
    )
    .build(Root::builder().appender("stdout").build(level))
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

  /// Activate ui mode
  #[structopt(name = "ui", short = "u", long = "ui")]
  ui: bool,

  /// Activate openapi mode
  #[structopt(name = "openapi", short = "o", long = "openapi")]
  openapi: bool,

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

  /// The path of the repo config file.
  #[structopt(
    name = "config",
    short = "c",
    long = "config",
    default_value = "/etc/indexd.json"
  )]
  config: String,
}

#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct Assets;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
  dotenv().ok();

  let args = Opt::from_args();

  let log_result = if args.debug {
    init_logger("biominer-indexd", LevelFilter::Trace)
  } else {
    init_logger("biominer-indexd", LevelFilter::Info)
  };

  if let Err(log) = log_result {
    error!(target:"stdout", "Log initialization error, {}", log);
    std::process::exit(1);
  };

  let host = args.host;
  let port = args.port;

  println!("\n\t\t*** Launch biominer-indexd on {}:{} ***", host, port);

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

  let config_path = args.config;
  let indexd_repo_config = match RepoConfig::read_config(&config_path) {
    Ok(v) => v,
    Err(e) => {
      error!("{}: {}", e, config_path);
      std::process::exit(1);
    }
  };
  let arc_config = Arc::new(indexd_repo_config);
  let shared_repo_config = AddData::new(arc_config.clone());

  let rb = init_rbatis(&database_url).await;
  let arc_rb = Arc::new(rb);
  let shared_rb = AddData::new(arc_rb.clone());

  let config = database::Config::init_config(&arc_rb.clone()).await;
  info!("Initialize Config with `{:?}`", config);
  let shared_config = AddData::new(Arc::new(config));

  let api_service = OpenApiService::new(api::files::FilesApi, "BioMiner Indexd", "v0.1.0")
    .summary("A RESTful API for BioMiner Indexd")
    .description("BioMiner Indexd is a hash-based data indexing and tracking service providing globally unique identifiers.")
    .license("GNU AFFERO GENERAL PUBLIC LICENSE v3")
    .server(format!("http://{}:{}", host, port));
  let ui = api_service.swagger_ui();
  let spec = api_service.spec();
  let route = Route::new().nest_no_strip("/api", api_service);

  let route = if args.ui {
    info!("UI mode is enabled.");
    route
      .at("/", EmbeddedFileEndpoint::<Assets>::new("index.html"))
      .nest("/assets", EmbeddedFilesEndpoint::<Assets>::new())
  } else {
    warn!("UI mode is disabled. If you need the UI, please use `--ui` flag.");
    route
  };

  let route = if args.openapi {
    info!("OpenApi mode is enabled.");
    route
      .nest("/ui", ui)
      .at("/spec", poem::endpoint::make_sync(move |_| spec.clone()))
  } else {
    warn!("OpenApi mode is disabled. If you need the OpenApi, please use `--openapi` flag.");
    route
  };

  let route = route
    .with(Cors::new())
    .with(shared_rb)
    .with(shared_config)
    .with(shared_repo_config);

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
