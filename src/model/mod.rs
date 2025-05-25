use super::query_builder::sql_builder::ComposeQuery;
use super::util;
use super::util::{get_delimiter, parse_csv_error, ValidationError};
use anyhow::Ok as AnyOk;
use chrono::{self, Utc};
use csv;
use log::{debug, info, warn};
use poem_openapi::Object;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx;
use sqlx::Row;
use std::{error::Error, option::Option, path::PathBuf};
use uuid;
use validator::Validate;

pub trait CheckData {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>>;

    // Implement the check function
    fn check_csv_is_valid_default<
        S: for<'de> serde::Deserialize<'de> + Validate + std::fmt::Debug,
    >(
        filepath: &PathBuf,
    ) -> Vec<Box<dyn Error>> {
        info!("Start to check the csv file: {:?}", filepath);
        let mut validation_errors: Vec<Box<dyn Error>> = vec![];
        let delimiter = match get_delimiter(filepath) {
            Ok(d) => d,
            Err(e) => {
                validation_errors.push(Box::new(ValidationError::new(
                    &format!("Failed to get delimiter: ({})", e),
                    vec![],
                )));
                return validation_errors;
            }
        };

        debug!("The delimiter is: {:?}", delimiter as char);
        // Build the CSV reader
        let mut reader = match csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .from_path(filepath)
        {
            Ok(r) => r,
            Err(e) => {
                validation_errors.push(Box::new(ValidationError::new(
                    &format!("Failed to read CSV: ({})", e),
                    vec![],
                )));
                return validation_errors;
            }
        };

        // Try to deserialize each record
        debug!(
            "Start to deserialize the csv file, real columns: {:?}, expected columns: {:?}",
            reader.headers().unwrap().into_iter().collect::<Vec<_>>(),
            Self::fields()
        );
        let mut line_number = 1;
        for result in reader.deserialize::<S>() {
            line_number += 1;

            match result {
                Ok(data) => match data.validate() {
                    Ok(_) => {
                        continue;
                    }
                    Err(e) => {
                        validation_errors.push(Box::new(ValidationError::new(
                            &format!(
                                "Failed to validate the data, line: {}, details: ({})",
                                line_number, e
                            ),
                            vec![],
                        )));
                        continue;
                    }
                },
                Err(e) => {
                    let error_msg = parse_csv_error(&e);

                    validation_errors.push(Box::new(ValidationError::new(&error_msg, vec![])));

                    continue;
                }
            };
        }

        validation_errors
    }

    fn fields() -> Vec<String>;

    fn unique_fields() -> Vec<String>;

    fn get_error_msg<S: for<'de> serde::Deserialize<'de> + Validate + std::fmt::Debug>(
        r: Result<Vec<S>, Box<dyn Error>>,
    ) -> String {
        match r {
            Ok(_) => "".to_string(),
            Err(e) => {
                return e.to_string();
            }
        }
    }

