use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

lazy_static! {
  static ref ACCEPTABLE_HASHES: HashMap<&'static str, Regex> = {
    let mut m = HashMap::new();
    m.insert("md5", Regex::new(r"^[0-9a-f]{32}$").unwrap());
    m.insert("sha1", Regex::new(r"^[0-9a-f]{40}$").unwrap());
    m.insert("sha256", Regex::new(r"^[0-9a-f]{64}$").unwrap());
    m.insert("sha512", Regex::new(r"^[0-9a-f]{128}$").unwrap());
    m.insert("crc32", Regex::new(r"^[0-9a-f]{8}$").unwrap());
    m.insert("etag", Regex::new(r"^[0-9a-f]{32}(-\d+)?$").unwrap());
    m.insert("crc64", Regex::new(r"^[0-9a-f]{16}$").unwrap());
    m
  };
}

pub fn validate_hash(hash: &str, algorithm: &str) -> bool {
  match ACCEPTABLE_HASHES.get(algorithm) {
    Some(regex) => regex.is_match(hash),
    None => false,
  }
}

pub fn which_hash_type(hash: &str) -> Option<&'static str> {
  for (algorithm, regex) in ACCEPTABLE_HASHES.iter() {
    if regex.is_match(hash) {
      return Some(algorithm);
    }
  }
  None
}

lazy_static! {
  static ref ACCEPTABLE_URLS: HashMap<&'static str, Regex> = {
    let mut m = HashMap::new();
    m.insert("node", Regex::new(r"^node://([a-zA-Z0-9@\-_.]+)/(OEP[0-9]+)/(OEX[0-9]+)/(OES[0-9]+)/(OER[0-9]+)/(OED[0-9]+)$").unwrap());
    m.insert("oss", Regex::new(r"^oss://([a-zA-Z0-9@\-_.]+)/(.*)$").unwrap());
    m.insert("s3", Regex::new(r"^s3://([a-zA-Z0-9@\-_.]+)/(.*)$").unwrap());
    m.insert("gsa", Regex::new(r"^gsa://([a-zA-Z0-9@\-_.]+)/(HRA[0-9]+)/(HRS[0-9]+)/(HRX[0-9]+)/(HRR[0-9]+)/(HRR[0-9]+.*)$").unwrap());
    m.insert("minio", Regex::new(r"^minio://([a-zA-Z0-9@\-_.]+)/(.*)$").unwrap());
    m
  };
}

pub fn validate_url(url: &str, protocol: &str) -> bool {
  match ACCEPTABLE_URLS.get(protocol) {
    Some(regex) => regex.is_match(url),
    None => false,
  }
}

pub fn which_protocol(url: &str) -> Option<&'static str> {
  for (protocol, regex) in ACCEPTABLE_URLS.iter() {
    if regex.is_match(url) {
      return Some(protocol);
    }
  }
  None
}

pub fn has_permission(auth_groups: &str, acl: &str) -> bool {
  let mut auth_group_vec: Vec<&str> = auth_groups.split(',').collect::<Vec<_>>().iter().map(|x| x.trim()).collect::<Vec<_>>();
  let mut acl_vec: Vec<&str> = acl.split(',').collect::<Vec<_>>().iter().map(|x| x.trim()).collect::<Vec<_>>();
  auth_group_vec.append(&mut acl_vec);
  let length = auth_group_vec.len();
  auth_group_vec.sort_unstable();
  auth_group_vec.dedup();
  return auth_group_vec.len() != length;
}
