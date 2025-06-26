use duckdb::types::{ValueRef, ValueRef::*};
use duckdb::Row;
use serde_json::{Map, Value};
use log::debug;

pub fn row_to_json(row: &Row, column_names: &[String]) -> Result<Value, duckdb::Error> {
    let mut record = Map::new();

    for (i, col_name) in column_names.iter().enumerate() {
        let value_ref = match row.get_ref(i) {
            Ok(value_ref) => value_ref,
            Err(e) => {
                debug!("Error getting value for column {}: {}", col_name, e);
                continue;
            }
        };
        let value = to_json_from_value(value_ref);
        record.insert(col_name.clone(), value);
    }

    Ok(Value::Object(record))
}

fn to_json_from_value(value: ValueRef) -> Value {
    match value {
        Null => Value::Null,
        Boolean(v) => Value::Bool(v),

        TinyInt(v) => Value::Number(v.into()),
        SmallInt(v) => Value::Number(v.into()),
        Int(v) => Value::Number(v.into()),
        BigInt(v) => Value::Number(v.into()),

        UTinyInt(v) => Value::Number(v.into()),
        USmallInt(v) => Value::Number(v.into()),
        UInt(v) => Value::Number(v.into()),
        UBigInt(v) => Value::Number(serde_json::Number::from(v)),

        HugeInt(v) => Value::String(v.to_string()),

        Float(v) => serde_json::json!(v),
        Double(v) => serde_json::json!(v),
        Decimal(v) => serde_json::json!(v),

        Timestamp(_, ts) => Value::String(ts.to_string()),
        Date32(v) => Value::String(v.to_string()),
        Time64(_, v) => Value::String(v.to_string()),

        Text(bytes) => match std::str::from_utf8(bytes) {
            Ok(s) => {
                let trimmed = s.trim();
                if trimmed.starts_with('{') || trimmed.starts_with('[') {
                    serde_json::from_str(trimmed).unwrap_or(Value::String(s.to_string()))
                } else {
                    Value::String(s.to_string())
                }
            }
            Err(_) => Value::String(base64::encode(bytes)),
        },

        Blob(bytes) => Value::String(hex::encode(bytes)),

        other => {
            eprintln!("Unsupported or nested type: {:?}", other);
            Value::Null
        }
    }
}
