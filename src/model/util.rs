use super::datafile::{Alias, File, Hash, Tag, URL};
use anyhow::{Context, Error, Result};
use regex::Regex;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::fs::File as FsFile;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Parses flattened fields with a numeric group pattern into structured subfield groups.
///
/// This function extracts grouped fields from a flat TSV row representation,
/// supporting patterns like `url_0_url`, `url_0_status`, `tag_1_field_value`, etc.
/// It organizes the fields into indexed groups.
///
/// # Arguments
/// * `record` - A key-value map representing a TSV row (column name to string value).
/// * `prefix` - The prefix to match (e.g., "url", "tag", "hash").
/// * `field_set` - A list of valid subfield names expected for the given group type.
///
/// # Returns
/// A BTreeMap from group index to a HashMap of subfield name to value.
///
/// # Example
/// Input: {"url_0_url": "http://a.com", "url_0_status": "validated"}
/// Output: { 0 => { "url": "http://a.com", "status": "validated" } }
fn parse_grouped_fields(
    record: &HashMap<String, String>,
    prefix: &str,
    field_set: &[&str],
) -> BTreeMap<usize, HashMap<String, String>> {
    let pattern = format!(r"^{}_(\d+)_([a-zA-Z0-9_]+)$", prefix);
    let re = Regex::new(&pattern).unwrap();

    let mut groups: BTreeMap<usize, HashMap<String, String>> = BTreeMap::new();

    for (key, val) in record {
        if let Some(caps) = re.captures(key) {
            let idx = caps[1].parse::<usize>().unwrap_or(0);
            let field = &caps[2];
            if field_set.contains(&field) {
                groups
                    .entry(idx)
                    .or_default()
                    .insert(field.to_string(), val.clone());
            }
        }
    }

    groups
}

/// Parses a single TSV row (as a HashMap) into a `File` struct.
///
/// This function extracts core file metadata and dynamically collects associated
/// `URL`, `Tag`, `Hash`, and `Alias` objects using grouped flattened fields.
///
/// # Arguments
/// * `record` - A HashMap representing one line from the TSV file (column name to string value).
///
/// # Returns
/// A fully constructed `File` struct, with substructures serialized as `serde_json::Value`.
///
/// # Errors
/// Returns an error if essential fields (e.g., `guid`) are missing or unparseable.

fn parse_record(record: &HashMap<String, String>) -> Result<File> {
    let guid = record.get("guid").context("missing guid")?.to_string();

    // 解析 urls
    let url_groups = parse_grouped_fields(record, "url", &["url", "status", "uploader"]);
    let urls: Vec<URL> = url_groups
        .into_iter()
        .filter_map(|(_, group)| {
            group.get("url").map(|url| URL {
                id: 0,
                url: url.clone(),
                status: group
                    .get("status")
                    .cloned()
                    .unwrap_or_else(|| "pending".to_string()),
                uploader: group.get("uploader").cloned().unwrap_or_default(),
                created_at: now_ts(),
                file: Some(guid.clone()),
            })
        })
        .collect();

    // tags
    let tag_groups = parse_grouped_fields(record, "tag", &["field_name", "field_value"]);
    let tags: Vec<Tag> = tag_groups
        .into_iter()
        .filter_map(
            |(_, group)| match (group.get("field_name"), group.get("field_value")) {
                (Some(name), Some(value)) => Some(Tag {
                    id: 0,
                    field_name: name.clone(),
                    field_value: value.clone(),
                    file: Some(guid.clone()),
                }),
                _ => None,
            },
        )
        .collect();

    // hashes
    let hash_groups = parse_grouped_fields(record, "hash", &["hash_type", "hash"]);
    let hashes: Vec<Hash> = hash_groups
        .into_iter()
        .filter_map(
            |(_, group)| match (group.get("hash_type"), group.get("hash")) {
                (Some(t), Some(h)) => Some(Hash {
                    id: 0,
                    hash_type: t.clone(),
                    hash: h.clone(),
                    file: Some(guid.clone()),
                }),
                _ => None,
            },
        )
        .collect();

    // aliases
    let alias_groups = parse_grouped_fields(record, "alias", &["name"]);
    let aliases: Vec<Alias> = alias_groups
        .into_iter()
        .filter_map(|(_, group)| {
            group.get("name").map(|name| Alias {
                id: 0,
                name: name.clone(),
                file: Some(guid.clone()),
            })
        })
        .collect();

    // 构造主 File
    Ok(File {
        guid,
        filename: record.get("filename").unwrap_or(&"".to_string()).clone(),
        size: record
            .get("size")
            .unwrap_or(&"0".to_string())
            .parse()
            .unwrap_or(0),
        created_at: record
            .get("created_at")
            .map(|v| v.parse().unwrap_or(now_ts()))
            .unwrap_or(now_ts()),
        updated_at: record
            .get("updated_at")
            .map(|v| v.parse().unwrap_or(now_ts()))
            .unwrap_or(now_ts()),
        status: record.get("status").unwrap_or(&"".to_string()).clone(),
        baseid: record.get("baseid").unwrap_or(&"".to_string()).clone(),
        rev: record.get("rev").unwrap_or(&"".to_string()).clone(),
        version: record
            .get("version")
            .unwrap_or(&"1".to_string())
            .parse()
            .unwrap_or(1),
        uploader: record.get("uploader").unwrap_or(&"".to_string()).clone(),
        access: record
            .get("access")
            .unwrap_or(&"private".to_string())
            .clone(),
        acl: record.get("acl").cloned(),

        urls: Some(json!(urls)),
        hashes: Some(json!(hashes)),
        aliases: Some(json!(aliases)),
        tags: Some(json!(tags)),
    })
}

