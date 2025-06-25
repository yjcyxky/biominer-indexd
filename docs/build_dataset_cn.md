# Biominer 数据集导入指南

Biominer 系统支持用户导入自定义的数据集。本文将通过**图文并茂**的教程形式，指导从未使用过该系统的新用户如何准备数据并导入 Biominer 数据集索引。

## 准备数据文件

在导入数据集之前，需要准备 **两个文件**：`dataset.txt`（数据集描述文件）和 `metadata_table.tsv`（元数据表文件）。这两个文件定义了数据集的基本信息和样本/临床数据，格式需要符合系统要求。

### 数据集描述文件 `dataset.txt`

`dataset.txt` 用于描述数据集的基本元信息。它是一个简单的文本文件，包含多行“键: 值”形式的条目。**必须**包含以下字段，且顺序和名称不可更改：

* **key**：数据集标识符（唯一键）。建议使用英文字母、数字或下划线组成的简短名称，作为数据集的“代号”。这个名称将用于数据文件夹命名和系统索引。
* **name**：数据集名称。可写成长描述名称，如研究课题标题，供界面显示。
* **description**：数据集详细描述。可包括该数据集的来源、内容简介、重要发现等（可较长）。尽量在同一行填写完整描述内容。
* **citation**：数据引用信息。如有论文发表，可填写引用格式或作者等；没有可留空。
* **pmid**：如果有对应论文的 PubMed ID，填写其 PMID；否则留空。
* **groups**：数据集分组标签。例如 PUBLIC（公开数据）、内部代号等。可填多个，用分号`;`隔开。
* **tags**：数据集标签。可用于描述组织、疾病等属性，例如 `org:FUSCC; disease:Triple-Negative Breast Cancer; organ:Breast;`。多个标签用分号隔开，可以采用`key:value`形式或自由格式。
* **total**：样本总数或记录总数。应与元数据表中的记录条数一致。
* **is\_filebased**：是否为文件型数据集。通常如果数据集仅包含表格型的元数据（如临床信息），则设为 `false`；如果数据集包含独立的文件（如测序数据文件），可设为 `true`（但请遵循系统对文件数据集的特殊配置）。
* **version**：数据集版本号。可自由命名，例如 `v0.0.1` 表示初始版本。若以后对数据集进行更新，可递增版本号（如 `v0.0.2` 或 `v1.0` 等）。
* **license**：数据集许可证信息。如果有特定的版权或许可证，填写其名称或URL；没有则留空。

下面是一个示例 `dataset.txt`（为简洁仅摘录部分内容）：

```text
key: fuscc_tnbc_465  
name: Chinese Triple-Negative Breast Cancer Cohort (FUSCC, Cancer Cell, 2019)  
description: This dataset contains a comprehensive multi-dimensional characterization of 465 primary TNBC samples ... （详细描述略）  
citation: Yizhou Jiang et al. Cancer Cell 2019  
pmid: 30853353  
groups: PUBLIC; FUSCC;  
tags: org:FUSCC; disease:Triple-Negative Breast Cancer; organ:Breast;  
total: 465  
is_filebased: false  
version: v0.0.1  
license:  
```

请确保**不要添加、删除或更改上述任何字段名**，即使某些字段暂时没有内容也请保留空值。特别地，`license` 没有信息时也需保留该行（如上例所示为空值）。另外，`key` 字段的值将被用作数据集文件夹的名字，为避免问题请使用**字母开头**且仅包含字母、数字和下划线的命名方式（遵循程序变量命名规则）。**重要提示**：`dataset.txt` 文件所在目录的名称必须与其中的 `key` 值相同。例如，上例中 `key: fuscc_tnbc_465`，则存放该文件的文件夹应命名为 “`fuscc_tnbc_465`”，否则后续转换工具会报错。

### 元数据表文件 `metadata_table.tsv`

`metadata_table.tsv` 是一个以制表符分隔 (`TSV`) 的表格文件，包含数据集中每个样本/病例的属性信息（临床信息等）。该文件格式参考了 cBioPortal 临床数据文件规范，需要**包含四行特殊表头**和随后的数据内容。

* **前四行：列注释行（需以 `#` 开头）** – 依次定义各列的元数据：

  1. **Name 行**（第1行，`#`开头）：每个数据列的人类可读名称。例如`Patient Identifier`、`Age at Diagnosis`等。
  2. **Description 行**（第2行，`#`开头）：对每个数据列的描述说明。例如`Identifier to uniquely specify a patient.`（用于唯一识别患者的标识）等。
  3. **Type 行**（第3行，`#`开头）：每列的数据类型。必须是`STRING`（字符串）, `NUMBER`（数值）或`BOOLEAN`（布尔）之一。
  4. **Order 行**（第4行，`#`开头）：每列的排序或重要性标识。可使用整数表示显示顺序，或使用二进制标记（如`1`表示主要字段，`0`表示次要字段）。具体约定视数据需要，可参考示例。

