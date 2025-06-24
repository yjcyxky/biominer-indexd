# 📦 数据集系统功能规范文档（Dataset & Query Engine）

本规范文档整合了数据集管理与查询引擎两个核心模块，统一描述从 Parquet 文件的结构化组织、研究级元信息、数据表抽象，到多表联合查询执行的完整架构设计。

---

## 🧱 整体模块分层架构

| 层级/模块         | 主要职责                                                                                                               |
| ----------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `Dataset`         | 表示一个"数据集集合单元"，包含一个或多个数据表及其结构/文件路径等；注册多个表，维护字段映射关系与表关系、绑定到 DuckDB |
| `DatasetMetadata` | 表示"研究级别描述信息"，如来源、标签、文献信息等                                                                       |
| `MetadataTable`       | 表示数据文件关联的元数据表，在多组学数据场景下，本质上是 Clinical/Phenotype 表                                         |
| `DataFileTable`   | 单个 Parquet 表的 schema 抽象和文件路径描述                                                                            |
| `QueryPlan`       | 用户查询意图的结构化表达，支持字段校验、SQL 构建等                                                                     |
| `DuckDBEngine`    | 统一执行 SQL，返回结果 DataFrame（未来可替换执行引擎）                                                                 |

---

## 📁 数据目录规范（Data Layout）

### 顶层目录结构（`data_dir/`）

* `index.json`：所有数据集的研究级元信息索引，字段包括：`key`、`name`、`description`、`citation`、`pmid`、`groups`、`tags`、`total`、`is_filebased`、`metadata_table`, `datafile_tables`.

### 每个数据集子目录（`data_dir/{key}/`）

* `README.md`：数据集的 README 文件，包含数据集的描述、使用方法、注意事项等。
* `LICENSE.md`：数据集的 LICENSE 文件，包含数据集的版权信息。[可选，只有当用户指定定义 LICENSE 时，才需要]
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
    pub version: String,  // The version of the dataset, like "v1.0.0"
    pub license: Option<String>, // The license of the dataset, like "CC-BY-4.0"
}

#[derive(Debug, Clone)]
pub struct Dataset {
    pub metadata: DatasetMetadata,
    pub path: PathBuf,
    pub metadata_table: MetadataTable,
    pub datafile_tables: HashMap<String, DataFileTable>,
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

### 🔹 `DataFileTable` 实现

```rust
#[derive(Debug, Clone)]
pub enum DataFileTable {
    MAF(MAFTable),
    MRNAExpr(MRNAExprTable),
}

#[derive(Debug, Clone)]
pub struct MetadataTable {
    pub table_name: &'static str,
    pub data_dictionary: DataDictionary,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct MAFTable {
    pub table_name: &'static str,
    pub data_dictionary: DataDictionary,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct MRNAExprTable {
    pub table_name: &'static str,
    pub data_dictionary: DataDictionary,
    pub path: PathBuf,
}
```

* 表示**单个 Parquet 表**的结构化描述，不负责执行查询
* 字段结构源于对应的 `*_dictionary.json` 文件
* 支持的表类型：
  - `MetadataTable`: 元数据表（临床/表型数据）
  - `MAFTable`: 突变注释文件表
  - `MRNAExprTable`: 基因表达量表

### 🔹 `File`：数据文件抽象

```rust
pub struct File {
    pub guid: String,
    pub filename: String,
    pub size: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub status: String, // "pending" | "processing" | "validated" | "failed"
    pub baseid: String, // The file with multiple versions will have the same baseid
    pub rev: String,
    pub version: i32,
    pub uploader: String,
    pub access: String, // public or private
    pub acl: Option<String>,
    pub urls: Option<serde_json::Value>,
    pub hashes: Option<serde_json::Value>,
    pub aliases: Option<serde_json::Value>,
    pub tags: Option<serde_json::Value>,
}
```

* 表示单个数据文件的元信息
* 支持文件版本管理（通过 `baseid` 和 `version`）
* 支持文件访问控制（通过 `access` 和 `acl`）
* 支持文件关联信息（URLs、哈希值、别名、标签）

### 🔹 `Dataset` 功能说明

* 数据集加载与缓存
  - `Dataset::load()`: 加载数据集包，包括元数据、数据字典和数据表
  - `init_cache()`: 缓存字段定义、数据集元信息和文件索引信息
  - 使用 `lazy_static` 实现全局缓存

* 数据集校验（validate）
  - 校验字段命名是否合法（`^[a-z][a-z0-9_]*$`）
  - 校验字段类型（STRING / NUMBER / BOOLEAN）
  - 校验字典字段是否出现在 parquet 表中
  - 校验必要文件是否存在（dataset.json, metadata_table.parquet, datafile.tsv）

* 查询能力
  - `Datasets::search()`: 基于 DuckDB 查询 `index.json`
  - `Dataset::search()`: 查询 metadata_table、maf、mrna_expr 等表，支持多表联合查询（通过 DuckDB 临时视图）
  - 支持分页、排序、条件过滤

* 数据文件管理
  - `File::from_file()`: 从 TSV 文件加载数据文件信息
  - `File::query_file()`: 查询单个文件信息
  - 支持文件元数据管理（URLs、哈希值、别名、标签）

---

## 🔍 查询语义与计划（QueryPlan）

### 支持两类查询：

| 查询类型     | 示例 SQL                                        |
| ------------ | ----------------------------------------------- |
| 单表查询     | `SELECT * FROM clinical WHERE age > 60`         |
| 多表联合查询 | `SELECT * FROM clinical JOIN maf ON patient_id` |

---

## 🧠 QueryPlan 查询计划构建器

支持：

* 字段选择（含聚合函数与自动别名）
* 条件过滤（WHERE）
* 分组与 HAVING 聚合筛选
* 排序、分页、DISTINCT 支持
* 多表 JOIN 与字段映射
* SQL 构建与参数化查询（防注入）
* EXPLAIN 模式支持
* 字段类型与表名校验

---

## 📂 技术依赖

* 🦆 DuckDB：SQL 查询与 read\_parquet 接口
* 📚 Serde：JSON 结构序列化/反序列化
* 🧪 Polars：用于字段结构验证（字段对齐）