/// Loads and parses a TSV file into a list of `File` objects.
///
/// This function reads a tab-delimited file, interprets the first line as the header,
/// and parses each subsequent row into a `File` struct using `parse_record()`.
///
/// # Arguments
/// * `filepath` - Path to the TSV file to load.
///
/// # Returns
/// A vector of parsed `File` objects.
///
/// # Errors
/// Returns an error if the file cannot be read, the header is missing, or row parsing fails.
pub fn load_tsv(filepath: &PathBuf) -> Result<Vec<File>, Error> {
    let file = match FsFile::open(filepath) {
        Ok(file) => file,
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to open file {}: {}", filepath.display(), e));
        }
    };
    let mut lines = BufReader::new(file).lines();

    let headers: Vec<String> = lines
        .next()
        .context("TSV has no header")??
        .split('\t')
        .map(|s| s.to_string())
        .collect();

    let mut result = Vec::new();

    for line in lines {
        let line = line?;
        let values: Vec<String> = line.split('\t').map(|s| s.to_string()).collect();

        let mut map = HashMap::new();
        for (h, v) in headers.iter().zip(values.iter()) {
            map.insert(h.clone(), v.clone());
        }

        let file_struct = parse_record(&map)?;
        result.push(file_struct);
    }

    Ok(result)
}

/// Flattens a `File` struct into a list of (key, value) string pairs suitable for TSV output.
///
/// This function decomposes the `File`'s main fields and serializes associated
/// `URL`, `Tag`, `Hash`, and `Alias` substructures using field names like:
/// `url_0_url`, `tag_1_field_value`, `hash_0_hash_type`, etc.
///
/// # Arguments
/// * `file` - A reference to a `File` object to flatten.
///
/// # Returns
/// A vector of key-value string pairs, ready for writing to a TSV row.
fn flatten_file(file: &File) -> Vec<(String, Value)> {
    let mut row = vec![
        ("guid".to_string(), json!(file.guid.clone())),
        ("filename".to_string(), json!(file.filename.clone())),
        ("size".to_string(), json!(file.size)),
        ("created_at".to_string(), json!(file.created_at)),
        ("updated_at".to_string(), json!(file.updated_at)),
        ("status".to_string(), json!(file.status.clone())),
        ("baseid".to_string(), json!(file.baseid.clone())),
        ("rev".to_string(), json!(file.rev.clone())),
        ("version".to_string(), json!(file.version)),
        ("uploader".to_string(), json!(file.uploader.clone())),
        ("access".to_string(), json!(file.access.clone())),
    ];

    if let Some(acl) = &file.acl {
        row.push(("acl".to_string(), json!(acl.clone())));
    }

    // URLs
    if let Some(Value::Array(urls)) = &file.urls {
        for (i, item) in urls.iter().enumerate() {
            if let Ok(u) = serde_json::from_value::<URL>(item.clone()) {
                row.push((format!("url_{}_url", i), json!(u.url.clone())));
                row.push((format!("url_{}_status", i), json!(u.status.clone())));
                row.push((format!("url_{}_uploader", i), json!(u.uploader.clone())));
            }
        }
    }

    // Tags
    if let Some(Value::Array(tags)) = &file.tags {
        for (i, item) in tags.iter().enumerate() {
            if let Ok(t) = serde_json::from_value::<Tag>(item.clone()) {
                row.push((format!("tag_{}_field_name", i), json!(t.field_name.clone())));
                row.push((
                    format!("tag_{}_field_value", i),
                    json!(t.field_value.clone()),
                ));
            }
        }
    }

    // Hashes
    if let Some(Value::Array(hashes)) = &file.hashes {
        for (i, item) in hashes.iter().enumerate() {
            if let Ok(h) = serde_json::from_value::<Hash>(item.clone()) {
                row.push((format!("hash_{}_hash_type", i), json!(h.hash_type.clone())));
                row.push((format!("hash_{}_hash", i), json!(h.hash.clone())));
            }
        }
    }

    // Aliases
    if let Some(Value::Array(aliases)) = &file.aliases {
        for (i, item) in aliases.iter().enumerate() {
            if let Ok(a) = serde_json::from_value::<Alias>(item.clone()) {
                row.push((format!("alias_{}_name", i), json!(a.name.clone())));
            }
        }
    }

    row
}