* **数据表头行**（第5行**不以#开头**）：实际数据表的列键名（字段名）。这一行定义了每列的字段标识符，一般使用大写字母和下划线组合。例如`PATIENT_ID`、`AGE_AT_DIAGNOSIS`等。系统会自动将这些键名规范化为小写并用下划线替换非法字符（只能包含字母、数字和下划线且以字母开头），建议直接采用符合规范的名称以避免自动修改。

* **数据行**（第6行及之后）：逐行列出每个样本的具体数值，按顺序对应各列。每行代表一个样本或病例。

一个简化的 `metadata_table.tsv` 示例（仅展示部分列）：

```text
#Patient Identifier    Age at Diagnosis    Gender    ...  
#每个患者的唯一识别码         诊断时年龄            性别       ...  
#STRING                NUMBER             STRING    ...  
#1                     1                  1         ...  
PATIENT_ID             AGE_AT_DIAGNOSIS   SEX       ...  
FUSCCTNBC001           63                 Female    ...  
FUSCCTNBC002           34                 Female    ...  
...                    ...                ...       ...  
```

在上述示例中：

* 第1行是 **Name 行**：定义了列的名称，如“Patient Identifier”、“Age at Diagnosis”、“Gender”等。
* 第2行是 **Description 行**：提供每列的解释说明（此示例中用中文描述，如“诊断时年龄”等）。
* 第3行是 **Type 行**：指定数据类型，如字符串`STRING`、数字`NUMBER`。
* 第4行是 **Order 行**：此处用`1`标记主要字段。实际使用中，可以根据需要使用序号或布尔标记。
* 第5行开始没有`#`的是数据表的字段名，如`PATIENT_ID`、`AGE_AT_DIAGNOSIS` 等。
* 第6行及以下是数据，每行对应一个样本（如`FUSCCTNBC001`、`FUSCCTNBC002`等）及其属性值。

**注意事项**：`metadata_table.tsv` **必须**包含上述四行表头。前四行的列数（字段数量）必须与后续数据列一一对应。如果您的数据有额外的字段，可以在第5行及后续行自由添加新列，但前四行也必须相应增加占位符并注明其 Name/Description/Type/Order。第一列通常是样本ID或病例ID，用于唯一标识记录，建议将其标记为主要字段（Order值为1）。您可以根据需要添加任意数量的字段列，**但请确保前四行的格式和顺序严格遵守上述要求**。系统将根据这四行注释来生成数据字典和进行数据验证。

## 使用转换脚本生成数据集文件

