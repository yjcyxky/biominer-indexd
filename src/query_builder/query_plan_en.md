# 🧠 QueryPlan Builder – Functional Specification

## 📌 Project Objective

To design a composable, structured, and secure SQL query builder that supports dynamic querying and aggregation across heterogeneous data sources (e.g., Parquet files). The system must support:

* Table-aware field mapping and type inference
* Parameterized SQL generation for injection protection
* Multi-table join handling
* Aggregation and nested filtering logic
* Validation and explanation of query semantics

---

## 1. 🧾 Field Selection (`SELECT` clause)

* Supports direct field selection:

  ```rust
  SelectExpr::Field(String)
  ```

* Supports aggregation expressions:

  ```rust
  SelectExpr::AggFunc {
      func: String,         // COUNT, SUM, AVG, etc.
      field: String,        // field name or '*'
      alias: Option<String> // optional alias
  }
  ```

* ✅ Auto aliasing: if `alias` is not specified, a default alias like `count_all` or `sum_expr` is generated

* ✅ Alias validation: must match SQL identifier rules

* ✅ Alias uniqueness: duplicate aliases in the same SELECT are not allowed

* ✅ Field name validation: disallows illegal characters (e.g., embedded dots)

---

## 2. 🔗 Multi-Table Joins (`JOIN` clause)

* Supports structured join conditions:

  ```rust
  JoinOn::Expr {
      left_table: String,
      left_field: String,
      right_table: String,
      right_field: String
  }
  ```

* Multiple joins are supported

* Field-to-table mapping is based on `field_table_map: HashMap<String, String>`

  * If specified, `gene_id` is mapped to `gene.gene_id`
  * If not specified, the original field name is used

* Validation includes:

  * Table and field names must be valid identifiers
  * Fields must exist in the provided `field_table_map`
  * Fields used in JOINs must belong to the correct table (disjoint mappings are not allowed)

---

## 3. 🧱 Filtering Conditions (`WHERE` clause)

* Uses a `ComposeQuery` tree to represent nested boolean logic (AND/OR)
* Supported operators: `=`, `!=`, `<`, `>`, `<=`, `>=`, `IN`, `NOT IN`, `IS`, `IS NOT`, `LIKE`, `ILIKE`, etc.
* Safe SQL generation using parameterized values: `to_sql_with_params()` outputs SQL + `Vec<Value>`

---

## 4. 📊 Grouping and Aggregation (`GROUP BY` and `HAVING`)

* `group_by: Vec<String>` specifies group keys
* `having: Option<HavingClause>` enables conditional aggregation:

  * Supports `COUNT(*) > 5`, `AVG(expr) < 2`, etc.
  * Nested logic via `Group { operator, items }` (e.g. `AND`, `OR`)
* HAVING clauses support both alias-based and function-based referencing
* Parameter binding is available for safe execution

---

## 5. ↕ Sorting and Distinctness

* `order_by: Vec<(String, bool)>`: field + direction (true = ASC, false = DESC)
* `distinct: bool`: enables `SELECT DISTINCT`

---

## 6. 📚 Pagination

* `limit: Option<usize>` restricts result rows
* `offset: Option<usize>` enables paging behavior (e.g. frontend scrolling)

---

## 7. 📎 Field & Type Mapping

* `field_table_map: HashMap<String, String>`
  → Maps logical field names to fully qualified ones like `table.field`
* `field_type_map: HashMap<String, String>`
  → Provides inferred types for validation, e.g., `"age" → \"int\"`

---

## 8. ✅ Query Validation (`validate()` method)

Ensures consistency, safety, and semantic correctness:

* Verifies:

  * Alias naming legality (SQL identifier)
  * Alias uniqueness
  * Function validity (`COUNT`, `SUM`, etc.)
* Validates:

  * Field and table names (against illegal characters)
  * Field presence in `field_table_map`
  * That each field belongs to the correct table when mapped
  * `HAVING` clause references must match aliases or aggregated fields
  * `GROUP BY`, `ORDER BY`, and `HAVING` fields must be resolvable
* Optional checks:

  * Type inference validation (e.g., `AVG(bool)` is disallowed)
  * Ensure that GROUP BY and SELECT fields are consistent

---

## 9. 🛠 SQL Construction APIs

Available interfaces:

* `to_sql()`
  → Outputs full SQL string (adds field prefixing if `field_table_map` is set)

* `to_sql_with_params()`
  → Outputs parameterized SQL and a list of parameter values for safe execution

* `to_explain_sql()`
  → Outputs `EXPLAIN SELECT ...` queries for inspection/debugging
