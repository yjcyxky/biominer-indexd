# 📘 Convertor 需求文档

## 🧩 项目目标

将 cBioPortal 格式的数据集转换为标准化结构，支持下游 AI 驱动的生信分析流程。转换输出包括统一格式的 `dataset.json`、`metadata_dictionary.json`、`metadata_table.parquet`、`datafile.tsv`，并将所有文件打包为 `tar.gz`，生成完整 `DataFile` 对象。

## 🗂️ 数据集组成

### 1. Clinical 信息

包含以下文件（按需合并）：

* `data_clinical_sample.txt` / `.tsv`
* `data_clinical_patient.txt` / `.tsv`

### 2. meta 信息

* `meta_study.txt`: 用于生成 `dataset.json` 并提取研究标识符、癌种、组织、PMID 等。

### 3. Omics 数据文件

一个数据集可能包含多个 omics 数据文件（如 mutation、expression、cnv 等）。每组 omics 文件包含：

* `meta_*.txt`：描述对应 data 文件的元信息（如 title、id\_column\_name 等）
* `data_*.txt`：原始数据矩阵

## 📎 文件结构输出

```
<output_dir>/v0.0.1/
├── dataset.json
├── metadata_dictionary.json
├── metadata_table.parquet
├── datafile.tsv
├── README.md
├── LICENSE.md
└── datafiles/
    ├── <prefix>.parquet
    ├── <prefix>_dictionary.json
    ├── <prefix>_metadata.json
    └── ...
```

## 🔧 临床数据转换流程

* **文件解析与归一化**

  * 自动读取并解析 annotated header
  * 支持类型转换（NUMBER/STRING/BOOLEAN）
  * 替换伪缺失值（NA/N/A/null/NULL/\[Not Available]/Na）

* **数据合并与标准化**

  * 合并 sample/patient 数据，去除重复列
  * 转为 Parquet 格式：`metadata_table.parquet`

* **字典生成**

  * 基于注释构建 `metadata_dictionary.json`

* **元信息提取**

  * 从 `meta_study.txt` 中提取 dataset key、PMID、癌种等，构建 `dataset.json`

* **数据文件注册**

  * 打包整个dataset目录为tarball，并生成一个DataFile对象
  * 支持生成 deterministic GUID、baseid
  * 构建 `datafile.tsv` 和完整的 `DataFile` 对象
  * 输出打包为 tarball

## 🧬 Omics Data 文件支持

将任意数量的 omics 文件（如 mutation、expression、methylation）统一转换为标准格式：

* `<prefix>.parquet`
* `<prefix>_dictionary.json`
* `<prefix>_metadata.json`

其中 `<prefix>` 由原始 `data_*.txt` 去掉 `data_` 前缀后的部分确定，例如：

| 原始文件名           | 目标文件                    |
| -------------------- | --------------------------- |
| `data_mutations.txt` | `mutations.parquet`         |
| `meta_mutations.txt` | `mutations_metadata.json`   |
| （推断）             | `mutations_dictionary.json` |

### 📥 输入要求

* 每个 `data_*.txt` 对应一个 `meta_*.txt`（匹配后缀）
* `meta_*.txt` JSON 格式，包含字段：

  * `title`: 字段名称说明
  * `description`: 简要描述
  * `id_column_name`: 主键列（如 `Tumor_Sample_Barcode`）

### ⚙️ 处理逻辑

1. **自动识别所有 data/meta 文件**

   * 遍历目录，匹配 `data_*.txt` 和 `meta_*.txt` 配对

2. **读取与转换**

   * 读取 data\_\*.txt 为 DataFrame（header 自动推断）
   * `id_column_name` 设为索引列或 primary column
   * 转换为 Parquet 格式 `<prefix>.parquet`

3. **构建字段字典**

   * 分析每列数据类型与取值分布
   * 构建 `<prefix>_dictionary.json`，内容结构同已有 `metadata_dictionary.json`，字段包括：

     * key, name, description
     * data\_type (STRING/NUMBER/BOOLEAN)
     * allowed\_values, min/max（数值列）
   * 依赖现有辅助函数 `normalize_column_name` 与缺失值处理逻辑

4. **输出 metadata**

   * 拷贝 meta\_\*.txt 原文或提取核心字段输出 `<prefix>_metadata.json`

5. **输出路径**

   * 所有文件统一写入 `{output_dir}/vX.Y.Z/datafiles/` 目录下
   * 不放入子目录，而是统一命名为：

     * `{prefix}.parquet`
     * `{prefix}_dictionary.json`
     * `{prefix}_metadata.json`

---

## 🧪 验证标准

每个转换后的数据集目录必须包含：

* ✔️ metadata\_table.parquet
* ✔️ metadata\_dictionary.json
* ✔️ dataset.json
* ✔️ datafiles/ 目录下如果存在 omics 数据文件，则必须包含：
  * `<prefix>.parquet`
  * `<prefix>_dictionary.json`
  * `<prefix>_metadata.json`
* ✔️ 所有输出文件必须可被打包并生成合法 DataFile 信息

## 其他要求

* 必须使用 click 作为命令行工具
* 拆分为多个合适的模块，放入convertor目录下
* 使用Python实现，且实现为可安装的python包，具备命令行入口
* 测试覆盖率必须达到 100%
* 命令行入口包含两个：
  * biominer-idxd convert <study_dir> <output_dir> --organization <name> --version <v>
  * biominer-idxd bconvert <study_dir> <output_dir> --organization <name> --version <v>
* 以已有的代码cbioportal2dataset.py为基础，实现一个可用的命令行工具。其已经可以很好的工作，只是缺少对omics datafiles的转换。尽可能复用已有的代码，不要重复造轮子。