准备好 `dataset.txt` 和 `metadata_table.tsv` 文件后，需要使用仓库提供的转换脚本将其转换为系统所需的标准格式文件。脚本位于项目仓库的 [`examples/build_dataset.py`](https://github.com/yjcyxky/biominer-indexd/blob/main/examples/build_dataset.py) 中。

### 转换脚本说明

`build_dataset.py` 脚本会执行以下操作：

* 读取并解析 `dataset.txt`，生成标准化的数据集元信息文件 `dataset.json`。
* 合并并转换 `metadata_table.tsv` 为 Parquet 格式的 `metadata_table.parquet`，并依据表头注释生成 `metadata_dictionary.json`（数据字典）。
* 自动计算数据集记录总数、文件校验和等信息，并打包原始数据集文件夹为压缩包（`.tar.gz`，可选）。
* 生成数据文件索引 `datafile.tsv`，记录数据文件的摘要信息（例如文件大小、MD5等）。
* 确保生成数据集中包含必要的文件结构（如缺少 README.md 或 LICENSE.md 时会创建空的占位文件）。

### 运行转换脚本

使用 Python 运行该脚本进行转换。基本用法为：

```bash
python examples/build_dataset.py convert <原始数据集目录> <输出目录> --version <版本号>
```

其中：

* `<原始数据集目录>` 指包含您准备好的 `dataset.txt` 和 `metadata_table.tsv` 文件的文件夹路径。该文件夹名应与 `dataset.txt` 中定义的 key 相同（前文已强调）。
* `<输出目录>` 指定转换后的文件输出路径。**建议**将其指向 Biominer 系统的数据集根目录下的新建文件夹（即数据集 key 命名的文件夹）。例如，若数据集 key 为 `fuscc_tnbc_465`，可以将输出目录指定为 `.../datasets/fuscc_tnbc_465`。脚本会在此目录下按照指定版本号创建子目录。
* `--version` 用于指定版本号（如 `v0.0.1`）。这会作为输出子目录名称。如果不指定该参数，脚本会使用 `dataset.txt` 中的版本号，默认为 `v0.0.1`。

**示例**：假设我们在 `/data/raw_datasets/fuscc_tnbc_465` 准备好了原始的 `dataset.txt` 和 `metadata_table.tsv`，希望将转换结果输出到 Biominer 的数据集目录 `/data/biominer/datasets/` 下。可执行命令：

```bash
# 确保当前在 biominer-indexd 仓库根目录下
python examples/build_dataset.py convert /data/raw_datasets/fuscc_tnbc_465 /data/biominer/datasets/fuscc_tnbc_465 --version v0.0.1
```

运行后，脚本会打印转换过程日志，包括计算的文件哈希（MD5）、生成的 GUID 等信息。例如：

```
✅ Converting /data/raw_datasets/fuscc_tnbc_465 to /data/biominer/datasets/fuscc_tnbc_465...
✅ MD5 sum: 5d41402abc4b2a76b9719d911017c592
✅ GUID: biominer.fudan-pgx/123e4567-e89b-12d3-a456-426614174000
✅ BaseID: c7c7d092-32b0-486c-aff2-7282416419ff
✅ Tarball saved to /data/raw_datasets/datasets/fuscc_tnbc_465.tar.gz
✅ Datafile saved to /data/biominer/datasets/fuscc_tnbc_465/v0.0.1/datafile.tsv
✅ Converted study saved to /data/biominer/datasets/fuscc_tnbc_465/v0.0.1
```

完成后，请前往输出目录查看生成的文件结构。

### 数据集目录结构

转换脚本成功执行后，会在指定输出目录下生成一个以版本号命名的文件夹，其中包含该数据集的所有标准化文件。 例如，上述示例将得到目录 `/data/biominer/datasets/fuscc_tnbc_465/v0.0.1/`，结构如下：

```
fuscc_tnbc_465/              # 数据集文件夹（名称为数据集 key）
└── v0.0.1/                  # 版本文件夹（名称为指定的版本号）
    ├── dataset.json         # 数据集元信息（由 dataset.txt 转换）
    ├── metadata_table.parquet   # 元数据表的 Parquet 格式文件
    ├── metadata_dictionary.json # 元数据表的字段字典（根据注释生成）
    ├── datafile.tsv         # 数据文件信息表（包含 Tar 包等文件摘要信息）
    ├── README.md            # 数据集自述文件（如果原始数据集提供则复制，否则为空文件）
    └── LICENSE.md           # 数据集许可证文件（如果提供则复制，否则为空文件）
```

上述文件说明：

* **dataset.json**：数据集的JSON格式元信息，内容对应 `dataset.txt` 中的各字段（如 key、name、description 等）。
* **metadata\_table.parquet**：Parquet格式的元数据表，包含所有样本的结构化信息。相较原始的 TSV 文本，有更高的读取效率。
* **metadata\_dictionary.json**：数据字典，描述元数据表中各字段的名称、类型、含义等。由 TSV 文件头四行注释自动生成，每个字段对应一条字典记录。
* **datafile.tsv**：数据文件列表及其元信息。对于通过脚本生成的 `.tar.gz` 文件，此 TSV 会记录其文件名、大小、哈希值等。如果您的数据集还包含其他类型的数据文件（如基因突变表、表达矩阵等），也会记录在此（当前教程情境下主要关注 `.tar.gz` 压缩包）。
* **README.md**：数据集说明文件。如果原始数据集文件夹中包含README，脚本会将其复制；否则会创建一个空的 README.md 供日后填写数据集的使用说明。
* **LICENSE.md**：数据集许可证文件。如果原始数据集提供了LICENSE则复制，否则创建空文件。

此外，脚本还在原始数据集目录的上一级生成了一个`datasets/<数据集Key>.tar.gz`压缩包（如上日志所示路径）。这是对原始输入文件的一个归档备份，可用于在其他环境中重现该数据集或与他人共享。该压缩包对应 **dataset.tar.gz**，属于可选文件，不影响数据集注册流程。您可以将其妥善保存或移动至备份位置。

完成以上步骤后，已获得符合 Biominer 系统要求的数据集文件结构。接下来需要将新数据集**注册到索引**中，使系统识别该数据集。

## 注册数据集（自动生成索引 index.json）

Biominer 使用 `index.json` 文件来维护所有已注册数据集的索引信息。该文件位于数据集根目录（通常为 `datasets/` 目录）下，系统启动时会自动读取并加载其中的所有数据集。

为了避免手动编辑 `index.json` 文件，推荐使用官方提供的命令行工具 `biominer-indexd-cli` 来**自动扫描和生成索引**。

### 使用命令行工具自动生成索引

在成功完成数据集格式转换并将文件放置到正确的 `datasets/<dataset_key>/<version>` 目录下后，可通过如下命令自动生成或更新 `index.json` 文件：

```bash
./biominer-indexd-cli index-datasets --datasets-dir datasets
```

该命令将会：

* 遍历指定的 `datasets/` 目录；
* 查找每个数据集版本下的 `dataset.json` 文件；
* 汇总所有数据集的元信息，并自动生成统一的 `index.json` 索引文件；
* 将结果输出到 `<datasets-dir>/index.json`。

**示例：**

假设你已将数据集转换输出到 `datasets/fuscc_tnbc_465/v0.0.1/`，只需在项目根目录运行以下命令即可完成注册：

```bash
./biominer-indexd-cli index-datasets --datasets-dir datasets
```

执行成功后，会在 `datasets/` 目录下生成或更新 `index.json` 文件，其内容为所有数据集的元信息数组，每个数据集一个条目。例如：

```json
[
  {
    "key": "fuscc_tnbc_465",
    "name": "...",
    "version": "v0.0.1",
    ...
  },
  ...
]
```

完成后，重启 Biominer Index 系统，使其读取最新的 `index.json`。系统启动时将验证所有数据集结构和必要文件是否完整，确保索引数据有效。一旦加载成功，你就可以在界面或 API 中看到新导入的数据集。

> 🛠️ **提示**：若 `index-datasets` 命令失败，请检查各数据集目录是否包含正确结构（如 `dataset.json` 文件、子目录名与版本匹配等），否则索引生成将跳过错误数据集。

---

## 新增数据集或版本的方法

当你需要**添加新的数据集**或**为已有数据集增加新版本**时，可重复使用上述流程并再次运行索引命令进行自动注册。

* **新增不同数据集**：为每个新数据集准备独立的 `dataset.txt` 和 `metadata_table.tsv`，通过转换脚本生成标准化结构，放入以数据集 `key` 命名的目录下，再运行 `index-datasets` 自动注册。
* **新增数据集版本**：若已有数据集需更新版本，准备新版本的原始数据（如修正后的 `metadata_table.tsv` 和 `dataset.txt`），通过 `--version` 参数调用转换脚本生成新版本目录（例如 `v0.0.2`），然后再次运行 `index-datasets` 自动将新版本加入索引中。

系统允许同一 `key` 存在多个版本，所有版本会同时出现在 `index.json` 中，并可按需选择版本进行访问。

> 📌 **注意事项**：
>
> * 每次添加或修改数据集后，都需要重新运行 `biominer-indexd-cli index-datasets` 来刷新索引。
> * 旧版本如不再需要，可手动删除对应版本目录，并再次执行索引命令以移除旧条目。

## 数据管理的最佳实践

为了更好地管理数据并确保 Biominer 系统顺利运行，建议遵循以下实践：

* **分离原始数据和生成文件**：将原始数据文件与系统使用的生成文件分开存放。您可以维护一个独立的原始数据集目录用于存放 `dataset.txt`、`metadata_table.tsv` 以及其他原始文件；转换生成的标准化文件则统一存放在 Biominer 的 `datasets` 目录下。 这种分离方便您在源数据更新时重新转换，并清晰区分原始资料和系统索引所需文件。
* **保持 `datasets` 目录结构清晰**：严格按照系统要求组织数据集目录。在 `datasets` 根目录下，每个数据集（key）占用一个文件夹，文件夹内按版本划分子目录，并存放对应版本的文件集。不要在错误的位置放置文件或混用目录名。例如，不要把不同数据集的文件混杂在同一文件夹，或不按照版本划分子目录。这将确保系统的索引检索到正确路径。
* **合理命名与校验**：命名数据集 key、版本、字段名时遵循规范（字母数字下划线）。脚本会自动规范化字段名为小写加下划线形式，但尽量在准备阶段就使用规范命名，以减少歧义。在将数据导入系统前，可使用脚本的输出日志或系统启动日志检查是否有字段命名或类型不被接受的情况。如果出现验证错误（如字段名不合法、数据类型不支持等），根据日志提示修改源数据再重试。
* **备份与迁移**：`datasets` 目录承载了所有已经注册的数据集内容以及索引（index.json）。在升级系统或更换部署环境时，建议**备份整个 datasets 文件夹**以及其中的 `index.json`。将其迁移到新环境后，确保路径设置正确，启动 Biominer Index 服务并加载该目录，即可恢复所有数据集索引。如果由于路径变化导致需要重新生成索引，可在新环境中重新运行转换脚本导入，或者使用提供的工具/命令重新扫描生成 `index.json`（高级用法，通常不需要）。总之，妥善保存 `datasets` 目录即可保证数据集的可移植性和安全性。

按照本指南准备和导入数据后，您的新数据集就会成功集成到 Biominer 系统中。您可以在系统界面上浏览该数据集，或通过 API 查询其中的数据。通过遵循规范的格式和流程，您可以不断为系统添加更多数据集或更新现有数据集版本，充分利用 Biominer 强大的数据索引与查询功能。祝您使用顺利！
