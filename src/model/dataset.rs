// =====================================================================================
// Dataset Manager (Rust)
//
// This module provides a lightweight dataset management system for structured datasets
// stored in a file-based directory format. Each dataset resides in a subdirectory under
// a common `data_dir` and includes metadata and data in standardized formats.
//
// ## Structure
// - `index.json`: A JSON array of datasets with fields: id, name, description, citation,
//   pmid, groups, tags, num_of_samples
// - Each dataset folder contains:
//   - `data_dictionary.json`: defines schema for the dataset
//   - `data.parquet`: tabular data file, column names correspond to keys in dictionary
//   - `dataset.json`: study metadata
//
// ## Features
// - Loads and validates dataset metadata and structure
// - Validates that dictionary keys are lowercase letters/digits/underscores and start with a letter
// - Validates that each field has a supported type: STRING, NUMBER, BOOLEAN
// - Supports SQL search over `index.json` using DuckDB
// - Supports SQL queries over individual dataset Parquet files
// - Provides a typed interface to load and inspect a dataset's dictionary
//
// ## Requirements
// - DuckDB
// - Parquet + JSON files per the structure above
//
// ## Usage Example
// ```rust
// let datasets = Datasets::load(Path::new("data_dir"))?;
// datasets.validate()?;
// datasets.search_index("SELECT * FROM datasets WHERE tags LIKE '%rna%'")?;
//
// let dataset = &datasets.entries[0];
// dataset.query_parquet("SELECT * FROM data WHERE age > 30")?;
// let dict = dataset.load_data_dictionary()?;
// ```
// =====================================================================================

use super::duckdb_util::row_to_json;
use crate::query_builder::sql_builder::ComposeQuery;
use anyhow::{bail, Error, Result};
use duckdb::{params, Connection};
use log::info;
use poem_openapi::Object;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, path::PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone, Object)]
pub struct DataDictionaryField {
    pub key: String,
    pub name: String,
    pub data_type: String,
    pub description: String,
    pub notes: String,
    pub allowed_values: serde_json::Value, // It might be a list of strings, numbers, or booleans
    pub order: usize,
}

#[derive(Debug, Clone, Object)]
pub struct DataDictionary {
    pub fields: Vec<DataDictionaryField>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Object)]
pub struct DatasetMetadata {
    pub key: String,
    pub name: String,
    pub description: String,
    pub citation: String,
    pub pmid: String,
    pub groups: Vec<String>,
    pub tags: Vec<String>,
    pub num_of_samples: usize,
}

