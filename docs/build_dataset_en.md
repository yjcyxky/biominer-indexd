# Biominer Dataset Import Guide

The Biominer system supports users in importing custom datasets. This tutorial provides a **step-by-step illustrated guide** for new users on how to prepare and register datasets into the Biominer index system.

## Prepare Your Data Files

Before importing a dataset, you need to prepare **two required files**: `dataset.txt` (dataset description) and `metadata_table.tsv` (metadata table). These files define the dataset's metadata and sample/clinical information and must follow strict formatting rules.

### Dataset Description File: `dataset.txt`

This file contains key-value pairs (one per line) that describe the dataset's metadata. **All the following fields are required**, and their **order and names must not be changed**:

* **key**: Unique dataset identifier. Suggested to use lowercase letters, numbers, or underscores. This will be used for folder naming and internal indexing.
* **name**: Human-readable dataset name.
* **description**: Detailed description of the dataset. Can include data source, scope, highlights, etc.
* **citation**: Publication or citation info. Leave empty if not applicable.
* **pmid**: PubMed ID if available; otherwise leave empty.
* **groups**: Group tags (e.g., `PUBLIC; FUSCC;`), separated by semicolons.
* **tags**: Arbitrary tags such as `org:FUSCC; disease:Triple-Negative Breast Cancer; organ:Breast;`.
* **total**: Total number of records or samples (should match metadata rows).
* **is\_filebased**: `true` if the dataset includes individual files; `false` for metadata-only datasets.
* **version**: Version identifier (e.g., `v0.0.1`).
* **license**: License or usage terms (URL or text); leave empty if none.

\*\*Example \*\***`dataset.txt`** (simplified):

```text
key: fuscc_tnbc_465  
name: Chinese Triple-Negative Breast Cancer Cohort (FUSCC, Cancer Cell, 2019)  
description: This dataset contains a comprehensive multi-dimensional characterization of 465 primary TNBC samples ...  
citation: Yizhou Jiang et al. Cancer Cell 2019  
pmid: 30853353
        
          
groups: PUBLIC; FUSCC;  
tags: org:FUSCC; disease:Triple-Negative Breast Cancer; organ:Breast;  
total: 465  
is_filebased: false  
version: v0.0.1  
license:  
```

> ⚠️ **Important**: The folder name containing `dataset.txt` **must exactly match the ****`key`**** value**.

### Metadata Table File: `metadata_table.tsv`

This is a tab-delimited file describing per-sample attributes. It follows a [cBioPortal-compatible TSV format](https://docs.cbioportal.org/file-formats/#clinical-data). The file must include **4 metadata header rows** and a **data section**:

1. `#` **Name Row**: Human-readable column names.
2. `#` **Description Row**: Explanation of each field.
3. `#` **Type Row**: One of `STRING`, `NUMBER`, or `BOOLEAN`.
4. `#` **Order Row**: Used for sorting or importance (e.g., `1` = primary).
5. **Header Row**: Column field identifiers (e.g., `PATIENT_ID`, `AGE_AT_DIAGNOSIS`, ...).
6. **Data Rows**: One row per sample or record.

**Example**:

```text
#Patient Identifier    Age at Diagnosis    Gender    ...  
#Unique ID             Age at Dx           Sex       ...  
#STRING                NUMBER              STRING    ...  
#1                     1                   1         ...  
PATIENT_ID             AGE_AT_DIAGNOSIS    SEX       ...  
FUSCCTNBC001           63                  Female    ...  
FUSCCTNBC002           34                  Female    ...  
```

> ✅ **Tip**: You may add additional fields beyond the default ones, but the **first four metadata rows must be aligned** with the column count and format.

## Run Conversion Script

Once you have both `dataset.txt` and `metadata_table.tsv`, use the official script to convert them into Biominer-ready files.

Script location: [`examples/build_dataset.py`](https://github.com/yjcyxky/biominer-indexd/blob/main/examples/build_dataset.py)

### Command Format

```bash
python examples/build_dataset.py convert <input_dir> <output_dir> --version <version>
```

* `<input_dir>`: Directory containing `dataset.txt` and `metadata_table.tsv`. Must match the `key`.
* `<output_dir>`: Output root (usually `datasets/<key>`).
* `--version`: Optional; overrides version from `dataset.txt`.

### Example

```bash
python examples/build_dataset.py convert /data/raw/fuscc_tnbc_465 /data/biominer/datasets/fuscc_tnbc_465 --version v0.0.1
```

### Output Directory Structure

After conversion, the tool creates:

```
fuscc_tnbc_465/
└── v0.0.1/
    ├── dataset.json
    ├── metadata_table.parquet
    ├── metadata_dictionary.json
    ├── datafile.tsv
    ├── README.md
    └── LICENSE.md
```

> 📦 A backup `.tar.gz` of the input is also created in the parent `datasets/` directory.

## Register Dataset (Auto-generate `index.json`)

Biominer uses an `index.json` to manage all datasets. Instead of editing it manually, use the CLI:

### Command:

```bash
./biominer-indexd-cli index-datasets --datasets-dir datasets
```

This will:

* Scan each `<key>/<version>/dataset.json`.
* Merge metadata into a unified `datasets/index.json`.
* Overwrite or create `index.json` with the complete list.

After running, restart the Biominer Index service to reload the updated index.

> ⚠️ **Tip**: If any dataset is malformed or missing files, it will be skipped. Check logs for details.

## Add New Dataset or Version

To add new datasets or versions:

* **New dataset**: Create new folder, prepare `dataset.txt` and `metadata_table.tsv`, run the conversion script, then re-run `index-datasets`.
* **New version**: Prepare updated files, use `--version` to convert, save in same dataset folder under new subdirectory (e.g., `v0.0.2`), then run `index-datasets` again.

> ✅ The system supports multiple versions per dataset.

## Best Practices

* **Keep raw and generated files separate**: Store raw `dataset.txt` and `metadata_table.tsv` outside the `datasets/` directory.
* **Follow naming conventions**: `key` must match folder name; use underscores and lowercase.
* **Structure properly**: `datasets/<key>/<version>/` is required.
* **Back up ****`datasets/`**** and \*\*\*\*`index.json`**: You can migrate or rebuild the index by re-running `index-datasets` if needed.

---

With this guide, your dataset will be correctly integrated into Biominer. You can now browse or query your data via the web UI or API. Follow the formatting rules and you can continually add or update datasets with ease.

Happy mining!
