//! A SQL builder for building SQL queries.

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
    Int(i32),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    ArrayString(Vec<String>),
    ArrayInt(Vec<i32>),
    ArrayFloat(Vec<f64>),
    ArrayBool(Vec<bool>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueryItem {
    pub field: String,
    pub value: Value,
    pub operator: String, // =, !=, like, not like, ilike, in, not in, %
}

impl QueryItem {
    pub fn new(field: String, value: Value, operator: String) -> Self {
        let allowed_operators = vec![
            "=", "!=", "like", "not like", "ilike", "in", "not in", "<>", "<", ">", "<=", ">=",
            "is", "is not", "%",
        ];
        if !allowed_operators.contains(&operator.as_str()) {
            panic!("Invalid operator: {}", operator);
        }

        match value {
            Value::Int(_) => {
                if !vec!["=", "!=", ">", "<", "<=", ">="].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::Float(_) => {
                if !vec!["=", "!=", ">", "<", "<=", ">="].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::String(_) => {
                if !vec!["=", "!=", "like", "not like", "ilike", "<>", "%"]
                    .contains(&operator.as_str())
                {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::Bool(_) => {
                if !vec!["=", "!="].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::Null => {
                if !vec!["is", "is not"].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::ArrayString(_) => {
                if !vec!["in", "not in"].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::ArrayInt(_) => {
                if !vec!["in", "not in"].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::ArrayFloat(_) => {
                if !vec!["in", "not in"].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::ArrayBool(_) => {
                if !vec!["in", "not in"].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
        }

        Self {
            field,
            value,
            operator,
        }
    }

    pub fn get_field(&self) -> &str {
        &self.field
    }

    pub fn get_field_value_pair(&self) -> Option<(String, String)> {
        // Only return a string value
        match &self.value {
            Value::String(v) => Some((self.field.clone(), v.clone())),
            _ => None,
        }
    }

    pub fn default() -> Self {
        QueryItem::new(
            "1".to_string(),
            Value::String("1".to_string()),
            "=".to_string(),
        )
    }

    pub fn format(&self) -> String {
        match &self.value {
            Value::Int(v) => format!("{} {} {}", self.field, self.operator, v),
            Value::Float(v) => format!("{} {} {}", self.field, self.operator, v),
            Value::String(v) => format!("{} {} '{}'", self.field, self.operator, v),
            Value::Bool(v) => format!("{} {} {}", self.field, self.operator, v),
            Value::Null => format!("{} {} NULL", self.field, self.operator),
            Value::ArrayString(v) => {
                let mut values = vec![];
                for item in v {
                    values.push(format!("'{}'", item));
                }
                format!("{} {} ({})", self.field, self.operator, values.join(","))
            }
            Value::ArrayInt(v) => {
                let mut values = vec![];
                for item in v {
                    values.push(format!("{}", item));
                }
                format!("{} {} ({})", self.field, self.operator, values.join(","))
            }
            Value::ArrayFloat(v) => {
                let mut values = vec![];
                for item in v {
                    values.push(format!("{}", item));
                }
                format!("{} {} ({})", self.field, self.operator, values.join(","))
            }
            Value::ArrayBool(v) => {
                let mut values = vec![];
                for item in v {
                    values.push(format!("{}", item));
                }
                format!("{} {} ({})", self.field, self.operator, values.join(","))
            }
        }
    }

    /// Build a parameterised SQL fragment for this `QueryItem`.
    /// Returns the fragment together with collected parameters.
    pub fn to_sql_with_params(&self) -> (String, Vec<Value>) {
        // Helper that yields a single placeholder string like "$n" where n is 1-based.
        let mut params: Vec<Value> = Vec::new();

        // How many placeholders we will generate for this value?
        let (fragment, generated_params): (String, Vec<Value>) = match &self.value {
            // ---------- Scalar values ----------
            Value::Int(_) | Value::Float(_) | Value::String(_) | Value::Bool(_) => {
                let placeholder = format!("?");
                (
                    format!("{} {} {}", self.field, self.operator, placeholder),
                    vec![self.value.clone()],
                )
            }
            // Null is embedded literally – there is no value to bind.
            Value::Null => (format!("{} {} NULL", self.field, self.operator), vec![]),

            // ---------- Array values (IN / NOT IN) ----------
            Value::ArrayString(arr) => {
                let placeholders: Vec<String> =
                    arr.iter().enumerate().map(|(i, _)| format!("?")).collect();
                (
                    format!(
                        "{} {} ({})",
                        self.field,
                        self.operator,
                        placeholders.join(", ")
                    ),
                    arr.iter().cloned().map(Value::String).collect(),
                )
            }
            Value::ArrayInt(arr) => {
                let placeholders: Vec<String> =
                    arr.iter().enumerate().map(|(i, _)| format!("?")).collect();
                (
                    format!(
                        "{} {} ({})",
                        self.field,
                        self.operator,
                        placeholders.join(", ")
                    ),
                    arr.iter().cloned().map(Value::Int).collect(),
                )
            }
            Value::ArrayFloat(arr) => {
                let placeholders: Vec<String> =
                    arr.iter().enumerate().map(|(i, _)| format!("?")).collect();
                (
                    format!(
                        "{} {} ({})",
                        self.field,
                        self.operator,
                        placeholders.join(", ")
                    ),
                    arr.iter().cloned().map(Value::Float).collect(),
                )
            }
            Value::ArrayBool(arr) => {
                let placeholders: Vec<String> =
                    arr.iter().enumerate().map(|(i, _)| format!("?")).collect();
                (
                    format!(
                        "{} {} ({})",
                        self.field,
                        self.operator,
                        placeholders.join(", ")
                    ),
                    arr.iter().cloned().map(Value::Bool).collect(),
                )
            }
        };

        params.extend(generated_params);
        (fragment, params)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComposeQueryItem {
    /// and, or
    pub operator: String,
    /// QueryItem or ComposeQuery
    pub items: Vec<ComposeQuery>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ComposeQuery {
    QueryItem(QueryItem),
    ComposeQueryItem(ComposeQueryItem),
}

impl ComposeQuery {
    pub fn to_string(&self) -> String {
        let mut query_str = match self {
            ComposeQuery::QueryItem(item) => item.format(),
            ComposeQuery::ComposeQueryItem(item) => item.format(),
        };

        query_str
    }

    pub fn from_str(query_str: &str) -> Result<Option<Self>, serde_json::Error> {
        let query = if query_str == "" {
            None
        } else {
            Some(serde_json::from_str(&query_str)?)
        };

        Ok(query)
    }

    pub fn format(&self) -> String {
        match self {
            ComposeQuery::QueryItem(item) => item.format(),
            ComposeQuery::ComposeQueryItem(item) => item.format(),
        }
    }

    /// Recursively build WHERE clause from a `ComposeQuery`, assigning parameter placeholders
    /// and returning the fragment together with collected parameters.
    pub fn to_sql_with_params(&self) -> (String, Vec<Value>) {
        match self {
            ComposeQuery::QueryItem(item) => item.to_sql_with_params(),
            ComposeQuery::ComposeQueryItem(comp) => {
                let mut sql_parts: Vec<String> = Vec::new();
                let mut params: Vec<Value> = Vec::new();

                for (idx, sub) in comp.items.iter().enumerate() {
                    let (frag, mut p) = sub.to_sql_with_params();

                    if idx > 0 {
                        sql_parts.push(format!(" {} ", comp.operator.to_uppercase()));
                    }
                    sql_parts.push(frag);
                    params.append(&mut p);
                }

                (format!("({})", sql_parts.join("")), params)
            }
        }
    }
}

impl ComposeQueryItem {
    pub fn new(operator: &str) -> Self {
        Self {
            operator: operator.to_string(),
            items: vec![],
        }
    }

    pub fn get_fields(&self, fields: &mut Vec<String>) {
        for item in &self.items {
            match item {
                ComposeQuery::QueryItem(query_item) => {
                    // Check if the field is not already in the Vec and add it
                    if !fields.contains(&query_item.field) {
                        fields.push(query_item.field.clone());
                    }
                }
                ComposeQuery::ComposeQueryItem(compose_query_item) => {
                    // Recursively traverse nested ComposeQueryItem
                    compose_query_item.get_fields(fields);
                }
            }
        }
    }

    pub fn get_field_value_pairs(&self, pairs: &mut Vec<(String, String)>) {
        for item in &self.items {
            match item {
                ComposeQuery::QueryItem(query_item) => {
                    // Check if the field is not already in the Vec and add it
                    if let Some(pair) = query_item.get_field_value_pair() {
                        if !pairs.contains(&pair) {
                            pairs.push(pair);
                        }
                    }
                }
                ComposeQuery::ComposeQueryItem(compose_query_item) => {
                    // Recursively traverse nested ComposeQueryItem
                    compose_query_item.get_field_value_pairs(pairs);
                }
            }
        }
    }

    // Why ComposeQuery here?
    // Because we can have nested ComposeQueryItem, it maybe a QueryItem or ComposeQueryItem
    pub fn add_item(&mut self, item: ComposeQuery) -> &mut Self {
        self.items.push(item);
        self
    }

    pub fn default() -> Self {
        let mut default_query = ComposeQueryItem::new("and");
        default_query.add_item(ComposeQuery::QueryItem(QueryItem::new(
            "1".to_string(),
            Value::Int(1),
            "=".to_string(),
        )));

        default_query
    }

    pub fn format(&self) -> String {
        let mut query = String::new();

        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                query.push_str(&format!(" {} ", self.operator));
            }

            match item {
                ComposeQuery::QueryItem(item) => {
                    query.push_str(&item.format());
                }
                ComposeQuery::ComposeQueryItem(item) => {
                    query.push_str(&format!("({})", item.format()));
                }
            }
        }
        query
    }
}

pub fn get_all_fields(query: &ComposeQuery) -> Vec<String> {
    match query {
        ComposeQuery::QueryItem(query_item) => {
            let mut fields = Vec::new();
            fields.push(query_item.get_field().to_string());
            return fields;
        }
        ComposeQuery::ComposeQueryItem(query) => {
            let mut fields = Vec::new();
            query.get_fields(&mut fields);
            return fields;
        }
    }
}

pub fn get_all_field_pairs(query: &ComposeQuery) -> Vec<(String, String)> {
    match query {
        ComposeQuery::QueryItem(query_item) => {
            let mut pairs = Vec::new();
            if let Some(pair) = query_item.get_field_value_pair() {
                pairs.push(pair);
            }
            return pairs;
        }
        ComposeQuery::ComposeQueryItem(query) => {
            let mut pairs = Vec::new();
            query.get_field_value_pairs(&mut pairs);
            return pairs;
        }
    }
}

pub fn make_order_clause(fields: Vec<String>) -> String {
    let mut order_by = String::new();
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            order_by.push_str(", ");
        }
        order_by.push_str(field);
    }
    order_by
}

pub fn make_order_clause_by_pairs(pairs: Vec<(String, String)>, topk: usize) -> String {
    let mut topk_pairs = Vec::new();
    if topk != 0 {
        let k = if pairs.len() < topk {
            pairs.len()
        } else {
            topk
        };
        topk_pairs = pairs[0..k].to_vec();
    } else {
        topk_pairs = pairs;
    }

    let mut order_by = String::new();
    for (i, pair) in topk_pairs.iter().enumerate() {
        if i > 0 {
            order_by.push_str(", ");
        }

        // Trim all special characters in the head and tail of the string
        let patterns: &[_] = &[
            '~', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '+', '=', '{', '}', '[',
            ']', '|', '\\', ':', ';', '"', '\'', '<', '>', ',', '.', '?', '/', ' ',
        ];
        let cleaned_str = pair.1.trim_matches(patterns);
        order_by.push_str(&format!("similarity({}, '{}') DESC", pair.0, cleaned_str));
    }
    order_by
}

// Test code
#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_logger;
    use log::LevelFilter;

    #[test]
    fn test_compose_query() {
        let _ = init_logger("sql-builder-test", LevelFilter::Debug);
        let mut query = ComposeQueryItem::new("and");
        query.add_item(ComposeQuery::QueryItem(QueryItem::new(
            "id".to_string(),
            Value::Int(1),
            "=".to_string(),
        )));
        query.add_item(ComposeQuery::QueryItem(QueryItem::new(
            "name".to_string(),
            Value::String("test".to_string()),
            "like".to_string(),
        )));

        let mut compose_query = ComposeQueryItem::new("or");
        compose_query.add_item(ComposeQuery::QueryItem(QueryItem::new(
            "id".to_string(),
            Value::Int(2),
            "=".to_string(),
        )));
        compose_query.add_item(ComposeQuery::QueryItem(QueryItem::new(
            "name".to_string(),
            Value::String("test2".to_string()),
            "like".to_string(),
        )));

        query.add_item(ComposeQuery::ComposeQueryItem(compose_query));

        assert_eq!(
            query.format(),
            "id = 1 and name like 'test' and (id = 2 or name like 'test2')"
        );

        let mut fields = Vec::new();
        query.get_fields(&mut fields);
        debug!("fields: {:?}", fields);
        assert_eq!(2, fields.len());

        let mut pairs = Vec::new();
        query.get_field_value_pairs(&mut pairs);
        debug!("pairs: {:?}", pairs);
        assert_eq!(2, pairs.len());
    }

    #[test]
    #[should_panic(expected = "Invalid operator")]
    fn test_invalid_operator() {
        let _ = QueryItem::new("age".to_string(), Value::Int(30), "like".to_string());
    }

    #[test]
    fn test_format_various_values() {
        let int_item = QueryItem::new("age".to_string(), Value::Int(25), ">".to_string());
        assert_eq!(int_item.format(), "age > 25");

        let str_item = QueryItem::new(
            "name".to_string(),
            Value::String("Alice".into()),
            "like".into(),
        );
        assert_eq!(str_item.format(), "name like 'Alice'");

        let bool_item = QueryItem::new("active".into(), Value::Bool(true), "=".into());
        assert_eq!(bool_item.format(), "active = true");

        let null_item = QueryItem::new("deleted".into(), Value::Null, "is".into());
        assert_eq!(null_item.format(), "deleted is NULL");

        let arr_str = QueryItem::new(
            "tags".into(),
            Value::ArrayString(vec!["x".into(), "y".into()]),
            "in".into(),
        );
        assert_eq!(arr_str.format(), "tags in ('x','y')");
    }

    #[test]
    fn test_to_sql_with_params_scalar() {
        let item = QueryItem::new("score".into(), Value::Float(88.5), ">".into());
        let (sql, params) = item.to_sql_with_params();
        assert_eq!(sql, "score > ?");
        assert_eq!(params, vec![Value::Float(88.5)]);
    }

    #[test]
    fn test_to_sql_with_params_array() {
        let item = QueryItem::new("ids".into(), Value::ArrayInt(vec![1, 2, 3]), "in".into());
        let (sql, params) = item.to_sql_with_params();
        assert_eq!(sql, "ids in (?, ?, ?)");
        assert_eq!(params, vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    }

    #[test]
    fn test_compose_query_nested_and_or() {
        let inner = ComposeQueryItem::new("or")
            .add_item(ComposeQuery::QueryItem(QueryItem::new(
                "a".into(),
                Value::Int(1),
                "=".into(),
            )))
            .add_item(ComposeQuery::QueryItem(QueryItem::new(
                "b".into(),
                Value::Int(2),
                "=".into(),
            )))
            .clone();

        let mut outer = ComposeQueryItem::new("and");
        outer
            .add_item(ComposeQuery::QueryItem(QueryItem::new(
                "c".into(),
                Value::Int(3),
                ">".into(),
            )))
            .add_item(ComposeQuery::ComposeQueryItem(inner));

        assert_eq!(outer.format(), "c > 3 and (a = 1 or b = 2)");
    }

    #[test]
    fn test_get_all_fields_and_pairs() {
        let q = ComposeQueryItem::new("and")
            .add_item(ComposeQuery::QueryItem(QueryItem::new(
                "x".into(),
                Value::String("a".into()),
                "=".into(),
            )))
            .add_item(ComposeQuery::QueryItem(QueryItem::new(
                "y".into(),
                Value::Int(2),
                ">".into(),
            )))
            .clone();

        let cq = ComposeQuery::ComposeQueryItem(q);

        let fields = get_all_fields(&cq);
        assert!(fields.contains(&"x".into()));
        assert!(fields.contains(&"y".into()));

        let pairs = get_all_field_pairs(&cq);
        assert_eq!(pairs, vec![("x".into(), "a".into())]);
    }

    #[test]
    fn test_order_clause_builders() {
        let fields = vec!["name".into(), "score".into()];
        assert_eq!(make_order_clause(fields), "name, score");

        let pairs = vec![
            ("name".into(), " alice ".into()),
            ("summary".into(), "~good~".into()),
        ];
        assert_eq!(
            make_order_clause_by_pairs(pairs.clone(), 2),
            "similarity(name, 'alice') DESC, similarity(summary, 'good') DESC"
        );
    }

    #[test]
    fn test_queryitem_default_and_getters() {
        let q = QueryItem::default();
        assert_eq!(q.get_field(), "1");

        let v = QueryItem::new("k".into(), Value::String("v".into()), "=".into());
        assert_eq!(v.get_field_value_pair(), Some(("k".into(), "v".into())));
    }

    #[test]
    fn test_from_str_example1() {
        let json = r#"
    {
      "operator": "AND",
      "items": [
        {
          "operator": "OR",
          "items": [
            {
              "operator": "ilike",
              "field": "name",
              "value": "%CFS%"
            },
            {
              "operator": "ilike",
              "field": "id",
              "value": "%CFS%"
            },
            {
              "operator": "ilike",
              "field": "synonyms",
              "value": "%CFS%"
            },
            {
              "operator": "ilike",
              "field": "xrefs",
              "value": "%CFS%"
            }
          ]
        },
        {
          "operator": "=",
          "field": "label",
          "value": "Disease"
        }
      ]
    }"#;

        let result = ComposeQuery::from_str(json).unwrap();
        assert!(result.is_some());

        if let Some(ComposeQuery::ComposeQueryItem(item)) = result {
            assert_eq!(item.operator.to_lowercase(), "and");
            assert_eq!(item.items.len(), 2);

            // Check first item is also ComposeQueryItem with 4 QueryItem children
            if let ComposeQuery::ComposeQueryItem(inner) = &item.items[0] {
                assert_eq!(inner.operator.to_lowercase(), "or");
                assert_eq!(inner.items.len(), 4);
            } else {
                panic!("Expected nested ComposeQueryItem");
            }
        } else {
            panic!("Expected ComposeQueryItem");
        }
    }

    #[test]
    fn test_from_str_example2() {
        let json = r#"
    {
      "operator": "or",
      "items": [
        {
          "operator": "ilike",
          "field": "name",
          "value": "%CFS%"
        },
        {
          "operator": "ilike",
          "field": "id",
          "value": "%CFS%"
        }
      ]
    }"#;

        let result = ComposeQuery::from_str(json).unwrap();
        assert!(result.is_some());

        if let Some(ComposeQuery::ComposeQueryItem(item)) = result {
            assert_eq!(item.operator.to_lowercase(), "or");
            assert_eq!(item.items.len(), 2);

            for sub in &item.items {
                if let ComposeQuery::QueryItem(qi) = sub {
                    assert_eq!(qi.operator.to_lowercase(), "ilike");
                    assert_eq!(qi.value, Value::String("%CFS%".into()));
                } else {
                    panic!("Expected QueryItem");
                }
            }
        } else {
            panic!("Expected ComposeQueryItem");
        }
    }

    #[test]
    fn test_from_str_empty() {
        let result = ComposeQuery::from_str("").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_from_str_invalid_json() {
        let json = r#"{ "operator": "and", "items": [ { "bad": "key" } ] }"#;
        let result = ComposeQuery::from_str(json);
        assert!(result.is_err()); // should fail due to missing required keys like `field` and `value`
    }
}
