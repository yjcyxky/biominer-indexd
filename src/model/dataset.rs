// =====================================================================================
// Dataset Manager (Rust)
//
// This module provides a dataset management system for structured datasets stored in a
// file-based directory format. Each dataset resides in a subdirectory under a common
// `data_dir` and includes metadata and data in standardized formats.
//
// ## Structure
// - `index.json`: A JSON array of datasets with fields: key, name, description, citation,
//   pmid, groups, tags, total, is_filebased
// - Each dataset folder contains:
//   - `dataset.json`: study metadata
//   - `data_dictionary.json`: defines schema for the dataset
//   - `metadata_table.parquet`: tabular file for metadata, column names correspond to keys in dictionary
//   - `datafiles`: a directory contains multiple datafiles, each datafile is a parquet file
//     - `maf.parquet`: maf file
//     - `maf_dictionary.json`: maf dictionary
//     - `mrna_expr.parquet`: mrna_expr file
//     - `mrna_expr_dictionary.json`: mrna_expr dictionary
//     - ...
//   - `datafile.tsv`: datafiles' metadata which contains the file path, file size, etc.
//   - `license.md`: [Optional] license information for the dataset
//   - `dataset.tar.gz`: [Optional, only for cBioPortal dataset] a tarball of the dataset
//
// ## Features
// - Loads and validates dataset metadata and structure
// - Validates that dictionary keys are lowercase letters/digits/underscores and start with a letter
// - Validates that each field has a supported type: STRING, NUMBER, BOOLEAN
// - Supports SQL search over `index.json` using DuckDB
// - Supports SQL queries over individual dataset Parquet files
// - Provides a typed interface to load and inspect a dataset's dictionary
// - Implements caching for dataset metadata, data dictionary, and datafiles
//
// ## Requirements
// - DuckDB
// - Parquet + JSON files per the structure above
//
// ## Usage Example
// ```rust
// // Initialize the cache first
// init_cache(PathBuf::from("data_dir"))?;
//
// // Load and validate datasets
// let datasets = Datasets::load(PathBuf::from("data_dir"))?;
// datasets.validate()?;
//
// // Search datasets
// let results = Datasets::search(
//     &PathBuf::from("data_dir"),
//     &None,
//     Some(1),
//     Some(10),
//     Some("name ASC")
// )?;
//
// // Get a specific dataset
// let dataset = Datasets::get("dataset_key")?;
//
// // Search within a dataset
// let data = dataset.search(
//     &None,
//     Some(1),
//     Some(10),
//     Some("field_name DESC")
// )?;
//
// // Get dataset metadata
// let dict = dataset.get_data_dictionary()?;
// let license = dataset.get_license()?;
// let datafiles = dataset.get_datafiles()?;
// ```
// =====================================================================================

use super::util::load_tsv;
use super::{datafile::File, duckdb_util::row_to_json};
use crate::model::data_dictionary::DataDictionary;
use crate::model::data_table::{
    DataFileTable, MAFTable, MRNAExprTable, MetadataTable, DATA_FILE_TABLES,
};
use crate::model::dataset_metadata::DatasetMetadata;
use crate::query_builder::query_plan::{QueryPlan, SelectExpr};
use crate::query_builder::where_builder::ComposeQuery;
use anyhow::{bail, Context, Error, Result};
use duckdb::{params, Connection};
use lazy_static::lazy_static;
use log::{info, warn};
use poem_openapi::Object;
use polars::prelude::LazyFrame;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::query;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::{fs, path::Path, path::PathBuf};

// Cache the dataset metadata and data dictionary for better performance
lazy_static! {
    static ref DATASET_CACHE: Mutex<HashMap<String, HashMap<String, Dataset>>> =
        Mutex::new(HashMap::new());
    static ref DATA_DICTIONARY_CACHE: Mutex<HashMap<String, HashMap<String, DataDictionary>>> =
        Mutex::new(HashMap::new());
    static ref DATAFILE_CACHE: Mutex<HashMap<String, HashMap<String, Vec<File>>>> =
        Mutex::new(HashMap::new());
}

pub fn get_version_key(key: &str, version: &str) -> String {
    format!("{}:{}", key, version)
}

