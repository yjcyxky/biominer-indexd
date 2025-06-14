use crate::model::data_dictionary::DataDictionary;
use anyhow::{anyhow, Error};
use lazy_static::lazy_static;
use std::path::PathBuf;
use std::sync::Mutex;

lazy_static! {
    pub static ref DATA_FILE_TABLES: Mutex<Vec<&'static str>> = Mutex::new(vec!["maf", "mrna_expr"]);
}

#[derive(Debug, Clone)]
pub struct MetadataTable {
    pub table_name: &'static str,
    pub data_dictionary: DataDictionary,
    pub path: PathBuf,
}

impl MetadataTable {
    pub fn new(base_path: &PathBuf) -> Result<Self, Error> {
        let path = base_path.join("metadata_table.parquet");
        let data_dictionary = DataDictionary::from_file(&path.join("metadata_dictionary.json"))?;

        Ok(Self {
            table_name: "metadata",
            data_dictionary,
            path,
        })
    }

    fn get_table_name(&self) -> &str {
        &self.table_name
    }

    fn get_data_dictionary(&self) -> &DataDictionary {
        &self.data_dictionary
    }

    fn get_path(&self) -> PathBuf {
        self.path.clone()
    }
}

#[derive(Debug, Clone)]
pub struct MAFTable {
    pub table_name: &'static str,
    pub data_dictionary: DataDictionary,
    pub path: PathBuf,
}

impl MAFTable {
    pub fn new(base_path: &PathBuf) -> Result<Self, Error> {
        let path = base_path.join("maf.parquet");
        let data_dictionary = DataDictionary::from_file(&path.join("maf_dictionary.json"))?;

        Ok(Self {
            table_name: "maf",
            data_dictionary,
            path,
        })
    }

    fn get_table_name(&self) -> &str {
        &self.table_name
    }

    fn get_data_dictionary(&self) -> &DataDictionary {
        &self.data_dictionary
    }

    fn get_path(&self) -> PathBuf {
        self.path.clone()
    }
}

#[derive(Debug, Clone)]
pub struct MRNAExprTable {
    pub table_name: &'static str,
    pub data_dictionary: DataDictionary,
    pub path: PathBuf,
}

impl MRNAExprTable {
    pub fn new(base_path: &PathBuf) -> Result<Self, Error> {
        let path = base_path.join("mrna_expr_table.parquet");
        let data_dictionary = DataDictionary::from_file(&path.join("mrna_expr_dictionary.json"))?;

        Ok(Self {
            table_name: "mrna_expr",
            data_dictionary,
            path,
        })
    }

    fn get_table_name(&self) -> &str {
        &self.table_name
    }

    fn get_data_dictionary(&self) -> &DataDictionary {
        &self.data_dictionary
    }

    fn get_path(&self) -> PathBuf {
        self.path.clone()
    }
}

#[derive(Debug, Clone)]
pub enum DataFileTable {
    MAF(MAFTable),
    MRNAExpr(MRNAExprTable),
}

impl DataFileTable {
    pub fn get_table_name(&self) -> &str {
        match self {
            DataFileTable::MAF(table) => table.get_table_name(),
            DataFileTable::MRNAExpr(table) => table.get_table_name(),
        }
    }
}