    /// Select the columns to keep
    /// Return the path of the output file which is a temporary file
    fn select_expected_columns<S: for<'de> serde::Deserialize<'de> + Validate + std::fmt::Debug>(
        in_filepath: &PathBuf,
        out_filepath: &PathBuf,
    ) -> Result<Vec<S>, Box<dyn Error>> {
        let delimiter = get_delimiter(in_filepath)?;
        debug!("The delimiter is: {:?}", delimiter as char);
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .from_path(in_filepath)?;

        let headers = reader.headers()?.clone();
        debug!("The headers are: {:?}", headers);

        // Identify the indices of the columns to keep
        let indices_to_keep: Vec<usize> = headers
            .iter()
            .enumerate()
            .filter_map(|(i, h)| {
                if Self::fields().contains(&h.to_string()) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        debug!(
            "The indices of the columns to keep are: {:?}",
            indices_to_keep
        );
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(delimiter)
            .from_writer(std::fs::File::create(out_filepath)?);

        // Write the headers of the columns to keep to the output file
        let headers_to_keep: Vec<&str> = indices_to_keep.iter().map(|&i| &headers[i]).collect();
        wtr.write_record(&headers_to_keep)?;

        // Read each record, keep only the desired fields, and write to the output file
        for result in reader.records() {
            let record = result?;
            let record_to_keep: Vec<&str> = indices_to_keep.iter().map(|&i| &record[i]).collect();
            wtr.write_record(&record_to_keep)?;
        }

        // Flush the writer to ensure all output is written
        wtr.flush()?;

        info!("Select the columns to keep successfully.");
        debug!(
            "The path of the temporary file is: {}",
            out_filepath.display()
        );

        // TODO: Poor performance, need to optimize?
        Ok(Self::get_records(out_filepath)?) // Return the records of the output file
    }

    fn get_column_names(filepath: &PathBuf) -> Result<Vec<String>, Box<dyn Error>> {
        let delimiter = get_delimiter(filepath)?;
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .from_path(filepath)?;

        let headers = reader.headers()?;
        let mut column_names = Vec::new();
        let expected_columns = Self::fields();
        for header in headers {
            let column = header.to_string();
            // Don't need to check whether all the columns are in the input file, because we have already checked it in the function `check_csv_is_valid`.
            if expected_columns.contains(&column) {
                column_names.push(column);
            } else {
                continue;
            }
        }

        Ok(column_names)
    }

    fn get_records<S: for<'de> serde::Deserialize<'de> + Validate + std::fmt::Debug>(
        filepath: &PathBuf,
    ) -> Result<Vec<S>, Box<dyn Error>> {
        debug!("Start to get records from the csv file: {:?}", filepath);
        let delimiter = get_delimiter(filepath)?;
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .from_path(filepath)?;

        let mut records = Vec::new();
        for result in reader.deserialize::<S>() {
            let record: S = result?;
            records.push(record);
        }

        debug!("Get {} records successfully.", records.len());

        Ok(records)
    }
}

pub struct QueryFilter<'a> {
    pub guid: Option<&'a str>,
    pub filename: Option<&'a str>,
    pub baseid: Option<&'a str>,
    pub status: Option<&'a str>,
    pub uploader: Option<&'a str>,
    pub hash: Option<&'a str>,
    pub alias: Option<&'a str>,
    pub url: Option<&'a str>,
    pub field_name: Option<&'a str>,
    pub field_value: Option<&'a str>,
}

impl<'a> QueryFilter<'a> {
    pub fn new(
        guid: &'a str,
        filename: &'a str,
        baseid: &'a str,
        status: &'a str,
        uploader: &'a str,
        hash: &'a str,
        alias: &'a str,
        url: &'a str,
        tag_field_name: &'a str,
        tag_field_value: &'a str,
    ) -> Self {
        Self {
            guid: if guid.is_empty() { None } else { Some(guid) },
            filename: if filename.is_empty() {
                None
            } else {
                Some(filename)
            },
            baseid: if baseid.is_empty() {
                None
            } else {
                Some(baseid)
            },
            status: if status.is_empty() {
                None
            } else {
                Some(status)
            },
            uploader: if uploader.is_empty() {
                None
            } else {
                Some(uploader)
            },
            hash: if hash.is_empty() { None } else { Some(hash) },
            alias: if alias.is_empty() { None } else { Some(alias) },
            url: if url.is_empty() { None } else { Some(url) },
            field_name: if tag_field_name.is_empty() {
                None
            } else {
                Some(tag_field_name)
            },
            field_value: if tag_field_value.is_empty() {
                None
            } else {
                Some(tag_field_value)
            },
        }
    }

