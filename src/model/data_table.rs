use crate::model::data_dictionary::DataDictionary;
use anyhow::Error;
use duckdb::{params, Connection};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileGroup {
    pub prefix: String,
    pub parquet: PathBuf,
    pub dictionary: PathBuf,
    pub metadata: PathBuf,
}

impl FileGroup {
    /// Find all file groups in the given directory
    ///
    /// # Returns
    ///
    /// - A vector of `FileGroup` objects
    /// 
    /// # Example
    ///
    /// ```rust
    /// let file_groups = FileGroup::find_file_groups("examples/datasets/acbc_mskcc_2015/v0.0.1/datafiles");
    /// assert!(!file_groups.is_empty());
    /// ```
    pub fn find_file_groups<P: AsRef<Path>>(dir: P) -> Vec<FileGroup> {
        let mut candidates: HashMap<String, HashSet<String>> = HashMap::new();
        let mut files: HashMap<(String, String), PathBuf> = HashMap::new();

        for entry in fs::read_dir(&dir).expect("Cannot read directory") {
            let entry = entry.expect("Invalid DirEntry");
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if let Some((prefix, suffix)) = Self::match_file(&file_name) {
                    candidates
                        .entry(prefix.to_string())
                        .or_default()
                        .insert(suffix.to_string());
                    files.insert((prefix.to_string(), suffix.to_string()), path);
                }
            }
        }

        let mut result = Vec::new();
        for (prefix, suffixes) in candidates {
            if ["parquet", "dictionary", "metadata"]
                .iter()
                .all(|k| suffixes.contains(*k))
            {
                result.push(FileGroup {
                    prefix: prefix.clone(),
                    parquet: files[&(prefix.clone(), "parquet".into())].clone(),
                    dictionary: files[&(prefix.clone(), "dictionary".into())].clone(),
                    metadata: files[&(prefix.clone(), "metadata".into())].clone(),
                });
            }
        }

