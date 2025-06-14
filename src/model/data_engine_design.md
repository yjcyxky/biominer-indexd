# 📦 数据集系统功能规范文档（Dataset & Query Engine）

本规范文档整合了数据集管理与查询引擎两个核心模块，统一描述从 Parquet 文件的结构化组织、研究级元信息、数据表抽象，到多表联合查询执行的完整架构设计。

---

## 🧱 整体模块分层架构

| 层级/模块         | 主要职责                                                                                                               |
| ----------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `Dataset`         | 表示一个“数据集集合单元”，包含一个或多个数据表及其结构/文件路径等；注册多个表，维护字段映射关系与表关系、绑定到 DuckDB |
| `DatasetMetadata` | 表示“研究级别描述信息”，如来源、标签、文献信息等                                                                       |
| `MetadataTable`       | 表示数据文件关联的元数据表，在多组学数据场景下，本质上是 Clinical/Phenotype 表                                         |
| `DataFileTable`   | 单个 Parquet 表的 schema 抽象和文件路径描述                                                                            |
| `QueryPlan`       | 用户查询意图的结构化表达，支持字段校验、SQL 构建等                                                                     |
| `DuckDBEngine`    | 统一执行 SQL，返回结果 DataFrame（未来可替换执行引擎）                                                                 |

---

## 📁 数据目录规范（Data Layout）

### 顶层目录结构（`data_dir/`）

* `index.json`：所有数据集的研究级元信息索引，字段包括：`key`、`name`、`description`、`citation`、`pmid`、`groups`、`tags`、`total`、`is_filebased`、`metadata_table`, `datafile_tables`.

### 每个数据集子目录（`data_dir/{key}/`）

* `dataset.json`：数据集元信息（结构同 index.json）
* `metadata_dictionary.json`：字段结构定义（DataDictionary）
* `metadata_table.parquet`：结构化元数据表（MetadataTable，例如临床/表型数据）
* `datafiles`: [可选] 数据文件目录，包含多个数据文件，每个数据文件的文件名格式为 `{filename}.parquet`。如 `datafiles/expression_table.parquet`、`datafiles/expression_table_dictionary.json`、`datafiles/maf.parquet`、`datafiles/maf_dictionary.json`。
* `datafile.tsv`：数据文件信息表（存储数据文件基本信息，如 FASTQ 路径、md5 等）
* `dataset.tar.gz`（可选）：完整压缩包（如用于 cBioPortal）

---

## ✅ 模块职责与接口说明

### 🔹 `Dataset`：数据集抽象

```rust
#[derive(Debug, Deserialize, Serialize, Clone, Object)]
pub struct DatasetMetadata {
    pub key: String,
    pub name: String,
    pub description: String,
    pub citation: String,
    pub pmid: String,
    pub groups: Vec<String>,
    pub tags: Vec<String>,
    pub total: usize,
    pub is_filebased: bool,
}

#[derive(Debug, Clone)]
pub struct Dataset {
    pub metadata: DatasetMetadata,
    pub path: PathBuf,
    pub metadata_table: Box<dyn DataFileTable>,
    pub data_file_tables: Option<Vec<Box<dyn DataFileTable>>>,
}
```

### 🔹 `DataDictionary`：数据字典

```rust
#[derive(Debug, Deserialize, Serialize, Clone, Object)]
pub struct DataDictionaryField {
    pub key: String,
    pub name: String,
    pub data_type: String,
    pub description: String,
    pub notes: String,
    pub allowed_values: serde_json::Value, // It might be a list of strings, numbers, or booleans
    pub order: usize,
}

#[derive(Debug, Clone, Object)]
pub struct DataDictionary {
    pub fields: Vec<DataDictionaryField>,
}
```

### 🔹 `DataFileTable` Trait

```rust
trait DataFileTable {
    fn get_table_name(&self) -> &str;
    fn get_data_dictionary(&self) -> &DataDictionary;
    fn get_path(&self) -> PathBuf;
}
```

* 表示**单个 Parquet 表**的结构化描述，不负责执行查询
* 字段结构源于 `metadata_dictionary.json`

### 🔹 `Dataset`：数据集抽象

```rust
pub struct Dataset {
    pub metadata: DatasetMetadata,
    pub meta_table: Box<dyn DataFileTable>,         // data.parquet
    pub datafile_table: Option<Box<dyn DataFileTable>>, // datafile.tsv
}

impl Dataset {
    pub fn execute_query(&self, query: &QueryPlan) -> Result<DataFrame, Error> {
    }
}
```

* 表示一个数据集集合单元，包括：研究元信息 + 多个结构化表资源

#### 功能：

* 注册多个 datafile_tables 和 meta_table 到 DuckDB
* 提供字段表名映射、字段类型信息
* 构建 DuckDB 临时视图用于跨表 JOIN 查询
* 支持：`execute_query(plan: &QueryPlan)`

---

## 🔍 查询语义与计划（QueryPlan）

### 支持两类查询：

| 查询类型     | 示例 SQL                                        |
| ------------ | ----------------------------------------------- |
| 单表查询     | `SELECT * FROM clinical WHERE age > 60`         |
| 多表联合查询 | `SELECT * FROM clinical JOIN maf ON patient_id` |

---

## 🧠 QueryPlan 查询计划构建器

详见另附模块说明，支持：

* 字段选择（含聚合函数与自动别名）
* 条件过滤（WHERE）
* 分组与 HAVING 聚合筛选
* 排序、分页、DISTINCT 支持
* 多表 JOIN 与字段映射
* SQL 构建与参数化查询（防注入）
* EXPLAIN 模式支持
* 字段类型与表名校验

---

## ⚙️ 支持的功能列表

### ✅ 数据集加载与缓存

* `Dataset::load()` 加载数据集包
* `init_cache()` 缓存字段定义与文件索引信息

### ✅ 数据集校验（validate）

* 校验字段命名是否合法
* 字段类型是否为 STRING / NUMBER / BOOLEAN
* 字典字段是否出现在 parquet 表中

### ✅ 查询能力

#### 📍 查询数据集索引（Dataset::search_index）

* 基于 DuckDB 查询 `index.json`
* 支持分页、排序、where 条件表达

#### 📍 查询单个数据表（Dataset::query_meta_table）

* 查询 metadata_table 表（read_parquet）
* 支持分页、排序、ComposeQuery

#### 📍 多表执行（Dataset::execute_query）

* 注册多个 Parquet 表 → DuckDB 临时表
* 构造 QueryPlan → validate → to_sql → 执行

### ✅ 其他功能

* `group_by(field)`：分组统计频率、占比
* `get_schema()`：获取字段定义信息
* `get_datafiles()`：获取原始数据文件列表

---

## 📂 技术依赖

* 🦆 DuckDB：SQL 查询与 read\_parquet 接口
* 📚 Serde：JSON 结构序列化/反序列化
* 🧪 Polars：用于字段结构验证（字段对齐）
