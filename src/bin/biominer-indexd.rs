#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use biominer_indexd::{api, connect_db, init_logger, model, repo_config::RepoConfig};
use dotenv::dotenv;
use log::{error, LevelFilter};
use poem::middleware::AddData;
use poem::EndpointExt;
use poem::{
    async_trait,
    endpoint::EmbeddedFilesEndpoint,
    http::{header, Method, StatusCode},
    listener::TcpListener,
    middleware::Cors,
    Endpoint, Request, Response, Result, Route, Server,
};
use poem_openapi::OpenApiService;
use rust_embed::RustEmbed;
use std::env;
use std::path::Path as OsPath;
use std::sync::Arc;
// use tokio::{self, time::Duration};

use structopt::StructOpt;

/// An Index Engine for Omics Data Files.
#[derive(Debug, PartialEq, StructOpt)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="BioMiner Indexd", author="Jingcheng Yang <yjcyxky@163.com>")]
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

    /// Which base path.
    #[structopt(
        name = "base_path",
        short = "b",
        long = "base-path",
        default_value = "/"
    )]
    base_path: String,

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

    /// Pool size for database connection.
    #[structopt(name = "pool-size", short = "s", long = "pool-size")]
    pool_size: Option<u32>,

    /// The path of the data directory.
    #[structopt(name = "data-dir", short = "D", long = "data-dir", required = true)]
    data_dir: String,
}

#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct Assets;

pub(crate) struct IndexHtmlEmbed;

#[async_trait]
impl Endpoint for IndexHtmlEmbed {
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        if req.method() != Method::GET {
            return Ok(StatusCode::METHOD_NOT_ALLOWED.into());
        }

        match Assets::get("index.html") {
            Some(content) => {
                let base_path_var = env::var("BASE_PATH").unwrap_or_else(|_| {
                    warn!("BASE_PATH: No such variable, use /.");
                    "/".to_string()
                });
                let base_path = OsPath::new(&base_path_var[..]);
                let body: Vec<u8> = content.data.into();
                let mut formated_str = String::from("");
                match std::str::from_utf8(&body) {
                    Ok(v) => {
                        return {
                            formated_str = v
                                .replace("/umi", base_path.join("assets/umi").to_str().unwrap())
                                .replace(
                                    "/logo.png",
                                    base_path.join("assets/logo.png").to_str().unwrap(),
                                )
                                .replace(
                                    "/favicon.ico",
                                    base_path.join("assets/favicon.ico").to_str().unwrap(),
                                )
                                .replace(
                                    "window.resourceBaseUrl || \"/\"",
                                    &format!("\"{}\"", base_path.join("assets/").to_str().unwrap())
                                        [..],
                                );
                            Ok(Response::builder()
                                .header(header::CONTENT_TYPE, "text/html")
                                .body(formated_str))
                        }
                    }
                    Err(e) => {
                        return {
                            Ok(Response::builder()
                                .header(header::CONTENT_TYPE, "text/html")
                                .body(body))
                        }
                    }
                };
            }
            None => Ok(Response::builder().status(StatusCode::NOT_FOUND).finish()),
        }
    }
}

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
    let base_path = args.base_path;
    // TODO: base_path may be not a valid path
    // It will generate double / when base_path is /.
    if base_path != "/" {
        match base_path.strip_prefix("/") {
            Some(v) => {
                env::set_var("BASE_PATH", format!("/{}", v));
            }
            None => {
                env::set_var("BASE_PATH", format!("/{}", &base_path));
            }
        }
    }
    let base_path = OsPath::new("/").join(&base_path[..]);

    env::set_var("BIOMINER_INDEXD_DATA_DIR", args.data_dir);

    println!(
        "\n\t\t*** Launch biominer-indexd on {}:{}{} ***",
        host,
        port,
        base_path.to_str().unwrap()
    );

    // Connect to the database
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
    let pool_size = args.pool_size.unwrap_or(10);
    let pool = connect_db(&database_url, pool_size).await;
    let arc_pool = Arc::new(pool);
    let shared_rb = AddData::new(arc_pool.clone());

    // Read the repo config file
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

    let config = model::datafile::Config::init_config(&arc_pool.clone()).await;
    info!("Initialize Config with `{:?}`", config);
    let shared_config = AddData::new(Arc::new(config));

    let api_service = OpenApiService::new(api::route::BioMinerIndexdApi, "BioMiner Indexd", "v0.1.0")
                                                                  .summary("A RESTful API for BioMiner Indexd")
                                                                  .description("BioMiner Indexd is a hash-based data indexing and tracking service providing globally unique identifiers.")
                                                                  .license("GNU AFFERO GENERAL PUBLIC LICENSE v3")
                                                                  .server(format!("http://{}:{}", host, port));
    let ui = api_service.swagger_ui();
    let spec = api_service.spec();
    let route = Route::new().nest(base_path.to_str().unwrap(), api_service);

    let route = if args.ui {
        info!("UI mode is enabled.");
        // TODO: How to redirect to /index.html when user access /?
        route
            .at(
                base_path.join("index.html").to_str().unwrap(),
                IndexHtmlEmbed,
            )
            .nest(
                base_path.join("assets").to_str().unwrap(),
                EmbeddedFilesEndpoint::<Assets>::new(),
            )
    } else {
        warn!("UI mode is disabled. If you need the UI, please use `--ui` flag.");
        route
    };

    let route = if args.openapi {
        info!("OpenApi mode is enabled.");
        route.nest(base_path.join("ui").to_str().unwrap(), ui).at(
            base_path.join("spec").to_str().unwrap(),
            poem::endpoint::make_sync(move |_| spec.clone()),
        )
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
