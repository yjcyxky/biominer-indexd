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
use postgresql_embedded::{PostgreSQL, Settings, Status, VersionReq};
use regex::Regex;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use std::{env, fs::File, io::Write, path::Path, path::PathBuf, time::Duration};
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

pub fn get_free_port() -> Option<u16> {
    match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => {
            let port = listener.local_addr().ok()?.port();
            info!("Found free port: {}", port);
            Some(port)
        }
        Err(e) => {
            eprintln!("Failed to get free port: {}", e);
            None
        }
    }
}

pub async fn setup_local_postgres(port: u16) -> Result<PostgreSQL, anyhow::Error> {
    let mut settings = Settings::default();
    settings.version = VersionReq::parse("17.5.0").unwrap();
    // TODO: Use a directory which is specified by the user for postgres data
    settings.installation_dir = PathBuf::from(
        env::home_dir()
            .unwrap()
            .join(".biominer-indexd")
            .join("postgres"),
    );
    if !settings.installation_dir.exists() {
        std::fs::create_dir_all(&settings.installation_dir).unwrap();
    }
    // Use a random port for local postgres
    settings.port = port;
    settings.username = "postgres".to_string();
    settings.password = "password".to_string();
    settings.data_dir = settings.installation_dir.join("data");
    if !settings.data_dir.exists() {
        std::fs::create_dir_all(&settings.data_dir).unwrap();
    }
    settings.temporary = false;
    settings.timeout = Some(Duration::from_secs(30));

    info!("Running local postgres with settings: {:?}", settings);
    let mut postgres = PostgreSQL::new(settings);
    let database = "biominer_indexd";

    info!("Checking if database {} exists", database);
    if postgres.status() != Status::Started {
        postgres.setup().await.expect("Failed to setup PostgreSQL");
        postgres.start().await.expect("Failed to start PostgreSQL");

        if !postgres.database_exists(&database).await.unwrap() {
            postgres
                .create_database(&database)
                .await
                .expect("Failed to create database");
        } else {
            info!("Database {} already exists", database);
        }
    }

    Ok(postgres)
}

pub fn get_local_postgres_url(postgres: &PostgreSQL, database_name: &str) -> String {
    let url = postgres
        .settings()
        .url(database_name)
        .replace("postgresql://", "postgres://");
    url
}
