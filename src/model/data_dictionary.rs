use anyhow::Error;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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

#[derive(Debug, Clone, Object, Deserialize, Serialize)]
pub struct DataDictionary {
    pub fields: Vec<DataDictionaryField>,
}

impl DataDictionary {
    pub fn from_file(data_dictionary_path: &PathBuf) -> Result<Self, anyhow::Error> {
        let data_dictionary = match fs::read_to_string(&data_dictionary_path) {
            Ok(data_dictionary) => data_dictionary,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to read data dictionary file: {}",
                    e
                ));
            }
        };
        let data_dictionary: Vec<DataDictionaryField> = match serde_json::from_str(&data_dictionary)
        {
            Ok(data_dictionary) => data_dictionary,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to parse data dictionary file: {}",
                    e
                ));
            }
        };

        Ok(DataDictionary {
            fields: data_dictionary,
        })
    }

    pub fn load_metadata_dictionary(base_path: &PathBuf) -> Result<Self, Error> {
        let path = base_path.join("metadata_dictionary.json");
        let data_dictionary = DataDictionary::from_file(&path)?;
        Ok(data_dictionary)
    }
}