/// Converts a list of `File` objects into flattened TSV-style row maps.
///
/// Each `File` is flattened into a `HashMap<String, String>` using the `flatten_file` function,
/// and missing keys are normalized to ensure all rows share the same column structure.
///
/// This function is ideal for generating TSV/CSV without exposing header management.
///
/// # Arguments
/// * `files` - A slice of `File` objects.
///
/// # Returns
/// A vector of row-wise field maps with consistent keys and normalized values.
pub fn to_hashmap(files: &[File]) -> Result<Vec<HashMap<String, Value>>, Error> {
    let mut all_keys = std::collections::BTreeSet::new();
    let mut raw_rows: Vec<HashMap<String, Value>> = Vec::new();

    // Step 1: Flatten all rows and collect all possible keys
    for file in files {
        let row_vec = flatten_file(file);
        let row_map: HashMap<_, _> = row_vec.into_iter().collect();

        for key in row_map.keys() {
            all_keys.insert(key.clone());
        }

        raw_rows.push(row_map);
    }

    let keys: Vec<String> = all_keys.into_iter().collect();

    // Step 2: Normalize all rows to have the same key set
    let normalized_rows: Vec<HashMap<String, Value>> = raw_rows
        .into_iter()
        .map(|row| {
            let mut normalized = HashMap::new();
            for key in &keys {
                normalized.insert(key.clone(), row.get(key).cloned().unwrap_or_default());
            }
            normalized
        })
        .collect();

    Ok(normalized_rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_load_file_template() {
        let path = Path::new("examples/file_template.tsv");
        assert!(path.exists(), "Template file does not exist: {:?}", path);

        let files = load_tsv(&path.to_path_buf()).expect("Failed to load TSV file");

        assert!(!files.is_empty(), "No files parsed from template");

        for file in files {
            // Check guid format
            assert!(
                file.guid.starts_with("biominer.fudan-pgx/"),
                "Invalid guid prefix"
            );
            assert_eq!(file.guid.len(), 36 + "biominer.fudan-pgx/".len());

            // Check baseid is UUID format
            assert_eq!(file.baseid.len(), 36);
            assert!(
                uuid::Uuid::parse_str(&file.baseid).is_ok(),
                "Invalid UUID in baseid"
            );

            // Check rev matches prefix of baseid
            assert_eq!(file.rev, &file.baseid[..8]);

            // Check size > 0
            assert!(file.size > 0);

            // Check status is one of allowed values
            let allowed = ["pending", "processing", "validated", "failed"];
            assert!(
                allowed.contains(&file.status.as_str()),
                "Unexpected status: {}",
                file.status
            );

            // Check urls
            if let Some(urls) = &file.urls {
                let urls: Vec<URL> =
                    serde_json::from_value(urls.clone()).expect("Invalid URL structure");
                for u in urls {
                    assert!(!u.url.is_empty(), "URL is empty");
                    assert!(allowed.contains(&u.status.as_str()), "Invalid URL status");
                    assert!(!u.uploader.is_empty(), "URL uploader is empty");
                }
            }

            // Check hashes
            if let Some(hashes) = &file.hashes {
                let hashes: Vec<Hash> =
                    serde_json::from_value(hashes.clone()).expect("Invalid Hash structure");
                for h in hashes {
                    assert!(!h.hash_type.is_empty(), "Hash type is empty");
                    assert!(!h.hash.is_empty(), "Hash value is empty");
                }
            }

            // Check aliases
            if let Some(aliases) = &file.aliases {
                let aliases: Vec<Alias> =
                    serde_json::from_value(aliases.clone()).expect("Invalid Alias structure");
                for a in aliases {
                    assert!(!a.name.is_empty(), "Alias name is empty");
                }
            }
        }
    }

    #[test]
    fn test_parse_grouped_fields() {
        use std::collections::HashMap;

        let mut record = HashMap::new();
        record.insert("url_0_url".to_string(), "http://a.com".to_string());
        record.insert("url_0_status".to_string(), "validated".to_string());
        record.insert("url_1_url".to_string(), "http://b.com".to_string());

        let grouped = parse_grouped_fields(&record, "url", &["url", "status"]);

        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[&0]["url"], "http://a.com");
        assert_eq!(grouped[&0]["status"], "validated");
        assert_eq!(grouped[&1]["url"], "http://b.com");
    }

    #[test]
    fn test_parse_record() {
        let mut row = std::collections::HashMap::new();
        row.insert(
            "guid".to_string(),
            "biominer.fudan-pgx/12345678-abcd-1234-abcd-1234567890ab".to_string(),
        );
        row.insert("filename".to_string(), "test.fq.gz".to_string());
        row.insert("size".to_string(), "123456".to_string());
        row.insert("status".to_string(), "validated".to_string());
        row.insert(
            "baseid".to_string(),
            "12345678-abcd-1234-abcd-1234567890ab".to_string(),
        );
        row.insert("rev".to_string(), "12345678".to_string());
        row.insert("version".to_string(), "1".to_string());
        row.insert("uploader".to_string(), "tester".to_string());
        row.insert("access".to_string(), "private".to_string());

        row.insert("url_0_url".to_string(), "http://a.com".to_string());
        row.insert("url_0_status".to_string(), "validated".to_string());
        row.insert("url_0_uploader".to_string(), "tester".to_string());

        let parsed = parse_record(&row).unwrap();
        assert_eq!(parsed.filename, "test.fq.gz");
        assert_eq!(parsed.size, 123456);
        assert_eq!(parsed.status, "validated");
        assert!(parsed.urls.is_some());

        let urls: Vec<URL> = serde_json::from_value(parsed.urls.unwrap()).unwrap();
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].url, "http://a.com");
    }

    #[test]
    fn test_load_tsv_basic() {
        use std::fs::write;

        let content = "guid\tfilename\tsize\tstatus\tbaseid\trev\tversion\tuploader\taccess\turl_0_url\turl_0_status\turl_0_uploader\nbiominer.fudan-pgx/12345678-abcd-1234-abcd-1234567890ab\ttest.fq.gz\t123456\tvalidated\t12345678-abcd-1234-abcd-1234567890ab\t12345678\t1\ttester\tprivate\thttp://a.com\tvalidated\ttester";

        std::fs::create_dir_all("tmp").unwrap();
        write("tmp/test_load.tsv", content).unwrap();

        let files = load_tsv(&PathBuf::from("tmp/test_load.tsv")).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename, "test.fq.gz");
    }

    #[test]
    fn test_flatten_file() {
        let file = File {
            guid: "biominer.fudan-pgx/xxx".into(),
            filename: "file.fq.gz".into(),
            size: 1000,
            created_at: 111,
            updated_at: 222,
            status: "validated".into(),
            baseid: "xxx".into(),
            rev: "xxx".into(),
            version: 1,
            uploader: "tester".into(),
            access: "private".into(),
            acl: None,
            urls: Some(serde_json::json!([
                { "id": 0, "url": "http://a.com", "status": "validated", "uploader": "tester", "created_at": 0, "file": null }
            ])),
            tags: Some(serde_json::json!([
                { "id": 0, "field_name": "type", "field_value": "fq", "file": null }
            ])),
            hashes: Some(serde_json::json!([
                { "id": 0, "hash_type": "md5", "hash": "abcd", "file": null }
            ])),
            aliases: Some(serde_json::json!([
                { "id": 0, "name": "ALIAS001", "file": null }
            ])),
        };

        let row = flatten_file(&file);
        let map: std::collections::HashMap<_, _> = row.into_iter().collect();
        assert_eq!(map["filename"], "file.fq.gz");
        assert_eq!(map["url_0_url"], "http://a.com");
        assert_eq!(map["hash_0_hash_type"], "md5");
        assert_eq!(map["alias_0_name"], "ALIAS001");
    }

    #[test]
    fn test_export_tsv_basic() {
        let files = vec![File {
            guid: "biominer.fudan-pgx/xxx".into(),
            filename: "f.fq".into(),
            size: 1,
            created_at: 0,
            updated_at: 0,
            status: "validated".into(),
            baseid: "xxx".into(),
            rev: "xxx".into(),
            version: 1,
            uploader: "a".into(),
            access: "private".into(),
            acl: None,
            urls: Some(serde_json::json!([
                { "id": 0, "url": "http://x", "status": "validated", "uploader": "a", "created_at": 0, "file": null }
            ])),
            hashes: None,
            tags: None,
            aliases: None,
        }];

        let rows = to_hashmap(&files).unwrap();

        assert_eq!(rows.len(), 1);
    }
}