    /// 拼接 SQL WHERE 条件，并返回参数列表
    pub fn to_sql_and_params(&self) -> (String, Vec<String>) {
        let mut clauses = vec![];
        let mut params = vec![];
        let mut param_index = 1;

        macro_rules! push_clause {
            ($cond:expr, $sql_fmt:expr, $val:expr) => {
                if $cond.is_some() {
                    clauses.push(format!($sql_fmt, param_index));
                    if let Some(vv) = $val {
                        params.push(vv.to_string());
                    } else {
                        params.push("".to_string());
                    };
                    param_index += 1;
                }
            };
        }

        push_clause!(self.guid, "f.guid = ${}", self.guid);
        push_clause!(
            self.filename,
            "f.filename ILIKE ${}",
            Some(&format!("%{}%", self.filename.unwrap()))
        );
        push_clause!(self.baseid, "f.baseid = ${}", self.baseid);
        push_clause!(self.status, "f.status = ${}", self.status);
        push_clause!(self.uploader, "f.uploader = ${}", self.uploader);
        push_clause!(
            self.hash,
            "EXISTS (SELECT 1 FROM biominer_indexd_hash h WHERE h.file = f.guid AND h.hash = ${})",
            self.hash
        );
        push_clause!(
            self.alias,
            "EXISTS (SELECT 1 FROM biominer_indexd_alias a WHERE a.file = f.guid AND a.name = ${})",
            self.alias
        );
        push_clause!(
            self.url,
            "EXISTS (SELECT 1 FROM biominer_indexd_url u WHERE u.file = f.guid AND u.url = ${})",
            self.url
        );

        if let (Some(fn_), Some(fv)) = (self.field_name, self.field_value) {
            clauses.push(format!(
                "EXISTS (SELECT 1 FROM biominer_indexd_tag t WHERE t.file = f.guid AND t.field_name = ${} AND t.field_value = ${})",
                param_index,
                param_index + 1
            ));
            params.push(fn_.to_string());
            params.push(fv.to_string());
            param_index += 2;
        }

        let where_clause = if clauses.is_empty() {
            "1=1".to_string()
        } else {
            clauses.join(" AND ")
        };

        (where_clause, params)
    }
}

pub async fn fetch_guid_page(
    pool: &sqlx::PgPool,
    filter: &QueryFilter<'_>,
    page_no: u64,
    page_size: u64,
) -> Result<(Vec<String>, i64), anyhow::Error> {
    let offset = (page_no - 1) * page_size;
    let (where_clause, params) = filter.to_sql_and_params();
    let num_params = params.len();

    let base_sql = format!("FROM biominer_indexd_file f WHERE {}", where_clause);

    let count_sql = format!("SELECT COUNT(*) {}", base_sql);
    let guid_sql = format!(
        "SELECT f.guid {} OFFSET ${} LIMIT ${}",
        base_sql,
        num_params + 1,
        num_params + 2
    );

    // 绑定参数（位置绑定：$1, $2...）
    let mut query = sqlx::query_scalar::<_, i64>(&count_sql);
    let mut guid_query = sqlx::query_scalar::<_, String>(&guid_sql);

    for (i, val) in params.iter().enumerate() {
        query = query.bind(val);
        guid_query = guid_query.bind(val);
    }

    let total = query.fetch_one(pool).await?;
    let guids = guid_query
        .bind(offset as i64)
        .bind(page_size as i64)
        .fetch_all(pool)
        .await?;

    debug!("Query Count SQL:    {:?}", count_sql);
    debug!("Query Count Params: {:?}", params);

    debug!("Query SQL:    {:?}", guid_sql);
    debug!("Query Params: {:?}", params);

    Ok((guids, total))
}

pub async fn load_files_by_guids(
    pool: &sqlx::PgPool,
    guids: &[String],
    include_urls: bool,
    include_aliases: bool,
    include_tags: bool,
) -> Result<Vec<File>, anyhow::Error> {
    if guids.is_empty() {
        return Ok(vec![]);
    }

    let in_clause = format!(
        "({})",
        (1..=guids.len())
            .map(|i| format!("${}", i))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let sql = format!(
        "
            SELECT
                f.guid, f.filename, f.size, f.updated_at, f.baseid, f.rev, f.version, f.acl,
                CASE WHEN f.acl IS NULL THEN 'public' ELSE 'private' END AS access,
                f.created_at, f.status, f.uploader,
                {},
                (SELECT json_agg(h) FROM biominer_indexd_hash h WHERE h.file = f.guid) AS hashes,
                {},
                {}
            FROM biominer_indexd_file f
            WHERE f.guid IN {}
        ",
        if include_urls {
            "(SELECT json_agg(u) FROM biominer_indexd_url u WHERE u.file = f.guid) AS urls"
        } else {
            "NULL AS urls"
        },
        if include_aliases {
            "(SELECT json_agg(a) FROM biominer_indexd_alias a WHERE a.file = f.guid) AS aliases"
        } else {
            "NULL AS aliases"
        },
        if include_tags {
            "(SELECT json_agg(t) FROM biominer_indexd_tag t WHERE t.file = f.guid) AS tags"
        } else {
            "NULL AS tags"
        },
        in_clause
    );

    let mut query = sqlx::query_as::<_, File>(&sql);
    for guid in guids {
        query = query.bind(guid);
    }

    let result = query.fetch_all(pool).await?;
    Ok(result)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct RecordResponse<S>
where
    S: Serialize
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + std::fmt::Debug
        + std::marker::Unpin
        + Send
        + Sync
        + poem_openapi::types::Type
        + poem_openapi::types::ParseFromJSON
        + poem_openapi::types::ToJSON,
{
    /// data
    pub records: Vec<S>,
    /// total num
    pub total: u64,
    /// current page index
    pub page: u64,
    /// default 10
    pub page_size: u64,
}

impl<
        S: Serialize
            + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
            + std::fmt::Debug
            + std::marker::Unpin
            + Send
            + Sync
            + poem_openapi::types::Type
            + poem_openapi::types::ParseFromJSON
            + poem_openapi::types::ToJSON,
    > RecordResponse<S>
{
    pub async fn query_files(
        pool: &sqlx::PgPool,
        filter: QueryFilter<'_>,
        page_no: u64,
        page_size: u64,
        include_urls: bool,
        include_aliases: bool,
        include_tags: bool,
    ) -> Result<RecordResponse<File>, anyhow::Error> {
        let (guids, total) = fetch_guid_page(pool, &filter, page_no, page_size).await?;
        let files =
            load_files_by_guids(pool, &guids, include_urls, include_aliases, include_tags).await?;

        AnyOk(RecordResponse {
            records: files,
            total: total as u64,
            page: page_no,
            page_size: page_size,
        })
    }

    pub async fn get_records(
        pool: &sqlx::PgPool,
        table_name: &str,
        query: &Option<ComposeQuery>,
        page: Option<u64>,
        page_size: Option<u64>,
        order_by: Option<&str>,
        owner: Option<&str>,
    ) -> Result<RecordResponse<S>, anyhow::Error> {
        let mut query_str = match query {
            Some(ComposeQuery::QueryItem(item)) => item.format(),
            Some(ComposeQuery::ComposeQueryItem(item)) => item.format(),
            None => "".to_string(),
        };

        if query_str.is_empty() {
            query_str = "1=1".to_string();
        };

        let order_by_str = if order_by.is_none() {
            "".to_string()
        } else {
            format!("ORDER BY {}", order_by.unwrap())
        };

        let pagination_str = if page.is_none() && page_size.is_none() {
            "LIMIT 10 OFFSET 0".to_string()
        } else {
            let page = match page {
                Some(page) => page,
                None => 1,
            };

            let page_size = match page_size {
                Some(page_size) => page_size,
                None => 10,
            };

            let limit = page_size;
            let offset = (page - 1) * page_size;

            format!("LIMIT {} OFFSET {}", limit, offset)
        };

        let which_owner = if owner.is_some() {
            format!("AND owner = '{}'", owner.unwrap())
        } else {
            "".to_string()
        };

        let query_str = format!("{} {}", query_str, which_owner);

        let sql_str = format!(
            "SELECT * FROM {} WHERE {} {} {}",
            table_name, query_str, order_by_str, pagination_str
        );

        let records = sqlx::query_as::<_, S>(sql_str.as_str())
            .fetch_all(pool)
            .await?;

        let sql_str = format!("SELECT COUNT(*) FROM {} WHERE {}", table_name, query_str);

        let total = sqlx::query_as::<_, (i64,)>(sql_str.as_str())
            .fetch_one(pool)
            .await?;

        AnyOk(RecordResponse {
            records: records,
            total: total.0 as u64,
            page: page.unwrap_or(1),
            page_size: page_size.unwrap_or(10),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow, Validate)]
pub struct FileStatResponse {
    pub total_size: i64,
    pub num_of_files: i64,
    pub num_of_baseid: i64,
    // SELECT REPLACE(CONCAT_WS('', 'v', DATE(TO_TIMESTAMP(updated_at / 1000))), '-', '') FROM biominer_indexd_file ORDER BY updated_at DESC LIMIT 1;
    pub version: String,
    pub registry_id: String,
}

impl FileStatResponse {
    pub async fn get_stat(pool: &sqlx::PgPool) -> Result<FileStatResponse, anyhow::Error> {
        // Database will return null when the table is empty, COALESCE will return 0/'' if the first argument is null
        let stat = sqlx::query_as::<_, FileStatResponse>("
          SELECT 
            COALESCE(SUM(size)::BIGINT, 0) AS total_size, 
            COUNT (guid)::BIGINT AS num_of_files, 
            COUNT(DISTINCT(baseid))::BIGINT AS num_of_baseid,
            COALESCE((SELECT REPLACE(CONCAT_WS('', 'v', DATE(TO_TIMESTAMP(updated_at / 1000))), '-', '') 
              FROM biominer_indexd_file ORDER BY updated_at DESC LIMIT 1), '') AS version,
            (SELECT registry_id FROM biominer_indexd_config LIMIT 1) AS registry_id
          FROM biominer_indexd_file",
        ).fetch_one(pool).await?;

        AnyOk(stat)
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct FieldName {
    pub field_names: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object, sqlx::FromRow)]
pub struct FileTagsResponse {
    pub field_names: Vec<String>,
}

impl FileTagsResponse {
    pub async fn get_fields(pool: &sqlx::PgPool) -> Result<FileTagsResponse, anyhow::Error> {
        let rows = sqlx::query_as::<_, FieldName>(
            "SELECT DISTINCT(field_name) AS field_names FROM biominer_indexd_tag",
        )
        .fetch_all(pool)
        .await?;

        let field_names = rows.into_iter().map(|r| r.field_names).collect();

        AnyOk(FileTagsResponse { field_names })
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Object, sqlx::FromRow)]
pub struct URL {
    #[oai(read_only)]
    pub id: i64,
    #[oai(validator(max_length = 255))]
    pub url: String,
    pub created_at: i64,
    #[oai(validator(max_length = 16))]
    pub status: String, // 'pending', 'processing', 'validated', 'failed'
    #[oai(validator(max_length = 64))]
    pub uploader: String,
    #[oai(validator(max_length = 64))]
    pub file: Option<String>,
}

impl URL {
    pub fn get_identity(&self) -> String {
        let url_parts = self.url.split("/").collect::<Vec<&str>>();
        // NODE: node://<account_name>/<project_id>/<experiment_id>/<sample_id>/<run_id>/<data_id>;
        // S3/OSS/Minio: s3://<bucket_name>/<object_name>;
        // GSA: gsa://<account_name>/<project_id>/<sample_id>/<experiment_id>/<run_id>/<filename>;
        match url_parts[0] {
            "node:" => url_parts[3].to_string(),  // project_id
            "s3:" => url_parts[2].to_string(),    // bucket_name
            "oss:" => url_parts[2].to_string(),   // bucket_name
            "minio:" => url_parts[2].to_string(), // bucket_name
            "gsa:" => url_parts[3].to_string(),   // project_id
            _ => "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Object, Default, sqlx::FromRow)]
pub struct Hash {
    #[oai(read_only)]
    pub id: i64,
    #[oai(validator(max_length = 16))]
    pub hash_type: String, // Max 16 characters, md5, sha1, sha256, sha512, crc32, crc64, etag, etc
    #[oai(validator(max_length = 128))]
    pub hash: String, // Max 128 characters
    #[oai(validator(max_length = 64))]
    pub file: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Object, sqlx::FromRow)]
pub struct Tag {
    #[oai(read_only)]
    pub id: i64,
    #[oai(validator(max_length = 128))]
    pub field_name: String, // Max 128 characters
    #[oai(validator(max_length = 128))]
    pub field_value: String, // Max 128 characters
    #[oai(validator(max_length = 64))]
    pub file: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Object, sqlx::FromRow)]
pub struct Alias {
    #[oai(read_only)]
    pub id: i64,
    #[oai(validator(max_length = 255))]
    pub name: String,
    #[oai(validator(max_length = 64))]
    pub file: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Object, sqlx::FromRow)]
pub struct Config {
    #[oai(read_only)]
    pub id: i64,
    #[oai(validator(max_length = 16, pattern = "^[0-9a-z-]{16}$"))]
    pub registry_id: String,
}

impl Config {
    pub fn new(registry_id: String) -> Self {
        Config {
            id: 0i64,
            registry_id: registry_id,
        }
    }

    // TODO: use a better way to get the registry_id?
    fn get_registry_id() -> String {
        let registry_id = match std::env::var("BIOMIER_REGISTRY_ID") {
            Ok(v) => v,
            Err(_) => "fudan-pgx".to_string(),
        };

        let registry_id_regex = Regex::new(r"^[0-9a-z-]{16}$").unwrap();
        if !registry_id_regex.is_match(&registry_id) {
            warn!("Environment variable `BIOMIER_REGISTRY_ID` is not valid (Regex: ^[0-9a-z-]{{16}}$), use default value: fudan-pgx");
        }

        return registry_id;
    }

    pub async fn init_config(pool: &sqlx::PgPool) -> Result<Config, anyhow::Error> {
        let registry_id = Config::get_registry_id();
        let configs = sqlx::query_as::<_, Config>("SELECT * FROM biominer_indexd_config")
            .fetch_all(pool)
            .await?;

        debug!("Config: {:?}", configs);

        // Configs always be an array, maybe have one or zero record.
        if configs.len() > 0 {
            warn!(
        "Config already exists, if you want to change the registry_id, please rebuild the database first."
      );
            Ok(configs[0].clone())
        } else {
            let v = sqlx::query("INSERT INTO biominer_indexd_config (registry_id) VALUES ($1);")
                .bind(registry_id.clone())
                .execute(pool)
                .await?;

            info!("Set registry_id to {}", registry_id);
            Ok(Config::new(registry_id))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Object, sqlx::FromRow)]
pub struct File {
    pub guid: String,
    pub filename: String,
    pub size: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub status: String,
    pub baseid: String,
    pub rev: String,
    pub version: i32,
    pub uploader: String,
    pub access: String, // public or private
    pub acl: Option<String>,
    pub urls: Option<serde_json::Value>,
    pub hashes: Option<serde_json::Value>,
    pub aliases: Option<serde_json::Value>,
    pub tags: Option<serde_json::Value>,
}

impl File {
    pub fn new(filename: &str, size: i64, uploader: &str, registry_id: &str) -> Self {
        let guid = uuid::Uuid::new_v4().to_string();
        let rev = guid[..8].to_string();
        let baseid = uuid::Uuid::new_v4().to_string();
        let now_ms = Utc::now().timestamp_millis();

        File {
            // biominer.fudan-pgx/3ec4d151-061b-4bcb-ad3a-425c712bfc88
            guid: format!("{}.{}/{}", "biominer", registry_id, guid),
            filename: filename.to_string(),
            size: size,
            created_at: now_ms,
            updated_at: now_ms,
            status: "pending".to_string(),
            baseid: baseid,
            uploader: uploader.to_string(),
            rev: rev,
            version: 1i32,
            access: "public".to_string(),
            acl: None,
            urls: None,
            hashes: None,
            aliases: None,
            tags: None,
        }
    }

    pub async fn query_file(
        pool: &sqlx::PgPool,
        field_name: &str,
        field_value: &str,
    ) -> Result<File, anyhow::Error> {
        if field_name.is_empty() || field_value.is_empty() {
            return Err(anyhow::anyhow!(
                "Field name and field value cannot be empty"
            ));
        }

        let id = if field_name == "guid" {
            let uid = match uuid::Uuid::parse_str(field_value) {
                Ok(id) => id,
                Err(_) => {
                    return Err(anyhow::anyhow!("Invalid guid: {}", field_value));
                }
            };

            File::gen_guid(&uid)
        } else if field_name == "hash" {
            field_value.to_string()
        } else {
            return Err(anyhow::anyhow!("Invalid field name: {}", field_name));
        };

        let sql_str = format!(
            "
                SELECT 
                f.*,

                (
                    SELECT json_agg(
                        jsonb_build_object(
                            'id', u.id,
                            'url', u.url,
                            'created_at', u.created_at,
                            'status', u.status,
                            'uploader', u.uploader,
                            'file', u.file
                        )
                    )
                    FROM biominer_indexd_url u
                    WHERE u.file = f.{field_name}
                ) AS urls,

                (
                    SELECT json_agg(
                        jsonb_build_object(
                            'id', h.id,
                            'hash_type', h.hash_type,
                            'hash', h.hash,
                            'file', h.file
                        )
                    )
                    FROM biominer_indexd_hash h
                    WHERE h.file = f.{field_name}
                ) AS hashes,

                (
                    SELECT json_agg(
                        jsonb_build_object(
                            'id', a.id,
                            'name', a.name,
                            'file', a.file
                        )
                    )
                    FROM biominer_indexd_alias a
                    WHERE a.file = f.{field_name}
                ) AS aliases,

                (
                    SELECT json_agg(
                        jsonb_build_object(
                            'id', t.id,
                            'field_name', t.field_name,
                            'field_value', t.field_value,
                            'file', t.file
                        )
                    )
                    FROM biominer_indexd_tag t
                    WHERE t.file = f.{field_name}
                ) AS tags

                FROM biominer_indexd_file f
                WHERE f.{field_name} = $1;

            ",
            field_name = field_name
        );

        let file = sqlx::query_as::<_, File>(&sql_str)
            .bind(id)
            .fetch_optional(pool)
            .await?;

        if file.is_none() {
            return Err(anyhow::anyhow!(
                "Cannot find the file with {} {}",
                field_name,
                field_value
            ));
        }

        AnyOk(file.unwrap())
    }

    pub async fn get_file(pool: &sqlx::PgPool, id: &uuid::Uuid) -> Result<File, anyhow::Error> {
        File::query_file(pool, "guid", &File::gen_guid(id)).await
    }

    pub async fn get_file_with_hash(
        pool: &sqlx::PgPool,
        hash: &str,
    ) -> Result<File, anyhow::Error> {
        File::query_file(pool, "hash", hash).await
    }

    fn gen_guid(id: &uuid::Uuid) -> String {
        return format!("biominer.{}/{}", Config::get_registry_id(), id);
    }

    pub async fn delete_file(pool: &sqlx::PgPool, id: &uuid::Uuid) -> Result<(), anyhow::Error> {
        let guid = File::gen_guid(id);
        // NOTICE: Be careful, this is a hard delete (delete cascade).
        let v = sqlx::query("DELETE FROM biominer_indexd_file WHERE guid = $1;")
            .bind(&guid)
            .execute(pool)
            .await?;

        if v.rows_affected() >= 1 {
            AnyOk(())
        } else {
            Err(anyhow::anyhow!("Cannot delete the file with guid {}", guid))
        }
    }

    pub async fn check_hash_exists(pool: &sqlx::PgPool, hash: &str) -> Result<bool, anyhow::Error> {
        let v = sqlx::query("SELECT count(*) as count FROM biominer_indexd_hash WHERE hash = $1")
            .bind(hash)
            .fetch_one(pool)
            .await?;

        Ok(v.get::<i64, _>("count") > 0)
    }

    pub async fn add_url(
        pool: &sqlx::PgPool,
        uuid: &uuid::Uuid,
        url: &str,
        uploader: &str,
        status: &str,
    ) -> Result<(), anyhow::Error> {
        let guid = File::gen_guid(uuid);

        // 校验文件是否存在
        let file_exists =
            sqlx::query_scalar::<_, i64>("SELECT 1 FROM biominer_indexd_file WHERE guid = $1")
                .bind(&guid)
                .fetch_optional(pool)
                .await?
                .is_some();

        if !file_exists {
            warn!("Cannot find the file {}.", guid);
            return Err(anyhow::anyhow!("Cannot find the file with guid {}", guid));
        }

        // 合法性校验
        let status = match status {
            "pending" | "processing" | "validated" | "failed" => status.to_string(),
            _ => "pending".to_string(),
        };

        // 插入或更新 URL
        let result = sqlx::query(
            "
                INSERT INTO biominer_indexd_url (file, url, status, uploader)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (file, url)
                DO UPDATE SET status = EXCLUDED.status, uploader = EXCLUDED.uploader
                RETURNING *;
            ",
        )
        .bind(&guid)
        .bind(url)
        .bind(&status)
        .bind(uploader)
        .execute(pool)
        .await?;

        if result.rows_affected() == 1 {
            info!("Add url {} to file {}", url, guid);
        } else {
            info!("Url {} already exists in file {}.", url, guid);
        }

        Ok(())
    }

    pub async fn delete_url(
        pool: &sqlx::PgPool,
        id: &uuid::Uuid,
        url: Option<&str>,
    ) -> Result<(), anyhow::Error> {
        let guid = File::gen_guid(id);

        let result = match url {
            Some(u) => {
                sqlx::query("DELETE FROM biominer_indexd_url WHERE file = $1 AND url = $2")
                    .bind(&guid)
                    .bind(u)
                    .execute(pool)
                    .await?
            }
            None => {
                sqlx::query("DELETE FROM biominer_indexd_url WHERE file = $1")
                    .bind(&guid)
                    .execute(pool)
                    .await?
            }
        };

        if result.rows_affected() >= 1 {
            AnyOk(())
        } else {
            Err(anyhow::anyhow!(
                "Cannot delete the url with guid {} and url {:?}",
                guid,
                url
            ))
        }
    }

    async fn check_file_exists(pool: &sqlx::PgPool, guid: &str) -> Result<bool, anyhow::Error> {
        let exists =
            sqlx::query_scalar::<_, i64>("SELECT 1 FROM biominer_indexd_file WHERE guid = $1")
                .bind(guid)
                .fetch_optional(pool)
                .await?
                .is_some();

        Ok(exists)
    }

    pub async fn add_alias(
        pool: &sqlx::PgPool,
        uuid: &uuid::Uuid,
        alias: &str,
    ) -> Result<(), anyhow::Error> {
        let guid = File::gen_guid(uuid);

        if !File::check_file_exists(pool, &guid).await? {
            warn!("Cannot find the file {}.", guid);
            return Err(anyhow::anyhow!("Cannot find the file with {}", guid));
        }

        let result = sqlx::query(
            "INSERT INTO biominer_indexd_alias (file, name) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(&guid)
        .bind(alias)
        .execute(pool)
        .await?;

        if result.rows_affected() == 1 {
            info!("Add alias {} to file {}", alias, guid);
        } else {
            info!("Alias {} already exists in file {}.", alias, guid);
        }

        Ok(())
    }

    pub async fn delete_alias(
        pool: &sqlx::PgPool,
        id: &uuid::Uuid,
        name: Option<&str>,
    ) -> Result<(), anyhow::Error> {
        let guid = File::gen_guid(id);
        let result = match name {
            Some(n) => {
                sqlx::query("DELETE FROM biominer_indexd_alias WHERE file = $1 AND name = $2")
                    .bind(&guid)
                    .bind(n)
                    .execute(pool)
                    .await?
            }
            None => {
                sqlx::query("DELETE FROM biominer_indexd_alias WHERE file = $1")
                    .bind(&guid)
                    .execute(pool)
                    .await?
            }
        };

        if result.rows_affected() >= 1 {
            AnyOk(())
        } else {
            Err(anyhow::anyhow!(
                "Cannot delete the alias with {} and {:?}",
                guid,
                name
            ))
        }
    }

    pub async fn add_tag(
        pool: &sqlx::PgPool,
        uuid: &uuid::Uuid,
        field_name: &str,
        field_value: &str,
    ) -> Result<(), anyhow::Error> {
        let guid = File::gen_guid(uuid);

        if !File::check_file_exists(pool, &guid).await? {
            warn!("Cannot find the file {}.", guid);
            return Err(anyhow::anyhow!("Cannot find the file with guid {}", guid));
        }

        // 插入或更新 tag
        let result = sqlx::query(
            "
                INSERT INTO biominer_indexd_tag (file, field_name, field_value)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (file, field_name)
                    DO UPDATE SET field_value = EXCLUDED.field_value
                    RETURNING *;
            ",
        )
        .bind(&guid)
        .bind(field_name)
        .bind(field_value)
        .execute(pool)
        .await?;

        if result.rows_affected() == 1 {
            info!(
                "Add tag \"{}:{}\" to file {}",
                field_name, field_value, guid
            );
        } else {
            info!(
                "Tag {} already exists in file {}, updated instead.",
                field_name, guid
            );
        }

        Ok(())
    }

    pub async fn delete_tag(
        pool: &sqlx::PgPool,
        id: &uuid::Uuid,
        field_name: Option<&str>,
    ) -> Result<(), anyhow::Error> {
        let guid = File::gen_guid(id);

        let v = match field_name {
            Some(h) => {
                sqlx::query("DELETE FROM biominer_indexd_tag WHERE file = $1 AND field_name = $2;")
                    .bind(&guid)
                    .bind(h)
                    .execute(pool)
                    .await?
            }
            None => {
                sqlx::query("DELETE FROM biominer_indexd_tag WHERE file = $1;")
                    .bind(&guid)
                    .execute(pool)
                    .await?
            }
        };

        if v.rows_affected() >= 1 {
            AnyOk(())
        } else {
            return Err(anyhow::anyhow!(
                "Cannot delete the tag with {} and {:?}",
                guid,
                field_name
            ));
        }
    }

    pub async fn add_hash(
        pool: &sqlx::PgPool,
        uuid: &uuid::Uuid,
        hash: &str,
    ) -> Result<(), anyhow::Error> {
        let guid = File::gen_guid(uuid);

        if !File::check_file_exists(pool, &guid).await? {
            warn!("Cannot find the file {}.", guid);
            return Err(anyhow::anyhow!("Cannot find the file with guid {}", guid));
        }

        // 判定 hash 类型
        let hash_type = match util::which_hash_type(hash) {
            Some(ht) => ht,
            None => {
                warn!("Cannot determine hash type of {}", hash);
                return Err(anyhow::anyhow!(
                    "Unsupported hash: {}. Only support md5, sha1, sha256, sha512, crc32, etag, crc64.",
                    hash
                ));
            }
        };

        // 插入 hash（去重）
        let result = sqlx::query(
            "
                INSERT INTO biominer_indexd_hash (file, hash_type, hash)
                VALUES ($1, $2, $3)
                ON CONFLICT DO NOTHING
                RETURNING *;
            ",
        )
        .bind(&guid)
        .bind(hash_type)
        .bind(hash)
        .execute(pool)
        .await?;

        if result.rows_affected() == 1 {
            info!("Add hash {} to file {}", hash, guid);
        } else {
            info!("Hash {} already exists in file {}.", hash, guid);
        }

        Ok(())
    }

    pub async fn delete_hash(
        pool: &sqlx::PgPool,
        id: &uuid::Uuid,
        hash: Option<&str>,
    ) -> Result<(), anyhow::Error> {
        let guid = File::gen_guid(id);

        let v = match hash {
            Some(h) => {
                sqlx::query("DELETE FROM biominer_indexd_hash WHERE file = $1 AND hash = $2;")
                    .bind(&guid)
                    .bind(h)
                    .execute(pool)
                    .await?
            }
            None => {
                sqlx::query("DELETE FROM biominer_indexd_hash WHERE file = $1;")
                    .bind(&guid)
                    .execute(pool)
                    .await?
            }
        };

        if v.rows_affected() >= 1 {
            AnyOk(())
        } else {
            return Err(anyhow::anyhow!(
                "Cannot delete the hash with {} and {:?}",
                guid,
                hash
            ));
        }
    }

    pub async fn add(
        &mut self,
        pool: &sqlx::PgPool,
        hash: &str,
        url: Option<&str>,
        alias: Option<&str>,
    ) -> Result<(), anyhow::Error> {
        // 开始事务
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = match pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                warn!("Cannot acquire transaction: {:?}", e);
                return Err(anyhow::anyhow!("Failed to start transaction: {}", e));
            }
        };

        // 插入 File（忽略冲突）
        let insert_file = sqlx::query(
        "
                INSERT INTO biominer_indexd_file 
                    (guid, filename, size, created_at, updated_at, status, baseid, uploader, rev, version)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                    ON CONFLICT DO NOTHING
                    RETURNING *;
            ",
        )
        .bind(&self.guid)
        .bind(&self.filename)
        .bind(self.size)
        .bind(self.created_at)
        .bind(self.updated_at)
        .bind(&self.status)
        .bind(&self.baseid)
        .bind(&self.uploader)
        .bind(&self.rev)
        .bind(self.version)
        .execute(&mut tx)
        .await;

        if let Err(e) = insert_file {
            warn!("Insert File Error: {:?}", e);
            tx.rollback().await?;
            return Err(anyhow::anyhow!("Failed to insert file: {}", e));
        }

        // 插入 Hash
        let insert_hash = sqlx::query(
            "
                INSERT INTO biominer_indexd_hash (hash, hash_type, file)
                    VALUES ($1, $2, $3)
                    ON CONFLICT DO NOTHING
                    RETURNING *;
            ",
        )
        .bind(hash)
        .bind("md5") // 默认 hash_type
        .bind(&self.guid)
        .execute(&mut tx)
        .await;

        if let Err(e) = insert_hash {
            warn!("Insert Hash Error: {:?}", e);
            tx.rollback().await?;
            return Err(anyhow::anyhow!(
                "The hash ({}) already exists or has been registered.",
                hash
            ));
        }

        // 插入 URL（如果有）
        if let Some(u) = url {
            let insert_url = sqlx::query(
                "
                    INSERT INTO biominer_indexd_url (file, url, uploader)
                        VALUES ($1, $2, $3)
                        ON CONFLICT DO NOTHING
                        RETURNING *;
                ",
            )
            .bind(&self.guid)
            .bind(u)
            .bind(&self.uploader)
            .execute(&mut tx)
            .await;

            if let Err(e) = insert_url {
                warn!("Insert URL Error: {:?}", e);
                tx.rollback().await?;
                return Err(anyhow::anyhow!(
                    "The URL ({}) already exists or has been registered.",
                    u
                ));
            }
        }

        // 插入 Alias（如果有）
        if let Some(a) = alias {
            let insert_alias = sqlx::query(
                "
                    INSERT INTO biominer_indexd_alias (file, name)
                        VALUES ($1, $2)
                        ON CONFLICT DO NOTHING
                        RETURNING *;
                ",
            )
            .bind(&self.guid)
            .bind(a)
            .execute(&mut tx)
            .await;

            if let Err(e) = insert_alias {
                warn!("Insert Alias Error: {:?}", e);
                tx.rollback().await?;
                return Err(anyhow::anyhow!("Failed to insert alias: {}", e));
            }
        }

        // 提交事务
        tx.commit().await?;
        Ok(())
    }
}

// TODO: How to set test env?
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{connect_db, run_migrations};

    async fn init() -> sqlx::PgPool {
        let database_url = match std::env::var("DATABASE_URL") {
            Ok(v) => v,
            Err(msg) => {
                "postgres://postgres:password@localhost:5432/test_biominer_indexd".to_string()
            }
        };

        return connect_db(&database_url, 1).await;
    }

    #[tokio::test]
    async fn test_query_files() {
        let pool = init().await;
        let files = RecordResponse::<File>::query_files(
            &pool,
            QueryFilter::new("", "", "", "", "", "", "", "", "", ""),
            1,
            10,
            true,
            true,
            true,
        )
        .await
        .unwrap();

        assert!(files.total > 0);
    }

    #[tokio::test]
    async fn test_query_file() {
        let pool = init().await;
        let file = File::query_file(
            &pool,
            "guid",
            "biominer.fudan-pgx/3ec4d151-061b-4bcb-ad3a-425c712bfc88",
        )
        .await
        .unwrap();

        assert!(file.guid == "biominer.fudan-pgx/3ec4d151-061b-4bcb-ad3a-425c712bfc88");
    }
}
