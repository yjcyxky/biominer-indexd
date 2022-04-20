pub mod api;
pub mod database;
pub mod util;

use custom_error::custom_error;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use serde_json;

custom_error! {pub ConfigError
  ConfigNotFound{protocol: String} = "config not found: {protocol}",
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct SignResponse {
  pub header: Vec<String>, // ["Content-Type: application/json"] or ["Content-Type: application/x-www-form-urlencoded"]
  pub data: Vec<String>,   // ["username=admin", "password=admin"]
  pub baseurl: String,     // "http://localhost:8080"
  pub method: String,      // "GET" or "POST"
  pub params: Vec<String>, // ["AWSAccessKeyId=AKIAIOSFODNN7EXAMPLE", "Signature=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"]
}

pub trait Sign {
  fn sign(&self, url: &str) -> SignResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
  pub download_url: String,
  pub account_name: String,
  pub member_id: String,
}

impl Sign for NodeConfig {
  fn sign(&self, url: &str) -> SignResponse {
    let url_parts = url.split("/").collect::<Vec<&str>>();
    let data_no = url_parts[url_parts.len() - 1];
    let header = vec!["Content-Type: application/x-www-form-urlencoded".to_string()];
    let data = vec![format!("dataNo={}&memberId={}", data_no, self.member_id)];
    let params = vec![];
    let baseurl = self.download_url.clone();
    let method = "POST".to_string();

    SignResponse {
      header: header,
      data: data,
      baseurl: baseurl,
      method: method,
      params: params,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GSAConfig {}

impl Sign for GSAConfig {
  fn sign(&self, url: &str) -> SignResponse {
    todo!();
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OSSConfig {
  pub access_key: String,
  pub access_secret: String,
  pub endpoint: String,
  pub bucket: String,
}

impl Sign for OSSConfig {
  fn sign(&self, url: &str) -> SignResponse {
    todo!();
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
  pub access_key: String,
  pub access_secret: String,
  pub endpoint: String,
  pub bucket: String,
}

impl Sign for S3Config {
  fn sign(&self, url: &str) -> SignResponse {
    todo!();
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinioConfig {
  pub access_key: String,
  pub access_secret: String,
  pub endpoint: String,
  pub bucket: String,
}

impl Sign for MinioConfig {
  fn sign(&self, url: &str) -> SignResponse {
    todo!();
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoConfig {
  pub node: Option<Vec<NodeConfig>>,
  pub oss: Option<Vec<OSSConfig>>,
  pub s3: Option<Vec<S3Config>>,
  pub minio: Option<Vec<MinioConfig>>,
  pub gsa: Option<Vec<GSAConfig>>,
}

impl RepoConfig {
  pub fn read_config(config_path: &str) -> Result<RepoConfig, std::io::Error> {
    let config_file = std::fs::read_to_string(config_path)?;
    let config: RepoConfig = serde_json::from_str(&config_file)?;
    Ok(config)
  }

  pub fn read_config_data(config_str: &str) -> Result<RepoConfig, serde_json::Error> {
    let config: RepoConfig = serde_json::from_str(config_str)?;
    Ok(config)
  }

  pub fn fetch_config(&self, protocol: &str, identity: &str) -> Option<Box<dyn Sign>> {
    match protocol {
      "node" => {
        if let Some(configs) = &self.node {
          for config in configs {
            if config.account_name == identity {
              return Some(Box::new(config.clone()));
            }
          }
        }
        return None;
      }
      "s3" => {
        if let Some(configs) = &self.s3 {
          for config in configs {
            if config.bucket == identity {
              return Some(Box::new(config.clone()));
            }
          }
        }

        return None;
      }
      "oss" => {
        if let Some(configs) = &self.oss {
          for config in configs {
            if config.bucket == identity {
              return Some(Box::new(config.clone()));
            }
          }
        }

        return None;
      }
      "minio" => {
        if let Some(configs) = &self.minio {
          for config in configs {
            if config.bucket == identity {
              return Some(Box::new(config.clone()));
            }
          }
        }

        return None;
      }
      "gsa" => {
        todo!();
        // return None;
      }
      _ => panic!("unsupported protocol"),
    };
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_read_config_data() {
    let config_str = r#"
      {
        "node": [{
          "download_url": "https://www.biosino.org/download/downloadByMember",
          "account_name": "yjcyxky@163.com",
          "member_id": "P34MVSHVGBDSLOQ6BGSAL4SFEN"
        }]
      }
    "#;
    let config = RepoConfig::read_config_data(config_str).unwrap();
    assert!(config.node.is_some());
    assert!(config.oss.is_none());

    match config.node {
      Some(node) => {
        assert!(node.len() == 1);
        assert!(node[0].account_name == "yjcyxky@163.com");
      }
      None => {}
    }
  }

  #[test]
  fn test_nodeconfig_sign() {
    let url = "node://yjcyxky@163.com/OEP003178/OEX015832/OES135568/OER249692/OED718095";
    let config_str = r#"
      {
        "node": [{
          "download_url": "https://www.biosino.org/download/downloadByMember",
          "account_name": "yjcyxky@163.com",
          "member_id": "P34MVSHVGBDSLOQ6BGSAL4SFEN"
        }]
      }
    "#;
    let config = RepoConfig::read_config_data(config_str).unwrap();
    let node = config.node.unwrap();
    let config = node[0].clone();
    let signed = config.sign(url);
    assert!(signed.header == ["Content-Type: application/x-www-form-urlencoded".to_string()]);
  }

  #[test]
  fn test_fetch_config() {
    let config_str = r#"
      {
        "node": [{
          "download_url": "https://www.biosino.org/download/downloadByMember",
          "member_id": "P34MVSHVGBDSLOQ6BGSAL4SFEN",
          "account_name": "yjcyxky@163.com"
        }]
      }
    "#;
    let url = "node://yjcyxky@163.com/OEP003178/OEX015832/OES135568/OER249692/OED718095";
    let config = RepoConfig::read_config_data(config_str).unwrap();
    let c = config.fetch_config("node", "yjcyxky@163.com").unwrap();
    assert!(c.sign(url).header == ["Content-Type: application/x-www-form-urlencoded".to_string()]);
  }
}