impl DatasetMetadata {
    pub fn from_value(value: serde_json::Value) -> Self {
        Self {
            key: value["key"].as_str().unwrap().to_string(),
            name: value["name"].as_str().unwrap().to_string(),
            description: value["description"].as_str().unwrap().to_string(),
            citation: value["citation"].as_str().unwrap().to_string(),
            pmid: value["pmid"].as_str().unwrap().to_string(),
            groups: value["groups"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect(),
            tags: value["tags"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect(),
            num_of_samples: value["num_of_samples"].as_u64().unwrap() as usize,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dataset {
    pub metadata: DatasetMetadata,
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
        let content = fs::read_to_string(&index_path)?;
        let index_entries: Vec<DatasetMetadata> = serde_json::from_str(&content)?;

        let entries = index_entries
            .into_iter()
            .map(|entry| {
                let path = base_path.join(&entry.key);
                Dataset {
                    metadata: entry,
                    path,
                }
            })
            .collect();

        Ok(Self {
            records: entries,
            base_path: base_path.to_path_buf(),
        })
    }

    /// Validates the dataset directory structure and contents at the specified base path.
    ///
    /// This function performs validation on a dataset collection rooted at `base_path`. It expects:
    /// - A file named `index.json` in the `base_path` directory.
    /// - A subdirectory for each dataset entry listed in the index.
    /// - Each dataset directory to contain a valid data dictionary and a `data.parquet` file.
    ///
    /// The validation steps include:
    /// 1. Checking that `base_path` exists and is a directory.
    /// 2. Ensuring the `index.json` file exists and can be parsed.
    /// 3. Verifying each dataset directory exists.
    /// 4. Validating that each field in the dataset's data dictionary:
    ///    - Has a key matching the regex: `^[a-z][a-z0-9_]*$`
    ///    - Uses a valid data type: `"STRING"`, `"NUMBER"`, or `"BOOLEAN"`
    /// 5. Ensuring that a `data.parquet` file exists in each dataset directory.
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
    /// - The `data.parquet` file is missing.
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

        let index_path = base_path.join("index.json");
        let content = fs::read_to_string(&index_path)?;
        let index_entries: Vec<DatasetMetadata> = match serde_json::from_str(&content) {
            Ok(entries) => entries,
            Err(e) => {
                bail!("Failed to parse index file: {}", e);
            }
        };

        let mut records = Vec::new();

        for entry in index_entries {
            let dataset = Dataset::load(&base_path.join(&entry.key))?;
            records.push(dataset);
        }

        let key_re = Regex::new(r"^[a-z][a-z0-9_]*$").unwrap();

        for dataset in &records {
            if !dataset.path.is_dir() {
                bail!("Dataset directory {:?} does not exist", dataset.path);
            }

            let dict = dataset.load_data_dictionary()?;
            for field in &dict.fields {
                if !key_re.is_match(&field.key) {
                    bail!(
                        "Invalid key '{}' in dataset '{}'.",
                        field.key,
                        dataset.metadata.key
                    );
                }

                if !matches!(field.data_type.as_str(), "STRING" | "NUMBER" | "BOOLEAN") {
                    bail!(
                        "Invalid data_type '{}' in dataset '{}', key '{}'.",
                        field.data_type,
                        dataset.metadata.key,
                        field.key
                    );
                }
            }

            let parquet_path = dataset.path.join("data.parquet");
            if !parquet_path.exists() {
                bail!("Missing data.parquet in {:?}", dataset.path);
            }
        }

        println!("✅ All datasets validated successfully.");
        Ok(())
    }

    /// Searches the dataset index using an optional query with pagination and sorting.
    ///
    /// This method allows flexible querying over the dataset index (`index.json`) using an in-memory
    /// Duckdb engine with `read_json_auto`. It supports optional filtering, sorting, and pagination.
    ///
    /// # Arguments
    ///
    /// * `query` - An optional query (`ComposeQuery`) to filter datasets. If `None`, all datasets are returned.
    /// * `page` - An optional page number (1-based). Defaults to 1 if not provided.
    /// * `page_size` - An optional page size. Defaults to 10 if not provided.
    /// * `order_by` - An optional SQL `ORDER BY` clause (e.g., `"name ASC"`, `"num_of_samples DESC"`).
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<DatasetMetadata>` if the search is successful, or an `Error` if the
    /// query fails or if the index cannot be read.
    ///
    /// # Behavior
    ///
    /// - Loads `index.json` from the base path.
    /// - Uses Duckdb to parse and query the JSON data as a table.
    /// - Applies any provided query conditions, ordering, and pagination.
    /// - If no query is provided, defaults to `WHERE 1=1`.
    /// - If no pagination is provided, returns the first 10 records.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `index.json` file cannot be read.
    /// - Duckdb query preparation or execution fails.
    /// - JSON deserialization from database fields fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// let results = dataset_collection.search(
    ///     &Some(ComposeQuery::QueryItem(...)),
    ///     Some(2),
    ///     Some(5),
    ///     Some("name ASC"),
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
            "SELECT key, name, description, citation, pmid, json(groups) AS groups, json(tags) AS tags, num_of_samples FROM datasets WHERE {} {} {}",
            query_str, order_by_str, pagination_str
        );

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            let record = row_to_json(
                row,
                &[
                    "key".to_string(),
                    "name".to_string(),
                    "description".to_string(),
                    "citation".to_string(),
                    "pmid".to_string(),
                    "groups".to_string(),
                    "tags".to_string(),
                    "num_of_samples".to_string(),
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
    /// let dataset = datasets.get("genomics_data")?;
    /// ```
    pub fn get(&self, key: &str) -> Result<Dataset, Error> {
        let dataset = self.records.iter().find(|d| d.metadata.key == key);
        if dataset.is_none() {
            bail!("Dataset not found: {}", key);
        }
        Ok(dataset.unwrap().clone())
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
    pub fn index(base_path: &Path, save_to_file: bool) -> Result<Self, Error> {
        let mut datasets = Vec::new();
        // List all subdirectories in base_path
        let entries = fs::read_dir(base_path)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let dataset = Dataset::load(&path)?;
                datasets.push(dataset);
            }
        }

        if save_to_file {
            let index_path = base_path.join("index.json");
            let index_entries: Vec<DatasetMetadata> =
                datasets.iter().map(|d| d.metadata.clone()).collect();
            fs::write(index_path, serde_json::to_string(&index_entries)?)?;
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
    pub fn load(dataset_path: &Path) -> Result<Self, Error> {
        let metadata_path = dataset_path.join("dataset.json");
        let content = fs::read_to_string(&metadata_path)?;
        let metadata: DatasetMetadata = serde_json::from_str(&content)?;
        Ok(Self {
            metadata,
            path: dataset_path.to_path_buf(),
        })
    }

    /// Searches records within the dataset's Parquet file using an optional SQL-like query,
    /// pagination, and sorting.
    ///
    /// This function reads the dataset's `data.parquet` file into an in-memory SQLite table
    /// using the `read_parquet` virtual table. It then performs a query over the data
    /// with optional filtering (`query`), ordering (`order_by`), and pagination (`page`, `page_size`).
    ///
    /// # Arguments
    ///
    /// * `query` - Optional `ComposeQuery` to filter records. If `None`, all records are returned.
    /// * `page` - Optional page number (1-based). Defaults to `1` if not specified.
    /// * `page_size` - Optional number of records per page. Defaults to `10` if not specified.
    /// * `order_by` - Optional SQL `ORDER BY` clause string (e.g., `"age DESC"`, `"name ASC"`).
    ///
    /// # Returns
    ///
    /// Returns a `serde_json::Value` representing the query results, structured as:
    /// ```json
    /// {
    ///   "records": [...],
    ///   "page_size": 10,
    ///   "page": 1,
    ///   "total": 42
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `data.parquet` file is missing.
    /// - The Parquet file cannot be read into SQLite.
    /// - The SQL query fails to execute or parse.
    /// - JSON serialization of a row fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// let result = dataset.search(
    ///     &Some(ComposeQuery::QueryItem(...)),
    ///     Some(1),
    ///     Some(20),
    ///     Some("age DESC"),
    /// )?;
    ///
    /// println!("Records on page 1: {}", result["records"]);
    /// println!("Total count: {}", result["total"]);
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
        let parquet_path = self.path.join("data.parquet");
        if !parquet_path.exists() {
            bail!("Dataset parquet file not found at {:?}", parquet_path);
        }

        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE data AS SELECT * FROM read_parquet(?)",
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
            "SELECT * FROM data WHERE {} {} {}",
            query_str, order_by_str, pagination_str
        );

        info!("Query SQL: {}", sql);

        let mut stmt = conn.prepare("PRAGMA table_info(data);")?;
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

        let count_sql = format!("SELECT COUNT(*) FROM data WHERE {}", query_str);
        let count: i64 = conn.query_row(&count_sql, [], |row| row.get(0))?;

        Ok(DatasetDataResponse {
            records,
            total: count as usize,
            page: page as usize,
            page_size: page_size as usize,
        })
    }

    /// Loads the data dictionary for this dataset.
    ///
    /// This method reads the `data_dictionary.json` file located in the dataset's directory,
    /// parses it into a list of `DataDictionaryField` entries, and returns a `DataDictionary` object.
    ///
    /// # Returns
    ///
    /// Returns `Ok(DataDictionary)` if the file is successfully read and parsed, or an `Err(Error)`
    /// if the file is missing, unreadable, or contains invalid JSON.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - The `data_dictionary.json` file does not exist in the dataset directory.
    /// - The file cannot be read (e.g., due to permissions).
    /// - The JSON structure is invalid or does not match `Vec<DataDictionaryField>`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let dictionary = dataset.load_data_dictionary()?;
    /// for field in dictionary.fields {
    ///     println!("{}: {}", field.key, field.data_type);
    /// }
    /// ```
    pub fn load_data_dictionary(self: &Self) -> Result<DataDictionary, Error> {
        let dict_path = self.path.join("data_dictionary.json");
        let content = fs::read_to_string(&dict_path)?;
        let fields: Vec<DataDictionaryField> = serde_json::from_str(&content)?;
        Ok(DataDictionary { fields })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::path::Path;

    #[test]
    fn test_validate_example_dataset() {
        let path = PathBuf::from("examples/datasets");

        Datasets::index(&path, true).expect("Failed to index example datasets");
        Datasets::validate(&path).expect("Failed to validate example datasets");

        let datasets = Datasets::load(&path).expect("Failed to load example datasets");
        assert!(datasets.records.len() > 0);

        let ds = datasets
            .get("acbc_mskcc_2015")
            .expect("Missing expected dataset 'acbc_mskcc_2015'");
        assert_eq!(ds.metadata.key, "acbc_mskcc_2015");
    }

    #[test]
    fn test_load_data_dictionary() {
        let path = PathBuf::from("examples/datasets");

        let datasets = Datasets::load(&path).expect("Failed to load example datasets");
        let ds = datasets
            .get("acbc_mskcc_2015")
            .expect("Missing expected dataset 'acbc_mskcc_2015'");
        let dict = ds
            .load_data_dictionary()
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
        let path = PathBuf::from("examples/datasets");

        Datasets::index(&path, true).expect("Failed to index example datasets");
        Datasets::validate(&path).expect("Failed to validate example datasets");

        let datasets = Datasets::load(&path).expect("Failed to load example datasets");
        let ds = datasets
            .get("acbc_mskcc_2015")
            .expect("Missing expected dataset 'acbc_mskcc_2015'");

        let result: DatasetDataResponse = ds
            .search(&None, Some(1), Some(5), None)
            .expect("Search failed");
        assert!(result.total > 0);
        assert!(result.records.len() > 0);
        assert_eq!(result.page, 1);
    }
}
