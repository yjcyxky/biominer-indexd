extern crate log;

use log::*;
use std::collections::HashMap;
use structopt::StructOpt;
use biominer_indexd::init_logger;
use biominer_indexd::run_migrations;

/// NOTE: In the first time, you need to follow the order to run the commands: initdb -> importdb.
///
#[derive(StructOpt, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name = "A cli for Biominer Indexd service.", author="Jingcheng Yang <yjcyxky@163.com>;")]
struct Opt {
    /// Short and long flags (--verbose or -v) will increase the log level.
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,

    #[structopt(subcommand)]
    cmd: SubCommands,
}

#[derive(Debug, PartialEq, StructOpt)]
enum SubCommands {
    #[structopt(name = "initdb")]
    InitDB(InitDbArguments),
    #[structopt(name = "importdb")]
    ImportDB(ImportDBArguments),
    #[structopt(name = "cleandb")]
    CleanDB(CleanDBArguments),
}

/// Initialize the database, only for the postgres database. We might need to run the initdb command when we want to upgrade the database schema or the first time we run the application.
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="BioMinerIndexd - initdb", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct InitDbArguments {
    /// Database url, such as postgres://postgres:postgres@localhost:5432/biominer-indexd, if not set, use the value of environment variable DATABASE_URL.
    #[structopt(name = "database_url", short = "d", long = "database-url")]
    database_url: Option<String>,
}

/// Clean the database, if you want to clean any table in the database, you can use this command.
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="BiominerIndexd - cleandb", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct CleanDBArguments {
    /// Database url, such as postgres://postgres:postgres@localhost:5432/biominer-indexd. if not set, use the value of environment variable DATABASE_URL or NEO4J_URL.
    #[structopt(name = "database_url", short = "d", long = "database-url")]
    database_url: Option<String>,

    /// [Required] The table name to clean. e.g We will empty all entity-related tables if you use the entity table name. such as entity, entity_metadata, entity2d.
    #[structopt(name = "table", short = "t", long = "table", possible_values = &["entity", "relation", "embedding", "subgraph", "curation", "score", "message", "metadata"], multiple = true)]
    table: Vec<String>,
}

/// Import data files into database
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="BiominerIndexd - importdb", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct ImportDBArguments {
    /// [Required] Database url, such as postgres://postgres:postgres@localhost:5432/biominer-indexd, if not set, use the value of environment variable DATABASE_URL.
    #[structopt(name = "database_url", short = "d", long = "database-url")]
    database_url: Option<String>,

    /// [Required] The file path of the data file to import. It may be a file or a directory. If you have multiple files to import, you can use the --filepath option with a directory path. We will import all files in the directory. But you need to disable the --drop option, otherwise, only the last file will be imported successfully.
    #[structopt(name = "filepath", short = "f", long = "filepath")]
    filepath: Option<String>,

    /// [Optional] Drop the table before import data. If you have multiple files to import, don't use this option. If you use this option, only the last file will be imported successfully.
    #[structopt(name = "drop", long = "drop")]
    drop: bool,

    /// [Optional] Don't check other related tables in the database. Such as knowledge_curation which might be related to entity.
    #[structopt(name = "skip_check", short = "s", long = "skip-check")]
    skip_check: bool,

    /// [Optional] Show the first 3 errors when import data.
    #[structopt(name = "show_all_errors", short = "e", long = "show-all-errors")]
    show_all_errors: bool,

    /// [Optional] The batch size for syncing data to the graph database.
    #[structopt(
        name = "batch_size",
        short = "b",
        long = "batch-size",
        default_value = "10000"
    )]
    batch_size: usize,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    info!("Setting up logger with {} level.", opt.verbose);
    let log_result = match opt.verbose {
        0 => init_logger("biominer-indexd", LevelFilter::Warn),
        1 => init_logger("biominer-indexd", LevelFilter::Info),
        2 => init_logger("biominer-indexd", LevelFilter::Debug),
        _ => init_logger("biominer-indexd", LevelFilter::Debug),
    };

    let _logger_handle = match log_result {
        Ok(handle) => handle,
        Err(log) => {
            error!(target:"stdout", "Log initialization error, {}", log);
            std::process::exit(1);
        }
    };

    match opt.cmd {
        SubCommands::InitDB(arguments) => {
            let database_url = arguments.database_url;

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

            match run_migrations(&database_url).await {
                Ok(_) => info!("Init database successfully."),
                Err(e) => error!("Init database failed: {}", e),
            }
        }
        SubCommands::ImportDB(arguments) => {
            let database_url = if arguments.database_url.is_none() {
                match std::env::var("DATABASE_URL") {
                    Ok(v) => v,
                    Err(_) => {
                        error!("{}", "DATABASE_URL is not set.");
                        std::process::exit(1);
                    }
                }
            } else {
                arguments.database_url.unwrap()
            };

            // TODO: Implement the importdb command.
        }
        SubCommands::CleanDB(arguments) => {
            let database_url = if arguments.database_url.is_none() {
                match std::env::var("DATABASE_URL") {
                    Ok(v) => v,
                    Err(_) => {
                        error!("{}", "DATABASE_URL is not set.");
                        std::process::exit(1);
                    }
                }
            } else {
                arguments.database_url.unwrap()
            };

            let pool = match sqlx::PgPool::connect(&database_url).await {
                Ok(v) => v,
                Err(e) => {
                    error!("Connect to database failed: {}", e);
                    std::process::exit(1);
                }
            };

            let mut table_names_map = HashMap::<&str, Vec<&str>>::new();
            let pairs = vec![
                ("file", vec!["biominer_indexd_url", "biominer_indexd_hash", "biominer_indexd_alias", "biominer_indexd_tag", "biominer_indexd_file"]),
                ("tag", vec!["biominer_indexd_tag"]),
                ("hash", vec!["biominer_indexd_hash"]),
                ("alias", vec!["biominer_indexd_alias"]),
                ("url", vec!["biominer_indexd_url"]),
            ];

            for pair in pairs {
                table_names_map.insert(pair.0, pair.1);
            }


            let tables = arguments.table;
            for table in tables {
                let table_names = table_names_map.get(table.as_str());
                if table_names.is_none() {
                    error!("The table name is not supported.");
                    std::process::exit(1);
                }

                let table_names = table_names.unwrap();
                for table_name in table_names {
                    let sql = format!("TRUNCATE TABLE {}", table_name);
                    match sqlx::query(&sql).execute(&pool).await {
                        Ok(_) => info!("Clean the {} table successfully.", table_name),
                        Err(e) => error!("Clean the {} table failed: {}", table_name, e),
                    }
                }
            }
            
        }
    }
}
