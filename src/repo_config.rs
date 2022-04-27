use custom_error::custom_error;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use serde_json;

custom_error! {pub ConfigError
  ConfigNotFound{protocol: String} = "config not found: {protocol}",
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Object)]
pub struct SignData {
  pub header: Vec<String>, // ["Content-Type: application/json"] or ["Content-Type: application/x-www-form-urlencoded"]
  pub data: Vec<String>,   // ["username=admin", "password=admin"]
  pub baseurl: String,     // "http://localhost:8080"
  pub method: String,      // "GET" or "POST"
  pub params: Vec<String>, // ["AWSAccessKeyId=AKIAIOSFODNN7EXAMPLE", "Signature=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"]
}

pub trait Sign {
  fn sign(&self, url: &str) -> SignData;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
  pub download_url: String,
  pub account_name: String,
  pub member_id: String,
  pub project_id: String
}

impl Sign for NodeConfig {
  fn sign(&self, url: &str) -> SignData {
    let url_parts = url.split("/").collect::<Vec<&str>>();
    let data_no = url_parts[url_parts.len() - 1];
    let header = vec!["Content-Type: application/x-www-form-urlencoded".to_string()];
    let data = vec![
      format!("dataNo={}", data_no),
      format!("memberId={}", self.member_id),
    ];
    let params = vec![];
    let baseurl = self.download_url.clone();
    let method = "POST".to_string();

    SignData {
      header: header,
      data: data,
      baseurl: baseurl,
      method: method,
      params: params,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GSAConfig {
  pub download_url: String,
  pub account_name: String,
  pub shared_id: String,
  pub project_id: String
}

impl Sign for GSAConfig {
  fn sign(&self, url: &str) -> SignData {
    // gsa://yjcyxky@163.com/HRA0001/HRS335657/HRX427830/HRR593463/HRR592563_f1.fastq.gz
    // ["gsa:", "", "yjcyxky@163.com", "HRR0001", "HRS335657", "HRX427830", "HRR593463", "HRR592563_f1.fastq.gz"]
    let url_parts = url.split("/").collect::<Vec<&str>>();
    let project_id = url_parts[3];
    // TODO: Check account_name?
    // let account_name = url_parts[2];
    let run_id = url_parts[6];
    let filename = url_parts[7];
    let header = vec![];
    let data = vec![];
    let params = vec![];
    let baseurl = std::path::Path::new(&self.download_url)
      .join(&self.shared_id)
      .join(project_id)
      .join(run_id)
      .join(filename)
      .to_str()
      .unwrap()
      .to_string();
    let method = "GET".to_string();

    SignData {
      header: header,
      data: data,
      baseurl: baseurl,
      method: method,
      params: params,
    }
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
  fn sign(&self, url: &str) -> SignData {
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
  fn sign(&self, url: &str) -> SignData {
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
  fn sign(&self, url: &str) -> SignData {
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
            if config.project_id == identity {
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
        if let Some(configs) = &self.gsa {
          for config in configs {
            if config.project_id == identity {
              return Some(Box::new(config.clone()));
            }
          }
        }
        return None;
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
          "member_id": "P34MVSHVGBDSLOQ6BGSAL4SFEN",
          "project_id": "OEP003178"
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
  fn test_gsaconfig_sign() {
    let url = "gsa://yjcyxky@163.com/HRA0001/HRS335657/HRX427830/HRR593463/HRR592563_f1.fastq.gz";
    let config_str = r#"
      {
        "gsa": [{
          "project_id": "HRA0001",
          "download_url": "https://share.cncb.ac.cn/",
          "account_name": "yjcyxky@163.com",
          "shared_id": "vsSyAX3A"
        }]
      }
    "#;
    let config = RepoConfig::read_config_data(config_str).unwrap();
    let gsa = config.gsa.unwrap();
    let config = gsa[0].clone();
    let signed = config.sign(url);
    assert!(signed.baseurl == "https://share.cncb.ac.cn/vsSyAX3A/HRA0001/HRR593463/HRR592563_f1.fastq.gz".to_string());
  }

  #[test]
  fn test_nodeconfig_sign() {
    let url = "node://yjcyxky@163.com/OEP003178/OEX015832/OES135568/OER249692/OED718095";
    let config_str = r#"
      {
        "node": [{
          "project_id": "OEP003178",
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
          "project_id": "OEP003178",
          "download_url": "https://www.biosino.org/download/downloadByMember",
          "member_id": "P34MVSHVGBDSLOQ6BGSAL4SFEN",
          "account_name": "yjcyxky@163.com"
        }]
      }
    "#;
    let url = "node://yjcyxky@163.com/OEP003178/OEX015832/OES135568/OER249692/OED718095";
    let config = RepoConfig::read_config_data(config_str).unwrap();
    let c = config.fetch_config("node", "OEP003178").unwrap();
    assert!(c.sign(url).header == ["Content-Type: application/x-www-form-urlencoded".to_string()]);
  }
}
