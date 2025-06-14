use anyhow::Error;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone, Object)]
pub struct DatasetMetadata {
    pub key: String,
    pub name: String,
    pub description: String,
    pub citation: String,
    pub pmid: String,
    pub groups: Vec<String>,
    pub tags: Vec<String>,
    pub total: usize,
    pub is_filebased: bool,
}

impl DatasetMetadata {
    pub fn from_file(base_path: &PathBuf) -> Result<Self, Error> {
        let path = base_path.join("dataset.json");
        let content = fs::read_to_string(path)?;
        let metadata: DatasetMetadata = serde_json::from_str(&content)?;
        Ok(metadata)
    }

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
            total: value["total"].as_u64().unwrap() as usize,
            is_filebased: value["is_filebased"].as_bool().unwrap(),
        }
    }
}
