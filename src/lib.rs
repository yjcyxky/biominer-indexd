// You must change the DB_VERSION to match the version of the database the library is compatible with.
const DB_VERSION: &str = "2.8.3";

pub mod api;
pub mod model;
pub mod query_builder;
pub mod repo_config;
pub mod util;

use log::{debug, error, info, warn, LevelFilter};
use log4rs;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use regex::Regex;
use sqlx::postgres::PgPoolOptions;
use sqlx::migrate::Migrator;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

const MIGRATIONS: include_dir::Dir = include_dir::include_dir!("migrations");

/// Connect to the database and run the migrations.
pub async fn run_migrations(database_url: &str) -> sqlx::Result<()> {
    info!("Running migrations.");
    // Create a temporary directory.
    let dir = tempdir()?;

    for file in MIGRATIONS.files() {
        // Create each file in the temporary directory.
        let file_path = dir.path().join(file.path());
        let mut temp_file = File::create(&file_path)?;
        // Write the contents of the included file to the temporary file.
        temp_file.write_all(file.contents())?;
    }

    // Now we can create a Migrator from the temporary directory.
    info!("Importing migrations from {:?}", dir.path());
    // List all files in the temporary directory.
    for file in dir.path().read_dir()? {
        match file {
            Ok(file) => info!("Found file: {:?}", file.path()),
            Err(e) => warn!("Error: {:?}", e),
        }
    }
    let migrator = Migrator::new(Path::new(dir.path())).await?;

    let pool = connect_db(database_url, 1).await;

    migrator.run(&pool).await?;

    // Don't forget to cleanup the temporary directory.
    dir.close()?;
    info!("Migrations finished.");

    Ok(())
}

pub fn init_logger(tag_name: &str, level: LevelFilter) -> Result<log4rs::Handle, String> {
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
                .build("biominer-indexd", level),
        )
        .build(Root::builder().appender("stdout").build(level))
        .map_err(|e| format!("Couldn't build logger config: {}", e.to_string()))?;

    // 初始化日志系统
    log4rs::init_config(config).map_err(|e| {
        format!(
            "Couldn't initialize log configuration. Reason: {}",
            e.to_string()
        )
    })
}

pub fn is_db_url_valid(db_url: &str) -> bool {
    // check whether db url is valid. the db_url format is <postgres|neo4j>://<username>:<password>@<host>:<port>/database
    let regex_str = r"^(postgres|neo4j)://((.+):(.+)@)?(.+):(\d+)(/.+)?$";
    let is_valid = match Regex::new(regex_str) {
        Ok(r) => r.is_match(db_url),
        Err(_) => false,
    };

    return is_valid;
}

pub fn parse_db_url(db_url: &str) -> (String, String, String, String, String) {
    // Get host, username and password from db_url. the db_url format is postgres://<username>:<password>@<host>:<port>/database
    let url = url::Url::parse(db_url).unwrap();
    let host = match url.host_str() {
        Some(h) => h.to_string(),
        None => "".to_string(),
    };
    let port = match url.port() {
        Some(p) => p.to_string(),
        None => "".to_string(),
    };
    let username = url.username().to_string();
    let password = match url.password() {
        Some(p) => p.to_string(),
        None => "".to_string(),
    };
    let database = url.path().to_string().replace("/", "");

    return (host, port, username, password, database);
}

pub async fn connect_db(database_url: &str, max_connections: u32) -> sqlx::PgPool {
    match is_db_url_valid(database_url) {
        true => (),
        false => {
            error!("Invalid database_url: {}, the format is postgres://<username>:<password>@<host>:<port>/<database>", database_url);
            std::process::exit(1);
        }
    };

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .idle_timeout(std::time::Duration::from_secs(600)) // 10 min
        .acquire_timeout(std::time::Duration::from_secs(30)) // 30 seconds
        .max_lifetime(std::time::Duration::from_secs(1800)) // 30 min
        .connect(&database_url)
        .await;

    match pool {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to connect to the database: {}", e);
            std::process::exit(1);
        }
    }
}
