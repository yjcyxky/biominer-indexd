use crate::query_builder::where_builder::{ComposeQuery, Value};
use anyhow::{anyhow, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SelectExpr {
    Field {
        value: String,
    },
    AggFunc {
        func: String,
        field: String,
        alias: Option<String>,
    },
}

impl SelectExpr {
    fn is_valid_agg_func(func: &str) -> bool {
        matches!(
            func.to_uppercase().as_str(),
            "COUNT" | "AVG" | "SUM" | "MIN" | "MAX"
        )
    }

    fn is_valid_identifier(name: &str) -> bool {
        let re = regex::Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
        re.is_match(name)
    }

    pub fn validate(&self, field_table_map: Option<&HashMap<String, String>>) -> Result<(), Error> {
        match self {
            SelectExpr::Field { value } => {
                if value.is_empty() {
                    return Err(anyhow!("Field name is empty."));
                }

                if !Self::is_valid_identifier(value) {
                    return Err(anyhow!("Invalid field name: '{}'", value));
                }

                // How to check if the field is in a specific table? In especial, join clause exists.
                if let Some(map) = field_table_map {
                    // check if the field is in the map
                    if !map.contains_key(value) {
                        return Err(anyhow!("Field '{}' is not in any table.", value));
                    }
                }
            }
            SelectExpr::AggFunc { func, field, alias } => {
                if !Self::is_valid_agg_func(func) {
                    return Err(anyhow!("Unsupported aggregation function: '{}'.", func));
                }

                if field.is_empty() {
                    return Err(anyhow!("Field name is empty."));
                }

                if field != "*" && !Self::is_valid_identifier(field) {
                    return Err(anyhow!("Invalid field name in aggregation: '{}'", field));
                }

                if let Some(alias) = alias {
                    if !Self::is_valid_identifier(alias) {
                        return Err(anyhow!("Invalid alias: '{}'", alias));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn format(
        &self,
        field_table_map: Option<&HashMap<String, String>>,
        multi_table: bool,
    ) -> String {
        match self {
            SelectExpr::Field { value } => {
                if let Some(map) = field_table_map {
                    if let Some(t) = map.get(value) {
                        // If only one table, no need to add the table name.
                        if !multi_table {
                            format!("{}", value)
                        } else {
                            format!("{}.{}", t, value)
                        }
                    } else {
                        value.clone()
                    }
                } else {
                    value.clone()
                }
            }
            SelectExpr::AggFunc { func, field, alias } => {
                let base = if field == "*" {
                    format!("{}(*)", func.to_uppercase())
                } else {
                    format!("{}({})", func.to_uppercase(), field)
                };
                if let Some(alias) = alias {
                    format!("{} AS {}", base, alias)
                } else {
                    base
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AggExpr {
    Alias { value: String },
    Function { func: String, field: String },
}

impl AggExpr {
    pub fn is_empty(&self) -> bool {
        match self {
            AggExpr::Alias { value } => value.is_empty(),
            AggExpr::Function { func, field } => func.is_empty() || field.is_empty(),
        }
    }

    pub fn format(&self) -> String {
        match self {
            AggExpr::Alias { value } => value.clone(),
            AggExpr::Function { func, field } => {
                if field == "*" {
                    format!("{}(*)", func.to_uppercase())
                } else {
                    format!("{}({})", func.to_uppercase(), field)
                }
            }
        }
    }

    fn is_valid_agg_func(func: &str) -> bool {
        matches!(
            func.to_uppercase().as_str(),
            "COUNT" | "AVG" | "SUM" | "MIN" | "MAX"
        )
    }

    fn is_valid_identifier(name: &str) -> bool {
        let re = regex::Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
        re.is_match(name)
    }

    pub fn validate(&self, field_table_map: Option<&HashMap<String, String>>) -> Result<(), Error> {
        match self {
            AggExpr::Alias { value } => {
                if value.is_empty() {
                    return Err(anyhow!("Alias is empty."));
                } else if !Self::is_valid_identifier(value) {
                    return Err(anyhow!("Invalid alias: '{}'", value));
                }

                if let Some(map) = field_table_map {
                    if !map.contains_key(value) {
                        return Err(anyhow!("Alias '{}' is not in any table.", value));
                    }
                }
            }
            AggExpr::Function { func, field } => {
                if !Self::is_valid_agg_func(func) {
                    return Err(anyhow!("Unsupported aggregation function: '{}'.", func));
                }
                if field != "*" && !Self::is_valid_identifier(field) {
                    return Err(anyhow!("Invalid field name in aggregation: '{}'", field));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HavingExpr {
    pub left: AggExpr,
    pub operator: String,
    pub value: Value,
}

impl HavingExpr {
    pub fn to_sql_with_params(&self) -> (String, Vec<Value>) {
        let sql = format!("{} {} ?", self.left.format(), self.operator);
        (sql, vec![self.value.clone()])
    }

    pub fn is_empty(&self) -> bool {
        return self.left.is_empty() || self.operator.is_empty();
    }

    pub fn validate(&self, field_table_map: Option<&HashMap<String, String>>) -> Result<(), Error> {
        self.left.validate(field_table_map)?;
        if self.operator.is_empty() {
            return Err(anyhow!("Operator is empty."));
        }
        Ok(())
    }

    pub fn format(&self) -> String {
        match &self.value {
            Value::Int(v) => format!("{} {} {}", self.left.format(), self.operator, v),
            Value::Float(v) => format!("{} {} {}", self.left.format(), self.operator, v),
            Value::String(v) => format!("{} {} {}", self.left.format(), self.operator, v),
            Value::Bool(v) => format!("{} {} {}", self.left.format(), self.operator, v),
            _ => format!("{} {} ?", self.left.format(), self.operator),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HavingClause {
    Expr {
        value: HavingExpr,
    },
    Group {
        operator: String, // "AND" or "OR"
        items: Vec<HavingClause>,
    },
}

impl HavingClause {
    pub fn is_empty(&self) -> bool {
        match self {
            HavingClause::Expr { value } => value.is_empty(),
            HavingClause::Group { operator, items } => items.is_empty(),
        }
    }

    pub fn format(&self) -> String {
        match self {
            HavingClause::Expr { value } => value.format(),
            HavingClause::Group { operator, items } => {
                format!(
                    "({})",
                    items
                        .iter()
                        .map(|i| i.format())
                        .collect::<Vec<_>>()
                        .join(&format!(" {} ", operator.to_uppercase()))
                )
            }
        }
    }

    pub fn to_sql_with_params(&self) -> (String, Vec<Value>) {
        match self {
            HavingClause::Expr { value } => value.to_sql_with_params(),
            HavingClause::Group { operator, items } => {
                let mut parts = Vec::new();
                let mut params = Vec::new();
                for item in items {
                    let (sub_clause, sub_params) = item.to_sql_with_params();
                    parts.push(sub_clause);
                    params.extend(sub_params);
                }
                (
                    format!(
                        "({})",
                        parts.join(&format!(" {} ", operator.to_uppercase()))
                    ),
                    params,
                )
            }
        }
    }

    pub fn validate(&self, field_table_map: Option<&HashMap<String, String>>) -> Result<(), Error> {
        match self {
            HavingClause::Expr { value } => value.validate(field_table_map),
            HavingClause::Group { operator, items } => {
                for item in items {
                    item.validate(field_table_map)?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JoinOn {
    Expr {
        left_table: String,
        left_field: String,
        right_table: String,
        right_field: String,
    },
    Raw {
        value: String,
    },
}

impl JoinOn {
    pub fn format(&self) -> String {
        match self {
            JoinOn::Expr {
                left_table,
                left_field,
                right_table,
                right_field,
            } => format!(
                "{}.{} = {}.{}",
                left_table, left_field, right_table, right_field
            ),
            JoinOn::Raw { value } => value.clone(),
        }
    }

    fn is_valid_identifier(name: &str) -> bool {
        let re = regex::Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
        re.is_match(name)
    }

    pub fn validate(&self) -> Result<(), Error> {
        match self {
            JoinOn::Expr {
                left_table,
                left_field,
                right_table,
                right_field,
            } => {
                if !Self::is_valid_identifier(left_table) || !Self::is_valid_identifier(right_table)
                {
                    return Err(anyhow!(
                        "Invalid table name: '{}' or '{}'",
                        left_table,
                        right_table
                    ));
                }

                if !Self::is_valid_identifier(left_field) || !Self::is_valid_identifier(right_field)
                {
                    return Err(anyhow!(
                        "Invalid field name: '{}' or '{}'",
                        left_field,
                        right_field
                    ));
                }
            }
            JoinOn::Raw { value } => {
                if value.is_empty() {
                    return Err(anyhow!("Raw join on clause is empty."));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JoinClause {
    pub table: String,
    pub on: JoinOn,
}

impl JoinClause {
    pub fn format(&self) -> String {
        format!("JOIN {} ON {}", self.table, self.on.format())
    }

    fn is_valid_identifier(name: &str) -> bool {
        let re = regex::Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
        re.is_match(name)
    }

    pub fn validate(&self) -> Result<(), Error> {
        if !Self::is_valid_identifier(&self.table) {
            return Err(anyhow!("Invalid table name '{}'.", self.table));
        }

        self.on.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SqlWithParams {
    pub sql: String,
    pub params: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryPlan {
    pub table: String,
    pub joins: Vec<JoinClause>,
    pub selects: Vec<SelectExpr>,
    pub filters: Option<ComposeQuery>,
    pub group_by: Vec<String>,
    pub having: Option<HavingClause>,
    pub order_by: Vec<(String, bool)>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub distinct: bool,
    pub field_table_map: Option<HashMap<String, String>>,
    pub field_type_map: Option<HashMap<String, String>>, // for type inference
}

impl QueryPlan {
    fn is_valid_identifier(name: &str) -> bool {
        let re = regex::Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
        re.is_match(name)
    }

    fn infer_field_type(&self, field: &str) -> Option<String> {
        self.field_type_map
            .as_ref()
            .and_then(|map| map.get(field))
            .cloned()
    }

    fn generate_alias(func: &str, field: &str) -> String {
        let normalized = field
            .replace("*", "all")
            .replace(|c: char| !c.is_alphanumeric(), "_");
        format!("{}_{}", func.to_lowercase(), normalized)
    }

    pub fn to_json(&self) -> Result<String, Error> {
        let json = serde_json::to_string(self)?;
        Ok(json)
    }

    pub fn from_json(json: &str) -> Result<QueryPlan, Error> {
        let plan: QueryPlan = serde_json::from_str(json)?;
        Ok(plan)
    }

    pub fn to_sql_with_params(&self) -> Result<SqlWithParams, Error> {
        self.build_sql(false)
    }

    pub fn to_sql(&self) -> Result<String, Error> {
        let sql_with_params = self.build_sql(false)?;
        Ok(self.replace_params_with_values(&sql_with_params.sql, &sql_with_params.params))
    }

    pub fn to_explain_sql(&self) -> Result<SqlWithParams, Error> {
        self.build_sql(true)
    }

    /// Replace parameter placeholders (?) with actual values in the SQL string
    fn replace_params_with_values(&self, sql: &str, params: &[Value]) -> String {
        let mut result = sql.to_string();
        let mut param_index = 0;

        while let Some(pos) = result.find('?') {
            if param_index >= params.len() {
                break;
            }

            let value_str = match &params[param_index] {
                Value::Int(i) => i.to_string(),
                Value::Float(f) => f.to_string(),
                Value::String(s) => format!("'{}'", s.replace("'", "''")), // Escape single quotes
                Value::Bool(b) => b.to_string(),
                Value::Null => "NULL".to_string(),
                Value::ArrayString(arr) => {
                    let values: Vec<String> = arr
                        .iter()
                        .map(|s| format!("'{}'", s.replace("'", "''")))
                        .collect();
                    format!("({})", values.join(", "))
                }
                Value::ArrayInt(arr) => {
                    let values: Vec<String> = arr.iter().map(|i| i.to_string()).collect();
                    format!("({})", values.join(", "))
                }
                Value::ArrayFloat(arr) => {
                    let values: Vec<String> = arr.iter().map(|f| f.to_string()).collect();
                    format!("({})", values.join(", "))
                }
                Value::ArrayBool(arr) => {
                    let values: Vec<String> = arr.iter().map(|b| b.to_string()).collect();
                    format!("({})", values.join(", "))
                }
            };

            result.replace_range(pos..pos + 1, &value_str);
            param_index += 1;
        }

        result
    }

    fn is_mixed_with_aggregation(&self) -> bool {
        let has_agg = self
            .selects
            .iter()
            .any(|expr| matches!(expr, SelectExpr::AggFunc { .. }));
        let has_non_agg = self
            .selects
            .iter()
            .any(|expr| matches!(expr, SelectExpr::Field { .. }));
        has_agg && has_non_agg
    }

    pub fn build_sql(&self, explain: bool) -> Result<SqlWithParams, Error> {
        self.validate()?;
        let mut params = Vec::new();

        let select_clause = self
            .selects
            .iter()
            .map(|expr| expr.format(self.field_table_map.as_ref(), !self.joins.is_empty()))
            .collect::<Vec<_>>()
            .join(", ");

        let mut sql = if explain {
            format!(
                "EXPLAIN SELECT{} {} FROM {}",
                if self.distinct { " DISTINCT" } else { "" },
                select_clause,
                self.table
            )
        } else {
            format!(
                "SELECT{} {} FROM {}",
                if self.distinct { " DISTINCT" } else { "" },
                select_clause,
                self.table
            )
        };

        for join in &self.joins {
            let on_clause = match &join.on {
                JoinOn::Expr {
                    left_table,
                    left_field,
                    right_table,
                    right_field,
                } => {
                    format!(
                        "{}.{} = {}.{}",
                        left_table, left_field, right_table, right_field
                    )
                }
                JoinOn::Raw { value } => value.clone(),
            };
            sql += &format!(" JOIN {} ON {}", join.table, on_clause);
        }

        if let Some(filter) = &self.filters {
            let (where_clause, extracted) = filter.to_sql_with_params();
            if !where_clause.is_empty() {
                sql += &format!(" WHERE {}", where_clause);
                params.extend(extracted);
            }
        }

        if !self.group_by.is_empty() {
            sql += &format!(" GROUP BY {}", self.group_by.join(", "));
        }

        if let Some(having_clause) = &self.having {
            let (having_sql, extracted) = having_clause.to_sql_with_params();
            if !having_sql.is_empty() {
                sql += &format!(" HAVING {}", having_sql);
                params.extend(extracted);
            }
        }

        if !self.order_by.is_empty() {
            let order_clause = self
                .order_by
                .iter()
                .map(|(f, desc)| {
                    if *desc {
                        format!("{} DESC", f)
                    } else {
                        format!("{} ASC", f)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            sql += &format!(" ORDER BY {}", order_clause);
        }

        if let Some(limit) = self.limit {
            sql += &format!(" LIMIT {}", limit);
        }

        Ok(SqlWithParams { sql, params })
    }

    /// Check if a field is in the field_table_map and in one of the expected tables.
    /// TODO: How to differentiate between the field in the table and the field in the join?
    fn field_in_field_table_map(&self, field: &str, expected_tables: &[&str]) -> bool {
        if let Some(map) = &self.field_table_map {
            if let Some(t) = map.get(field) {
                expected_tables.contains(&t.as_str())
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        let mut non_agg_fields = HashSet::new();
        let mut agg_aliases = HashSet::new();

        // Check if the table name is valid.
        if !Self::is_valid_identifier(&self.table) {
            return Err(anyhow!("Invalid table name '{}'.", self.table));
        }

        // Check if the join clause is valid.
        for join in &self.joins {
            join.validate()?;
        }

        // Check if the select clause is valid.
        let mut expected_tables = self
            .joins
            .iter()
            .map(|j| j.table.as_str())
            .collect::<Vec<_>>();
        expected_tables.push(&self.table);

        for expr in &self.selects {
            expr.validate(self.field_table_map.as_ref())?;
            match expr {
                SelectExpr::Field { value } => {
                    non_agg_fields.insert(value.clone());
                }
                SelectExpr::AggFunc { func, field, alias } => match alias {
                    Some(name) => {
                        if !agg_aliases.insert(name.clone()) {
                            return Err(anyhow!("Duplicate alias '{}' in SELECT clause", name));
                        }
                    }
                    None => return Err(anyhow!("Alias is required for aggregation: '{}'", func)),
                },
            }
        }

        // Check if the group by clause is valid.
        if self.is_mixed_with_aggregation() {
            if self.group_by.is_empty() {
                return Err(anyhow!("GROUP BY clause is required when aggregation and non-aggregation fields are mixed."));
            }

            for f in &non_agg_fields {
                if !self.group_by.contains(f) {
                    return Err(anyhow!(
                        "Field '{}' must appear in GROUP BY when aggregation and non-aggregation fields are mixed.",
                        f
                    ));
                }
            }
        }

        for f in &self.group_by {
            if !Self::is_valid_identifier(f) {
                return Err(anyhow!("Invalid GROUP BY field: '{}'", f));
            }

            if !self.field_in_field_table_map(f, &expected_tables) && !agg_aliases.contains(f) {
                return Err(anyhow!(
                    "Field '{}' is not in the table '{}' or is not an alias.",
                    f,
                    expected_tables.join(", ")
                ));
            }
        }

        // Check if the having clause is valid.
        if let Some(having_clause) = &self.having {
            having_clause.validate(self.field_table_map.as_ref())?;
        }

        // Check if the order by clause is valid.
        for (order_field, _) in &self.order_by {
            if !Self::is_valid_identifier(order_field) {
                return Err(anyhow!("Invalid ORDER BY field: '{}'", order_field));
            }

            if !self.field_in_field_table_map(order_field, &expected_tables)
                && !agg_aliases.contains(order_field)
            {
                return Err(anyhow!(
                    "Field '{}' is not in the table '{}' or is not an alias.",
                    order_field,
                    expected_tables.join(", ")
                ));
            }

            if !non_agg_fields.contains(order_field) && !agg_aliases.contains(order_field) {
                return Err(anyhow!(
                    "ORDER BY field '{}' must be in SELECT clause or be an alias.",
                    order_field
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query_builder::where_builder::{ComposeQuery, QueryItem};

    fn mock_field_table_map() -> HashMap<String, String> {
        HashMap::from([
            ("id".into(), "t1".into()),
            ("name".into(), "t1".into()),
            ("age".into(), "t1".into()),
            ("score".into(), "t2".into()),
            ("group".into(), "t1".into()),
        ])
    }

    fn mock_field_type_map() -> HashMap<String, String> {
        HashMap::from([
            ("age".into(), "int".into()),
            ("score".into(), "float".into()),
        ])
    }

    #[test]
    fn test_simple_select() {
        let plan = QueryPlan {
            table: "t1".into(),
            joins: vec![],
            selects: vec![SelectExpr::Field { value: "id".into() }],
            filters: None,
            group_by: vec![],
            having: None,
            order_by: vec![],
            limit: Some(10),
            offset: None,
            distinct: false,
            field_table_map: Some(mock_field_table_map()),
            field_type_map: None,
        };

        let sql = plan.to_sql().unwrap();
        println!("sql: {}", sql);
        assert!(sql.starts_with("SELECT id FROM t1"));
        assert!(sql.contains("LIMIT 10"));
    }

    #[test]
    fn test_select_with_aggregation_and_alias() {
        let plan = QueryPlan {
            table: "t1".into(),
            joins: vec![],
            selects: vec![
                SelectExpr::Field {
                    value: "group".into(),
                },
                SelectExpr::AggFunc {
                    func: "SUM".into(),
                    field: "age".into(),
                    alias: Some("sum_age".into()),
                },
            ],
            filters: None,
            group_by: vec!["group".into()],
            having: None,
            order_by: vec![("sum_age".into(), true)],
            limit: None,
            offset: None,
            distinct: false,
            field_table_map: Some(mock_field_table_map()),
            field_type_map: None,
        };

        let sql = plan.to_sql().unwrap();
        assert!(sql.contains("SUM(age) AS sum_age"));
        assert!(sql.contains("GROUP BY group"));
        assert!(sql.contains("ORDER BY sum_age DESC"));
    }

    #[test]
    fn test_invalid_table_name() {
        let mut plan = QueryPlan {
            table: "123table".into(), // invalid
            joins: vec![],
            selects: vec![SelectExpr::Field { value: "id".into() }],
            filters: None,
            group_by: vec![],
            having: None,
            order_by: vec![],
            limit: None,
            offset: None,
            distinct: false,
            field_table_map: Some(mock_field_table_map()),
            field_type_map: None,
        };

        let err = plan.validate().unwrap_err();
        assert!(err.to_string().contains("Invalid table name"));
    }

    #[test]
    fn test_duplicate_alias() {
        let plan = QueryPlan {
            table: "t1".into(),
            joins: vec![],
            selects: vec![
                SelectExpr::AggFunc {
                    func: "SUM".into(),
                    field: "age".into(),
                    alias: Some("total".into()),
                },
                SelectExpr::AggFunc {
                    func: "MAX".into(),
                    field: "age".into(),
                    alias: Some("total".into()), // duplicate alias
                },
            ],
            filters: None,
            group_by: vec![],
            having: None,
            order_by: vec![],
            limit: None,
            offset: None,
            distinct: false,
            field_table_map: Some(mock_field_table_map()),
            field_type_map: None,
        };

        let err = plan.validate().unwrap_err();
        assert!(err.to_string().contains("Duplicate alias"));
    }

    #[test]
    fn test_select_with_join_and_filter_and_having() {
        let filters =
            ComposeQuery::QueryItem(QueryItem::new("age".into(), Value::Int(20), ">".into()));

        let having_clause = HavingClause::Expr {
            value: HavingExpr {
                left: AggExpr::Function {
                    func: "avg".into(),
                    field: "score".into(),
                },
                operator: ">".into(),
                value: Value::Float(60.0),
            },
        };

        let plan = QueryPlan {
            table: "t1".into(),
            joins: vec![JoinClause {
                table: "t2".into(),
                on: JoinOn::Expr {
                    left_table: "t1".into(),
                    left_field: "id".into(),
                    right_table: "t2".into(),
                    right_field: "id".into(),
                },
            }],
            selects: vec![
                SelectExpr::Field {
                    value: "group".into(),
                },
                SelectExpr::AggFunc {
                    func: "AVG".into(),
                    field: "score".into(),
                    alias: Some("avg_score".into()),
                },
            ],
            filters: Some(filters),
            group_by: vec!["group".into()],
            having: Some(having_clause),
            order_by: vec![("avg_score".into(), false)],
            limit: Some(100),
            offset: None,
            distinct: true,
            field_table_map: Some(mock_field_table_map()),
            field_type_map: Some(mock_field_type_map()),
        };

        let SqlWithParams { sql, params } = plan.to_sql_with_params().unwrap();

        println!("sql: {}", sql);
        assert!(sql.starts_with("SELECT DISTINCT"));
        assert!(sql.contains("JOIN t2 ON t1.id = t2.id"));
        assert!(sql.contains("WHERE age > ?"));
        assert!(sql.contains("HAVING AVG(score) > ?"));
        assert_eq!(params, vec![Value::Int(20), Value::Float(60.0)]);
    }

    #[test]
    fn test_invalid_group_by_missing_select_field() {
        let plan = QueryPlan {
            table: "t1".into(),
            joins: vec![],
            selects: vec![
                SelectExpr::Field {
                    value: "name".into(),
                },
                SelectExpr::AggFunc {
                    func: "count".into(),
                    field: "*".into(),
                    alias: Some("cnt".into()),
                },
            ],
            filters: None,
            group_by: vec![], // should contain "name"
            having: None,
            order_by: vec![],
            limit: None,
            offset: None,
            distinct: false,
            field_table_map: Some(mock_field_table_map()),
            field_type_map: None,
        };

        let err = plan.validate().unwrap_err();
        println!("err: {}", err);
        assert!(err.to_string().contains("GROUP BY clause is required"));
    }

    #[test]
    fn test_to_json_and_from_json() {
        let plan = QueryPlan {
            table: "t1".into(),
            joins: vec![],
            selects: vec![SelectExpr::Field { value: "id".into() }],
            filters: None,
            group_by: vec![],
            having: None,
            order_by: vec![],
            limit: None,
            offset: None,
            distinct: false,
            field_table_map: Some(mock_field_table_map()),
            field_type_map: None,
        };

        let json = plan.to_json().unwrap();
        println!("json: {}", json);

        let plan2 = QueryPlan::from_json(&json).unwrap();
        println!("plan2: {:?}", plan2);
        assert_eq!(plan, plan2);
    }
}