        result
    }

    /// Match the file name to the prefix and type (parquet/dictionary/metadata)
    ///
    /// # Returns
    ///
    /// - If the file name matches the pattern, return the prefix and type
    /// - If the file name does not match the pattern, return None
    /// 
    /// # Example
    ///
    /// ```rust
    /// let file_name = "maf.parquet";
    /// let (prefix, suffix) = FileGroup::match_file(file_name);
    /// assert_eq!(prefix, "maf");
    /// assert_eq!(suffix, "parquet");
    /// ```
    fn match_file(file_name: &str) -> Option<(&str, &str)> {
        let patterns = [
            (".parquet", "parquet"),
            ("_dictionary.json", "dictionary"),
            ("_metadata.json", "metadata"),
        ];
        for (suffix, label) in &patterns {
            if let Some(prefix) = file_name.strip_suffix(suffix) {
                return Some((prefix, label));
            }
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct DataFileTableMetadata {
    pub title: String,
    pub description: String,
    pub id_column_name: String,
}

impl DataFileTableMetadata {
    pub fn from_file(filepath: &PathBuf) -> Result<Self, Error> {
        if !filepath.exists() {
            return Err(Error::msg(format!("Data file table metadata file ({}) not found", &filepath.display())));
        }

        let metadata = fs::read_to_string(&filepath)?;
        let metadata: DataFileTableMetadata = match serde_json::from_str(&metadata) {
            Ok(metadata) => metadata,
            Err(e) => {
                return Err(Error::msg(format!("Failed to parse data file table metadata file ({}): {}", &filepath.display(), e)));
            }
        };

        Ok(metadata)
    }
}

pub trait DataTable: Sized {
    fn set_title(&mut self, title: &str);
    fn set_description(&mut self, description: &str);
    fn get_title(&self) -> &str;
    fn get_description(&self) -> &str;
    fn set_table_name(&mut self, table_name: &'static str);
    fn get_table_name(&self) -> &str;
    fn get_data_dictionary(&self) -> &DataDictionary;
    fn get_path(&self) -> PathBuf;
    fn get_conn(&self) -> Result<Connection, Error>;
    fn get_id_column_name(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct MetadataTable {
    pub title: String,
    pub description: String,
    pub table_name: String,
    pub id_column_name: String,
    pub data_dictionary: DataDictionary,
    pub path: String,
}

impl MetadataTable {
    pub fn new(base_path: &PathBuf) -> Result<Self, Error> {
        let path = base_path.join("metadata_table.parquet");
        let data_dictionary = DataDictionary::load_metadata_dictionary(base_path)?;

        Ok(Self {
            title: "Clinical Data".to_string(),
            id_column_name: "patient_id".to_string(),
            description: "Information about the patients and samples".to_string(),
            table_name: "metadata_table".to_string(),
            data_dictionary,
            // TODO: It might be a risk point to expose the full path to the client.
            path: path.to_str().unwrap().to_string(),
        })
    }
}

impl DataTable for MetadataTable {
    fn set_table_name(&mut self, table_name: &'static str) {
        // Don't allow changing the table name
    }

    fn set_title(&mut self, title: &str) {
        // Don't allow changing the title
    }

    fn set_description(&mut self, description: &str) {
        // Don't allow changing the description
    }

    fn get_description(&self) -> &str {
        &self.description
    }

    fn get_title(&self) -> &str {
        &self.title
    }

    fn get_table_name(&self) -> &str {
        &self.table_name
    }

    fn get_data_dictionary(&self) -> &DataDictionary {
        &self.data_dictionary
    }

    fn get_path(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }

    fn get_conn(&self) -> Result<Connection, Error> {
        let conn = Connection::open_in_memory()?;
        conn.execute(
            &format!(
                "CREATE TABLE {} AS SELECT * FROM read_parquet(?)",
                self.table_name
            ),
            params![&self.path],
        )?;

        Ok(conn)
    }

    fn get_id_column_name(&self) -> &str {
        &self.id_column_name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct DataFileTable {
    pub title: String,
    pub description: String,
    pub id_column_name: String,
    pub table_name: String,
    pub data_dictionary: DataDictionary,
    pub path: String,
}

impl DataFileTable {
    pub fn new(base_path: &PathBuf, table_name: &str) -> Result<Self, Error> {
        let data_dictionary =
            DataDictionary::from_file(&base_path.join(format!("{}_dictionary.json", table_name)))?;

        let metadata = DataFileTableMetadata::from_file(
            &base_path.join(format!("{}_metadata.json", table_name)),
        )?;

        Ok(Self {
            title: metadata.title,
            description: metadata.description,
            id_column_name: metadata.id_column_name,
            table_name: table_name.to_string(),
            data_dictionary,
            // TODO: It might be a risk point to expose the full path to the client.
            path: base_path.join(format!("{}.parquet", table_name)).to_str().unwrap().to_string(),
        })
    }
}

impl DataTable for DataFileTable {
    fn set_table_name(&mut self, table_name: &'static str) {
        self.table_name = table_name.to_string();
    }

    fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    fn set_description(&mut self, description: &str) {
        self.description = description.to_string();
    }

    fn get_title(&self) -> &str {
        &self.title
    }

    fn get_description(&self) -> &str {
        &self.description
    }

    fn get_table_name(&self) -> &str {
        &self.table_name
    }

    fn get_data_dictionary(&self) -> &DataDictionary {
        &self.data_dictionary
    }

    fn get_path(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }

    fn get_conn(&self) -> Result<Connection, Error> {
        let conn = Connection::open_in_memory()?;
        conn.execute(
            &format!(
                "CREATE TABLE {} AS SELECT * FROM read_parquet(?)",
                self.table_name
            ),
            params![&self.path],
        )?;

        Ok(conn)
    }

    fn get_id_column_name(&self) -> &str {
        &self.id_column_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn example_base_path() -> PathBuf {
        PathBuf::from("examples/datasets/acbc_mskcc_2015/v0.0.1")
    }

    #[test]
    fn test_metadata_table_new_and_methods() {
        let base_path = example_base_path();
        let table = MetadataTable::new(&base_path).expect("Failed to create MetadataTable");
        assert_eq!(table.get_title(), "Clinical Data");
        assert_eq!(
            table.get_description(),
            "Information about the patients and samples"
        );
        assert_eq!(table.get_table_name(), "metadata_table");
        assert!(table.get_path().ends_with("metadata_table.parquet"));
        let dict = table.get_data_dictionary();
        assert!(!dict.fields.is_empty());
    }

    #[test]
    fn test_datafile_table_new_and_methods() {
        let base_path = example_base_path();
        // 该目录下应有maf_dictionary.json和maf.parquet
        let table = DataFileTable::new(&base_path, "maf").expect("Failed to create DataFileTable");
        assert_eq!(table.get_title(), "MAF Table");
        assert_eq!(table.get_description(), "Mutation Annotation Format Table");
        assert_eq!(table.get_table_name(), "maf");
        assert!(table.get_path().ends_with("maf.parquet"));
        let dict = table.get_data_dictionary();
        assert!(!dict.fields.is_empty());
    }

    #[test]
    fn test_metadata_table_get_conn_fail() {
        let base_path = PathBuf::from("/tmp/nonexistent");
        let table = MetadataTable::new(&base_path).expect("Failed to create MetadataTable");
        assert!(table.get_conn().is_err());
    }

    #[test]
    fn test_datafile_table_get_conn_fail() {
        let base_path = PathBuf::from("/tmp/nonexistent");
        let table = DataFileTable::new(&base_path, "maf").expect("Failed to create DataFileTable");
        assert!(table.get_conn().is_err());
    }
}
