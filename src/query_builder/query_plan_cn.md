# 🧠 QueryPlan Builder 功能需求文档（修订版）

## 📌 项目目标

构建一个结构化、安全、可组合的 SQL 查询构造器，支持面向多来源（如 Parquet 文件）的动态查询、聚合分析和前端可视化。系统将具备以下核心能力：

* 字段别名管理与合法性校验
* 聚合函数支持与字段类型推断
* 参数化 SQL 安全构建，防止 SQL 注入
* 多表 JOIN 查询支持，自动字段表名映射
* 查询语义合法性验证

---

## 1. 🧾 字段选择（SELECT 子句）

* 支持字段表达形式：

  ```rust
  SelectExpr::Field(String)
  ```

* 支持聚合函数表达形式：

  ```rust
  SelectExpr::AggFunc {
      func: String,         // 如 COUNT, SUM, AVG
      field: String,        // 字段名或 "*"
      alias: Option<String> // 可选别名
  }
  ```

* 自动生成别名：当 `alias` 缺失时自动生成，规则为 `func_field`（如 `count_all`）

* 校验别名合法性：必须匹配 SQL 标识符规则

* 校验别名唯一性：同一查询中不能重复

* 校验字段名合法性：不允许包含非法字符，如 `.`（仅用于分表字段映射）

---

## 2. 🔗 多表关联（JOIN）

* 支持结构化 JOIN 条件表达：

  ```rust
  JoinOn::Expr {
      left_table: String,
      left_field: String,
      right_table: String,
      right_field: String
  }
  ```

* 支持多个 JOIN 子句

* JOIN 字段映射通过 `field_table_map: HashMap<String, String>` 实现

  * 若设置该映射，字段 `gene_id` 会自动映射为 `gene.gene_id`
  * 若未设置则使用原始字段

* 校验项：

  * 表名和字段名必须为合法 SQL 标识符（不包含 `.` 等非法字符）
  * 映射字段和表必须存在于 `field_table_map` 中
  * 校验字段是否确实属于当前 JOIN 使用的表（避免跨表引用错误）

---

## 3. 🧱 条件过滤（WHERE 子句）

* 使用 `ComposeQuery` 表达树状 AND/OR 查询条件
* 支持以下操作符：=, !=, <, >, <=, >=, in, not in, is, is not, like, ilike 等
* 支持嵌套复合条件构建
* 参数化构建：`to_sql_with_params()` 支持生成安全的 SQL 与绑定参数，避免注入攻击

---

## 4. 📊 分组聚合（GROUP BY & HAVING）

* `group_by: Vec<String>`：分组字段支持
* `having: Option<HavingClause>`：结构化表达 HAVING 子句

  * 聚合表达式支持如 `COUNT(*) > 5`
  * 支持嵌套组合：如 `(COUNT(*) > 5 AND AVG(x) > 2) OR SUM(y) < 10`
* `HavingExpr` 结构支持聚合函数和别名引用
* 支持参数化 `HAVING` 构建：自动提取参数值

---

## 5. ↕ 排序与去重

* `order_by: Vec<(String, bool)>`：支持多字段排序，true 为 ASC，false 为 DESC
* `distinct: bool`：支持 `SELECT DISTINCT` 查询语义

---

## 6. 📚 分页支持

* `limit: Option<usize>`：限制结果行数
* `offset: Option<usize>`：分页偏移支持（如前端分页）

---

## 7. 📎 字段映射与类型推断

* `field_table_map: HashMap<String, String>`：用于 SQL 生成时自动为字段添加前缀
* `field_type_map: HashMap<String, String>`：支持字段类型推断（如 `"age" → \"int\"`）

---

## 8. ✅ 查询计划验证 `validate()`

系统提供 `QueryPlan::validate()` 方法用于以下校验：

* 字段名、别名、表名合法性（不得包含非法字符）
* 聚合函数是否合法（如仅允许 COUNT/SUM/AVG/MIN/MAX）
* SELECT 中别名不能重复
* HAVING 中引用的别名必须出现在 SELECT 中
* group\_by、order\_by、having 中的字段必须：

  * 是合法 SQL 标识符
  * 存在于当前 SELECT 或字段映射中
  * 与实际查询的表/字段对应（参考 field\_table\_map）
* 额外建议的冲突检查：

  * group\_by 引用字段是否存在于 select 中
  * order\_by 字段是否存在于 select 或 group\_by 中
  * having 中字段是否为合法 alias 或聚合字段

---

## 9. 🛠 SQL 构建能力

支持以下 SQL 构建接口：

* `to_sql()`: 构建标准 SQL 字符串
* `to_sql_with_params()`: 构建参数化 SQL + 参数值 Vec
* `to_explain_sql()`: 构建 `EXPLAIN SELECT ...` 查询用于调试和分析

所有子句（SELECT, JOIN, WHERE, GROUP BY, HAVING, ORDER BY, LIMIT, OFFSET）均支持组合与合法性保障。