/// Initialize the cache for the dataset metadata, data dictionary, and datafiles.
///
/// This function loads all datasets from the specified base path and caches their metadata,
/// data dictionary, and datafiles in memory for faster access. You must call this function
/// before using any dataset operations that rely on cached data.
///
/// # Arguments
/// * `base_path` - The path to the root directory containing `index.json` and dataset subdirectories.
///
/// # Returns
/// Returns `Ok(())` if the cache is successfully initialized, or an `Err(Error)` if the cache
/// initialization fails.
///
/// # Errors
/// This function returns an error if:
/// - The dataset metadata cannot be loaded from the specified base path
/// - The data dictionary cannot be loaded for any dataset
/// - The datafiles cannot be loaded for any dataset
///
/// # Example
/// ```rust
/// init_cache(PathBuf::from("data_dir"))?;
/// ```
pub fn init_cache(base_path: &PathBuf) -> Result<(), Error> {
    let datasets = Datasets::load(base_path)?;
    for dataset in datasets.records {
        // Data dictionary
        let data_dictionary = DataDictionary::load_metadata_dictionary(&dataset.path)?;
        {
            let mut dict_cache = DATA_DICTIONARY_CACHE.lock().unwrap();
            dict_cache
                .entry(dataset.metadata.key.clone())
                .or_insert_with(HashMap::new)
                .insert(dataset.metadata.version.clone(), data_dictionary);
        }

        // Dataset metadata
        {
            let mut meta_cache = DATASET_CACHE.lock().unwrap();
            meta_cache
                .entry(dataset.metadata.key.clone())
                .or_insert_with(HashMap::new)
                .insert(dataset.metadata.version.clone(), dataset.clone());
        }

        // Datafiles
        let datafiles = File::from_file(&dataset.path.join("datafile.tsv"))?;
        {
            let mut file_cache = DATAFILE_CACHE.lock().unwrap();
            file_cache
                .entry(dataset.metadata.key.clone())
                .or_insert_with(HashMap::new)
                .insert(dataset.metadata.version.clone(), datafiles);
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Dataset {
    pub metadata: DatasetMetadata,
    pub metadata_table: MetadataTable,
    pub datafile_tables: HashMap<String, Option<DataFileTable>>,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Datasets {
    pub records: Vec<Dataset>,
    pub base_path: PathBuf,
}

#[derive(Serialize, Deserialize, Clone, Object)]
pub struct DatasetsResponse {
    pub records: Vec<DatasetMetadata>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

#[derive(Serialize, Deserialize, Clone, Object)]
pub struct DatasetDataResponse {
    pub records: Vec<serde_json::Value>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

impl Datasets {
    /// Loads a dataset collection from the specified base directory.
    ///
    /// This function attempts to load a dataset index from a file named `index.json`
    /// located in the given `base_path`. It reads the file contents, parses the JSON into
    /// a list of `DatasetMetadata`, and constructs `Dataset` instances for each entry.
    ///
    /// Each dataset's file path is resolved relative to the `base_path`, using the `key`
    /// field from its corresponding metadata entry.
    ///
    /// # Arguments
    ///
    /// * `base_path` - A reference to the path where the dataset index (`index.json`) and
    ///   corresponding dataset files are located.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` if the index file is successfully read and parsed, and all
    /// dataset paths are constructed. If an error occurs during file I/O or JSON parsing,
    /// it returns an `Err(Error)`.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following cases:
    /// - The `index.json` file does not exist at the specified `base_path`.
    /// - The file cannot be read (e.g., due to permission issues).
    /// - The file content is not valid JSON or does not match the expected format.
    ///
    /// # Example
    ///
    /// ```rust
    /// let base_path = Path::new("/path/to/dataset");
    /// let datasets = Datasets::load(base_path)?;
    /// ```
    pub fn load(base_path: &PathBuf) -> Result<Self, Error> {
        let index_path = base_path.join("index.json");
        let content = match fs::read_to_string(&index_path) {
            Ok(content) => content,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to read index file: {}", e));
            }
        };
        let index_entries: Vec<DatasetMetadata> = match serde_json::from_str(&content) {
            Ok(entries) => entries,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to parse index file: {}", e));
            }
        };

        let entries = index_entries
            .into_iter()
            .map(|entry| {
                let path = base_path.join(&entry.key).join(&entry.version);
                Dataset::load(&path).expect("Failed to load dataset")
            })
            .collect();

        Ok(Self {
            records: entries,
            base_path: base_path.to_path_buf(),
        })
    }

    /// Validates that the fields in the data dictionary match the columns in the parquet file.
    ///
    /// This function reads a parquet file and checks that all fields defined in the data dictionary
    /// are present in the file's columns. It is used to ensure that the dictionary and data files
    /// are consistent.
    ///
    /// # Arguments
    ///
    /// * `dict` - A reference to the data dictionary to validate against the parquet file.
    /// * `parquet_path` - The path to the parquet file to validate.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all fields in the dictionary are present in the parquet file, otherwise returns an `Err(Error)`
    /// with a descriptive message about the failure.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - The parquet file cannot be read.
    /// - The parquet file does not contain all fields defined in the dictionary.
    ///
    /// # Example
    ///
    /// ```rust
    /// let dict = DataDictionary::load_metadata_dictionary(&dataset.path)?;
    /// let parquet_path = dataset.path.join("metadata_table.parquet");
    /// Datasets::validate_fields_against_parquet(&dict, &parquet_path)?;
    /// ```
    pub fn validate_fields_against_parquet(
        dict: &DataDictionary,
        parquet_path: &PathBuf,
    ) -> Result<()> {
        // 读取 parquet 文件的列名
        let df = LazyFrame::scan_parquet(parquet_path, Default::default())?.collect()?; // 触发读取
        let parquet_columns: HashSet<String> = df
            .get_column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();

        for field in &dict.fields {
            if !parquet_columns.contains(&field.key) {
                warn!(
                    "Field key '{}' defined in dictionary but missing in metadata_table.parquet.",
                    field.key
                );
            }
        }

        Ok(())
    }

    /// Validates the dataset directory structure and contents at the specified base path.
    ///
    /// This function performs validation on a dataset collection rooted at `base_path`. It expects:
    /// - A file named `index.json` in the `base_path` directory.
    /// - A subdirectory for each dataset entry listed in the index.
    /// - Each dataset directory to contain a valid data dictionary and a `metadata_table.parquet` file.
    ///
    /// The validation steps include:
    /// 1. Checking that `base_path` exists and is a directory.
    /// 2. Ensuring the `index.json` file exists and can be parsed.
    /// 3. Verifying each dataset directory exists.
    /// 4. Validating that each field in the dataset's data dictionary:
    ///    - Has a key matching the regex: `^[a-z][a-z0-9_]*$`
    ///    - Uses a valid data type: `"STRING"`, `"NUMBER"`, or `"BOOLEAN"`
    /// 5. Ensuring that a `metadata_table.parquet` file exists in each dataset directory.
    ///
    /// # Arguments
    ///
    /// * `base_path` - The path to the root directory containing `index.json` and dataset subdirectories.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all datasets and their metadata pass validation, otherwise returns an `Err(Error)`
    /// with a descriptive message about the failure.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - `base_path` is not a directory.
    /// - `index.json` is missing or cannot be parsed.
    /// - A dataset directory is missing.
    /// - A dataset has invalid field keys or unsupported data types.
    /// - The `metadata_table.parquet` file is missing.
    ///
    /// # Example
    ///
    /// ```rust
    /// let base_path = PathBuf::from("/path/to/datasets");
    /// validate(&base_path)?;
    /// ```
    pub fn validate(base_path: &PathBuf) -> Result<(), Error> {
        if !base_path.is_dir() {
            bail!("Base path {:?} is not a directory", base_path);
        }

        if !base_path.join("index.json").exists() {
            bail!(
                "Index file {:?} does not exist",
                base_path.join("index.json")
            );
        }

        // 1. Check index.json which contains all datasets' metadata
        let index_path = base_path.join("index.json");
        let content = fs::read_to_string(&index_path)?;
        let index_entries: Vec<DatasetMetadata> = match serde_json::from_str(&content) {
            Ok(entries) => entries,
            Err(e) => {
                bail!("Failed to parse index file: {}", e);
            }
        };

        if index_entries.len() == 0 {
            bail!("No datasets found in index.json, you might forget to index the datasets.");
        }

        // 2. Load all datasets' metadata from index.json
        let mut records = Vec::new();
        for entry in index_entries {
            let dataset = Dataset::load(&base_path.join(&entry.key).join(&entry.version))?;
            records.push(dataset);
        }

        // 3. Validate each dataset's metadata
        let key_re = Regex::new(r"^[a-z][a-z0-9_]*$").unwrap();
        for dataset in &records {
            if !dataset.path.is_dir() {
                warn!("Dataset directory {:?} does not exist", dataset.path);
                continue;
            }

            let dict = DataDictionary::load_metadata_dictionary(&dataset.path)?;
            for field in &dict.fields {
                if !key_re.is_match(&field.key) {
                    warn!(
                        "Invalid key '{}' in dataset '{}'.",
                        field.key, dataset.metadata.key
                    );

                    continue;
                }

                if !matches!(field.data_type.as_str(), "STRING" | "NUMBER" | "BOOLEAN") {
                    warn!(
                        "Invalid data_type '{}' in dataset '{}', key '{}'.",
                        field.data_type, dataset.metadata.key, field.key
                    );

                    continue;
                }
            }

            // 4. Check whether the metadata_table.parquet file exists
            let parquet_path = dataset.path.join("metadata_table.parquet");
            if !parquet_path.exists() {
                warn!("Missing metadata_table.parquet in {:?}", dataset.path);
                continue;
            }

            // 5. Check whether the dict.fields.key is the same as the metadata_table.parquet.columns
            Datasets::validate_fields_against_parquet(&dict, &parquet_path)?;

            // 6. Check whether the datafile.tsv file exists
            let datafile_path = dataset.path.join("datafile.tsv");
            if !datafile_path.exists() {
                warn!("Missing datafile.tsv in {:?}", dataset.path);
                continue;
            }

            match load_tsv(&datafile_path.to_path_buf()) {
                Ok(datafiles) => {}
                Err(e) => {
                    warn!("Failed to load datafile.tsv in {:?}: {}", dataset.path, e);
                    continue;
                }
            }

            // 7. Check whether the data files' dictionary is the same as the data files' columns
            for (table_name, datafile_table) in &dataset.datafile_tables {
                if datafile_table.is_none() {
                    warn!(
                        "Missing datafile table {:?} in {:?}",
                        table_name, dataset.path
                    );
                    continue;
                }

                let datafile_table = datafile_table.as_ref().unwrap();

                todo!()
            }
        }

        println!("✅ All datasets validated successfully.");
        Ok(())
    }

    /// Searches the dataset index using an optional query with pagination and sorting.
    ///
    /// This method allows flexible querying over the dataset index (`index.json`) using DuckDB.
    /// It supports optional filtering, sorting, and pagination.
    ///
    /// # Arguments
    ///
    /// * `base_path` - The path to the root directory containing the dataset index
    /// * `query` - An optional query (`ComposeQuery`) to filter datasets. If `None`, all datasets are returned
    /// * `page` - An optional page number (1-based). Defaults to 1 if not provided
    /// * `page_size` - An optional page size. Defaults to 10 if not provided
    /// * `order_by` - An optional SQL `ORDER BY` clause (e.g., `"name ASC"`, `"total DESC"`)
    ///
    /// # Returns
    ///
    /// Returns `Ok(DatasetsResponse)` containing:
    /// - `records`: Vector of matching `DatasetMetadata`
    /// - `total`: Total number of matching records
    /// - `page`: Current page number
    /// - `page_size`: Number of records per page
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `index.json` file cannot be read
    /// - DuckDB query preparation or execution fails
    /// - JSON deserialization from database fields fails
    ///
    /// # Example
    /// ```rust
    /// let results = Datasets::search(
    ///     &PathBuf::from("data_dir"),
    ///     &Some(ComposeQuery::QueryItem(...)),
    ///     Some(2),
    ///     Some(5),
    ///     Some("name ASC")
    /// )?;
    /// ```
    ///
    /// # Note
    ///
    /// This function depends on the `read_json_auto` virtual table feature in Duckdb.
    pub fn search(
        base_path: &PathBuf,
        query: &Option<ComposeQuery>,
        page: Option<usize>,
        page_size: Option<usize>,
        order_by: Option<&str>,
    ) -> Result<DatasetsResponse, Error> {
        let index_path = base_path.join("index.json");
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE datasets AS SELECT * FROM read_json(?)",
            params![index_path.to_str().unwrap()],
        )?;

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

        let sql = format!(
            "SELECT key, version, name, description, citation, pmid, json(groups) AS groups, json(tags) AS tags, total, is_filebased FROM datasets WHERE {} {} {}",
            query_str, order_by_str, pagination_str
        );

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            let record = row_to_json(
                row,
                &[
                    "key".to_string(),
                    "version".to_string(),
                    "name".to_string(),
                    "description".to_string(),
                    "citation".to_string(),
                    "pmid".to_string(),
                    "groups".to_string(),
                    "tags".to_string(),
                    "total".to_string(),
                    "is_filebased".to_string(),
                ],
            );

            Ok(record.unwrap())
        })?;

        let results: Vec<DatasetMetadata> = rows
            .map(|row| {
                row.map(DatasetMetadata::from_value)
                    .map_err(|e| anyhow::anyhow!("Error querying data: {}", e))
            })
            .collect::<Result<Vec<DatasetMetadata>, Error>>()?;

        let total_sql = format!("SELECT COUNT(*) FROM datasets WHERE {}", query_str);
        let total: i64 = conn.query_row(&total_sql, [], |row| row.get(0))?;

        Ok(DatasetsResponse {
            records: results,
            total: total as usize,
            page: page.unwrap_or(1),
            page_size: page_size.unwrap_or(10),
        })
    }

    /// Retrieves a dataset by its unique key.
    ///
    /// This method searches through the loaded dataset records and returns the one
    /// that matches the given `key`.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice representing the unique key of the dataset to retrieve.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Dataset)` if a dataset with the specified key is found, or an `Err(Error)`
    /// if no matching dataset exists.
    ///
    /// # Errors
    ///
    /// Returns an error if no dataset with the given key is found.
    ///
    /// # Example
    ///
    /// ```rust
    /// let dataset = Datasets::get("tcga_brca")?;
    /// ```
    pub fn get(key: &str) -> Result<Vec<Dataset>, Error> {
        let dataset_cache = DATASET_CACHE.lock().unwrap();
        let dataset = dataset_cache.get(key);
        if dataset.is_none() {
            return Err(anyhow::anyhow!(
                "Dataset not found: {}, it may not be cached or does not exist.",
                key
            ));
        }

        Ok(dataset.unwrap().values().cloned().collect())
    }

    pub fn get_by_version(key: &str, version: &str) -> Result<Dataset, Error> {
        let dataset_cache = DATASET_CACHE.lock().unwrap();
        let dataset = dataset_cache.get(key);
        if dataset.is_none() {
            return Err(anyhow::anyhow!(
                "Dataset not found: {}, it may not be cached or does not exist.",
                key
            ));
        }

        let dataset_by_version = dataset.unwrap().get(version);
        if dataset_by_version.is_none() {
            return Err(anyhow::anyhow!(
                "Dataset version not found: {}, it may not be cached or does not exist.",
                version
            ));
        }

        Ok(dataset_by_version.unwrap().clone())
    }

    /// Indexes all datasets within the specified base directory.
    ///
    /// This function scans the given `base_path` for subdirectories, attempts to load each as a `Dataset`,
    /// and constructs a new dataset collection. Optionally, it can serialize the metadata of all
    /// datasets into an `index.json` file in the base directory.
    ///
    /// # Arguments
    ///
    /// * `base_path` - The root directory containing dataset subdirectories.
    /// * `save_to_file` - If `true`, saves a generated `index.json` file with the metadata of all discovered datasets.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` with the newly created dataset collection if successful, or an `Err(Error)` if any
    /// part of the indexing or writing process fails.
    ///
    /// # Behavior
    ///
    /// - Scans each subdirectory in `base_path`.
    /// - Calls `Dataset::load` on each subdirectory.
    /// - If `save_to_file` is `true`, serializes the metadata into `index.json`.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - The base directory cannot be read.
    /// - A subdirectory cannot be loaded as a valid `Dataset`.
    /// - Writing to `index.json` fails when `save_to_file` is enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// let datasets = Datasets::index(Path::new("/path/to/datasets"), true)?;
    /// ```
    pub fn index(base_path: &Path, save_to_file: bool) -> Result<Self> {
        let mut datasets = Vec::new();

        // Iterate over <base_path>/<dataset_key>/<dataset_version>
        for dataset_key_entry in fs::read_dir(base_path)
            .with_context(|| format!("Failed to read base directory: {:?}", base_path))?
        {
            let dataset_key_entry = dataset_key_entry?;
            let dataset_key_path = dataset_key_entry.path();

            if !dataset_key_path.is_dir() {
                continue;
            }

            for version_entry in fs::read_dir(&dataset_key_path).with_context(|| {
                format!(
                    "Failed to read dataset_key directory: {:?}",
                    dataset_key_path
                )
            })? {
                let version_entry = version_entry?;
                let version_path = version_entry.path();

                if !version_path.is_dir() {
                    continue;
                }

                match Dataset::load(&version_path) {
                    Ok(dataset) => datasets.push(dataset),
                    Err(e) => {
                        warn!("Failed to load dataset at {:?}: {}", version_path, e);
                        continue;
                    }
                }
            }
        }

        // Optional save index.json
        if save_to_file {
            let index_path = base_path.join("index.json");
            let index_entries: Vec<DatasetMetadata> =
                datasets.iter().map(|d| d.metadata.clone()).collect();
            fs::write(&index_path, serde_json::to_string_pretty(&index_entries)?)
                .with_context(|| format!("Failed to write index file: {:?}", index_path))?;
        }

        Ok(Self {
            records: datasets,
            base_path: base_path.to_path_buf(),
        })
    }
}

impl Dataset {
    /// Loads a single dataset from the specified directory path.
    ///
    /// This function reads the `dataset.json` metadata file located in the given `dataset_path`,
    /// parses its contents into a `DatasetMetadata` struct, and constructs a `Dataset` instance.
    ///
    /// # Arguments
    ///
    /// * `dataset_path` - The path to the directory containing the dataset, including the `dataset.json` file.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` if the dataset metadata is successfully read and parsed, or an `Err(Error)`
    /// if the file is missing or cannot be deserialized.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - `dataset.json` does not exist at the given path.
    /// - The file cannot be read (e.g., due to permissions).
    /// - The JSON content is invalid or does not match the `DatasetMetadata` structure.
    ///
    /// # Example
    ///
    /// ```rust
    /// let dataset = Dataset::load(Path::new("/path/to/dataset"))?;
    /// ```
    pub fn load(dataset_path: &PathBuf) -> Result<Self, Error> {
        let metadata = DatasetMetadata::from_file(&dataset_path)?;
        let metadata_table = MetadataTable::new(&dataset_path)?;

        let mut datafile_tables: HashMap<String, Option<DataFileTable>> = HashMap::new();
        let path = dataset_path.join("datafiles");

        for table_name in DATA_FILE_TABLES.lock().unwrap().iter() {
            if table_name.to_string() == "maf" {
                match MAFTable::new(&path) {
                    Ok(datafile_table) => {
                        let datafile_table = DataFileTable::MAF(datafile_table);
                        datafile_tables.insert(table_name.to_string(), Some(datafile_table));
                    }
                    Err(e) => {
                        warn!("{}", e);
                        datafile_tables.insert(table_name.to_string(), None);
                    }
                }
            } else if table_name.to_string() == "mrna_expr" {
                match MRNAExprTable::new(&path) {
                    Ok(datafile_table) => {
                        let datafile_table = DataFileTable::MRNAExpr(datafile_table);
                        datafile_tables.insert(table_name.to_string(), Some(datafile_table));
                    }
                    Err(e) => {
                        warn!("{}", e);
                        datafile_tables.insert(table_name.to_string(), None);
                    }
                }
            }
        }

        Ok(Self {
            metadata,
            path: dataset_path.to_path_buf(),
            metadata_table,
            datafile_tables,
        })
    }

    /// Searches records within the dataset's Parquet file using an optional SQL-like query,
    /// pagination, and sorting.
    ///
    /// This function reads the dataset's `metadata_table.parquet` file into DuckDB and performs a query
    /// with optional filtering, ordering, and pagination.
    ///
    /// # Arguments
    ///
    /// * `query` - Optional `ComposeQuery` to filter records. If `None`, all records are returned
    /// * `page` - Optional page number (1-based). Defaults to 1 if not specified
    /// * `page_size` - Optional number of records per page. Defaults to 10 if not specified
    /// * `order_by` - Optional SQL `ORDER BY` clause string (e.g., `"age DESC"`, `"name ASC"`)
    ///
    /// # Returns
    ///
    /// Returns `Ok(DatasetDataResponse)` containing:
    /// - `records`: Vector of matching records as JSON values
    /// - `total`: Total number of matching records
    /// - `page`: Current page number
    /// - `page_size`: Number of records per page
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `metadata_table.parquet` file is missing
    /// - DuckDB query preparation or execution fails
    /// - JSON serialization of results fails
    ///
    /// # Example
    /// ```rust
    /// let result = dataset.search(
    ///     &Some(ComposeQuery::QueryItem(...)),
    ///     Some(1),
    ///     Some(20),
    ///     Some("age DESC")
    /// )?;
    /// ```
    ///
    /// # Note
    ///
    /// This method uses `read_parquet(?)` via Duckdb's virtual table capabilities.
    /// Ensure your Duckdb setup supports this feature.
    pub fn search(
        &self,
        query: &Option<ComposeQuery>,
        page: Option<u64>,
        page_size: Option<u64>,
        order_by: Option<&str>,
    ) -> Result<DatasetDataResponse, Error> {
        let parquet_path = self.path.join("metadata_table.parquet");
        if !parquet_path.exists() {
            return Err(anyhow::anyhow!(
                "Dataset parquet file not found at {:?}",
                parquet_path
            ));
        }

        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE metadata AS SELECT * FROM read_parquet(?)",
            params![parquet_path.to_str().unwrap()],
        )?;

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

        let pagination_str = format!("LIMIT {} OFFSET {}", limit, offset);

        let sql = format!(
            "SELECT * FROM metadata WHERE {} {} {}",
            query_str, order_by_str, pagination_str
        );

        info!("Query SQL: {}", sql);

        let mut stmt = conn.prepare("PRAGMA table_info(metadata);")?;
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(Result::ok)
            .collect();

        info!("Table Columns: {:?}", columns);
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], move |row| {
            let record = row_to_json(row, &columns)?;
            Ok(record)
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }

        let count_sql = format!("SELECT COUNT(*) FROM metadata WHERE {}", query_str);
        let count: i64 = conn.query_row(&count_sql, [], |row| row.get(0))?;

        Ok(DatasetDataResponse {
            records,
            total: count as usize,
            page: page as usize,
            page_size: page_size as usize,
        })
    }

    /// Search engine for the dataset.
    pub fn search_with_query_plan(
        &self,
        query_plan: &QueryPlan,
    ) -> Result<DatasetDataResponse, Error> {
        // TODO: Only support one table for now, how to handle joins to support multiple tables?
        let conn = if query_plan.table == self.metadata_table.table_name {
            self.metadata_table.get_conn()?
        } else {
            let conn = match self.datafile_tables.get(&query_plan.table) {
                Some(Some(DataFileTable::MAF(maf_table))) => maf_table.get_conn()?,
                Some(Some(DataFileTable::MRNAExpr(mrna_expr_table))) => {
                    mrna_expr_table.get_conn()?
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "Datafile table not found: {}",
                        query_plan.table
                    ))
                }
            };

            conn
        };

        let sql = query_plan.to_sql()?;

        let mut stmt = conn.prepare(&format!("PRAGMA table_info({});", query_plan.table))?;
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(Result::ok)
            .collect();

        info!("Table Columns: {:?}", columns);
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], move |row| {
            let record = row_to_json(row, &columns)?;
            Ok(record)
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }

        let mut query_plan_clone = query_plan.clone();
        query_plan_clone.selects = vec![SelectExpr::AggFunc {
            func: "count".to_string(),
            field: "*".to_string(),
            alias: None,
        }];

        let count_sql = query_plan_clone.to_sql()?;
        let count: i64 = conn.query_row(&count_sql, [], |row| row.get(0))?;
        let page_size = query_plan.limit.unwrap_or(10).max(1) as usize; // 避免除以 0
        let offset = query_plan.offset.unwrap_or(0).max(0) as usize;
        let page = (offset / page_size) + 1;

        Ok(DatasetDataResponse {
            records,
            total: count as usize,
            page,
            page_size,
        })
    }

    /// Search engine for the dataset.
    // pub fn search(&self, query_plan: &QueryPlan) -> Result<DatasetDataResponse, Error> {
    //     let parquet_path = self.path.join("metadata_table.parquet");
    //     if !parquet_path.exists() {
    //         return Err(anyhow::anyhow!(
    //             "Dataset parquet file not found at {:?}",
    //             parquet_path
    //         ));
    //     }

    //     let conn = Connection::open_in_memory()?;
    //     conn.execute(
    //         "CREATE TABLE metadata_table AS SELECT * FROM read_parquet(?)",
    //         params![parquet_path.to_str().unwrap()],
    //     )?;

    //     let sql_with_params = query_plan.to_sql()?;

    //     info!("Query SQL: {}", sql_with_params.sql);

    //     let mut stmt = conn.prepare("PRAGMA table_info(metadata);")?;
    //     let columns: Vec<String> = stmt
    //         .query_map([], |row| row.get::<_, String>(1))?
    //         .filter_map(Result::ok)
    //         .collect();

    //     info!("Table Columns: {:?}", columns);
    //     let mut stmt = conn.prepare(&sql_with_params.sql)?;
    //     let rows = stmt.query_map(sql_with_params.params, move |row| {
    //         let record = row_to_json(row, &columns)?;
    //         Ok(record)
    //     })?;

    //     let mut records = Vec::new();
    //     for row in rows {
    //         records.push(row?);
    //     }

    //     let count_sql = format!("SELECT COUNT(*) FROM metadata WHERE {}", query_str);
    //     let count: i64 = conn.query_row(&count_sql, [], |row| row.get(0))?;

    //     Ok(DatasetDataResponse {
    //         records,
    //         total: count as usize,
    //         page: page as usize,
    //         page_size: page_size as usize,
    //     })
    // }

    /// Get the data dictionary for this dataset from the cache.
    ///
    /// This method retrieves the cached data dictionary for the dataset. The cache must be
    /// initialized using `init_cache()` before calling this method.
    ///
    /// # Returns
    ///
    /// Returns `Ok(DataDictionary)` containing the dataset's field definitions, or an `Err(Error)`
    /// if the dictionary is not found in the cache.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - The data dictionary is not found in the cache
    /// - The cache has not been initialized
    ///
    /// # Example
    /// ```rust
    /// let dictionary = dataset.get_data_dictionary()?;
    /// for field in dictionary.fields {
    ///     println!("{}: {}", field.key, field.data_type);
    /// }
    /// ```
    pub fn get_data_dictionary(&self) -> Result<DataDictionary, Error> {
        let data_dictionary_cache = DATA_DICTIONARY_CACHE.lock().unwrap();
        let data_dictionary_by_key = data_dictionary_cache.get(&self.metadata.key);

        if data_dictionary_by_key.is_none() {
            return Err(anyhow::anyhow!(
                "Data dictionary not found: {}, it may not be cached or does not exist.",
                self.metadata.key
            ));
        }

        Ok(data_dictionary_by_key
            .unwrap()
            .get(&self.metadata.version)
            .unwrap()
            .clone())
    }

    /// Get the license information for this dataset.
    ///
    /// This method reads the `LICENSE.md` file from the dataset directory.
    ///
    /// # Returns
    ///
    /// Returns `Ok(String)` containing the license text, or an `Err(Error)` if the license file
    /// cannot be read.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - The `LICENSE.md` file does not exist
    /// - The file cannot be read
    ///
    /// # Example
    /// ```rust
    /// let license = dataset.get_license()?;
    /// println!("License: {}", license);
    /// ```
    pub fn get_license(&self) -> Result<String, Error> {
        let license_path = self.path.join("LICENSE.md");
        let license = match fs::read_to_string(&license_path) {
            Ok(license) => license,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to read license file: {}", e));
            }
        };
        Ok(license)
    }

    /// Get the README for this dataset.
    ///
    /// This method reads the `README.md` file from the dataset directory.
    ///
    /// # Returns
    ///
    /// Returns `Ok(String)` containing the README text, or an `Err(Error)` if the README file cannot be read.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - The `README.md` file does not exist
    /// - The file cannot be read
    ///
    /// # Example
    /// ```rust
    /// let readme = dataset.get_readme()?;
    /// println!("README: {}", readme);
    /// ```
    pub fn get_readme(&self) -> Result<String, Error> {
        let readme_path = self.path.join("README.md");
        let readme = match fs::read_to_string(&readme_path) {
            Ok(readme) => readme,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to read README file: {}", e));
            }
        };
        Ok(readme)
    }

    /// Get the datafiles for this dataset from the cache.
    ///
    /// This method retrieves the cached datafiles for the dataset. The cache must be
    /// initialized using `init_cache()` before calling this method.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<File>)` containing the dataset's datafiles, or an `Err(Error)` if the
    /// datafiles are not found in the cache.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - The datafiles are not found in the cache
    /// - The cache has not been initialized
    ///
    /// # Example
    /// ```rust
    /// let datafiles = dataset.get_datafiles()?;
    /// for file in datafiles {
    ///     println!("File: {}", file.path);
    /// }
    /// ```
    pub fn get_datafiles(&self) -> Result<Vec<File>, Error> {
        let datafiles_cache = DATAFILE_CACHE.lock().unwrap();
        let datafiles_by_key = datafiles_cache.get(&self.metadata.key);

        if datafiles_by_key.is_none() {
            return Err(anyhow::anyhow!(
                "Datafiles not found: {}, it may not be cached or does not exist.",
                self.metadata.key
            ));
        }

        Ok(datafiles_by_key
            .unwrap()
            .get(&self.metadata.version)
            .unwrap()
            .clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_logger;
    use log::LevelFilter;

    #[test]
    fn test_validate_example_dataset() {
        let _ = init_logger("dataset_tests", LevelFilter::Debug);

        let path = PathBuf::from("examples/datasets");

        Datasets::index(&path, true).expect("Failed to index example datasets");
        Datasets::validate(&path).expect("Failed to validate example datasets");

        init_cache(&path).expect("Failed to init cache");
    }

    #[test]
    fn test_get_data_dictionary() {
        init_cache(&PathBuf::from("examples/datasets")).expect("Failed to init cache");

        let ds =
            Datasets::get("acbc_mskcc_2015").expect("Missing expected dataset 'acbc_mskcc_2015'");
        assert!(ds.len() > 0);
        let dict = ds[0]
            .get_data_dictionary()
            .expect("Failed to load data dictionary");
        assert!(dict.fields.len() > 0);
    }

    #[test]
    fn test_search_datasets() {
        let path = PathBuf::from("examples/datasets");
        let result = Datasets::search(&path, &None, None, None, None).expect("Search failed");
        assert!(result.total > 0);
    }

    #[test]
    fn test_search_example_dataset() {
        init_cache(&PathBuf::from("examples/datasets")).expect("Failed to init cache");

        let ds =
            Datasets::get("acbc_mskcc_2015").expect("Missing expected dataset 'acbc_mskcc_2015'");
        assert!(ds.len() > 0);

        let result: DatasetDataResponse = ds[0]
            .search(&None, Some(1), Some(5), None)
            .expect("Search failed");
        assert!(result.total > 0);
        assert!(result.records.len() > 0);
        assert_eq!(result.page, 1);
    }
}
