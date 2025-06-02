import os
import json
import pandas as pd
from pathlib import Path
import click
import numpy as np
import requests
import shutil
import re

oncotree_url = "https://oncotree.mskcc.org/api/tumorTypes/tree/?&version=oncotree_latest_stable"
code_to_disease_mapping = {}
code_to_organ_mapping = {}

def build_mappings():
    if os.path.exists("code_to_disease_mapping.json") and os.path.exists(
        "code_to_organ_mapping.json"
    ):
        with open("code_to_disease_mapping.json", "r") as f:
            code_to_disease_mapping = json.load(f)
        with open("code_to_organ_mapping.json", "r") as f:
            code_to_organ_mapping = json.load(f)
        return

    print("Building mappings...")

    def recurse(node):
        code = node.get("code")
        name = node.get("name")
        tissue = node.get("tissue")

        # 跳过根节点或中间节点（如 TISSUE 本身），只添加有组织归属的具体癌症类型
        if code and name and tissue:
            code_to_disease_mapping[code] = name
            code_to_organ_mapping[code] = tissue

        children = node.get("children", {})
        for child in children.values():
            recurse(child)

    response = requests.get(oncotree_url)
    data = response.json()
    # 从 TISSUE 根节点开始递归
    recurse(data["TISSUE"])


def parse_meta_study(meta_path):
    """
    Parse the cBioPortal `meta_study.txt` metadata file into a structured dictionary.

    Args:
        meta_path (Path): Path to the `meta_study.txt` file.

    Returns:
        dict: Dictionary containing dataset metadata fields such as key, name, description, etc.
    """
    metadata = {}
    with open(meta_path, "r") as f:
        for line in f:
            if ":" in line:
                key, val = line.strip().split(":", 1)
                metadata[key.strip()] = val.strip()

    tags = []
    disease = code_to_disease_mapping[metadata.get('type_of_cancer').upper()]
    if disease:
        tags.append(f"disease:{disease}")

    organ = code_to_organ_mapping[metadata.get('type_of_cancer').upper()]
    if organ:
        tags.append(f"organ:{organ}")

    tags.append("org:Unassigned")

    return {
        "key": metadata.get("cancer_study_identifier", "unknown"),
        "name": metadata.get("name", "Unnamed Study"),
        "description": metadata.get("description", ""),
        "citation": metadata.get("citation", ""),
        "pmid": metadata.get("pmid", ""),
        "groups": metadata.get("groups", "").split(";"),
        "tags": tags,
        "total": 0,
        "is_filebased": False,
    }

def build_data_dictionary_from_header(df, header_lines):
    """
    Build a data dictionary using annotated headers from clinical files.

    Args:
        df (pandas.DataFrame): The loaded clinical DataFrame.
        header_lines (List[List[str]]): List of header rows including name, description, type, and order.

    Returns:
        list: List of dictionaries defining each field's metadata.
    """
    fields = []
    name_row, desc_row, type_row, order_row = header_lines[:4]
    try:
        order_row = {key: int(value) for key, value in order_row.items()}
    except:
        pass

    for idx, col_key in enumerate(df.columns):
        data_type = type_row[col_key].strip().upper() if col_key in type_row else "STRING"
        data_type = (
            data_type if data_type in ("STRING", "NUMBER", "BOOLEAN") else "STRING"
        )

        allowed_values = df[col_key].dropna().unique().tolist()
        # if len(allowed_values) > 100:
        #     allowed_values = []

        fields.append(
            {
                "key": col_key,
                "name": name_row[col_key],
                "description": desc_row[col_key],
                "data_type": data_type,
                "notes": "",
                "allowed_values": allowed_values,
                "order": order_row[col_key],
            }
        )
    return fields


def normalize_column_name(col):
    # 替换所有非字母数字下划线的字符为下划线
    col = re.sub(r"\W", "_", col.strip())
    # 如果首字符不是字母或下划线，加前缀 "_"
    if not re.match(r"^[A-Za-z_]", col):
        col = "_" + col
    return col.lower()


def read_clinical_file(path):
    """
    Read a cBioPortal clinical file and extract its annotated headers and data.

    Args:
        path (Path): File path to the .txt file

    Returns:
        tuple: (DataFrame of content, list of header rows [name, desc, type, ...])
    """
    with open(path) as f:
        header_lines = []
        data_lines = []
        for line in f:
            if line.startswith("#"):
                header_lines.append(line[1:].strip().split("\t"))
            elif line.strip():
                data_lines.append(line.strip().split("\t"))

    if data_lines:
        original_columns = data_lines[0]
        normalized_columns = [normalize_column_name(col) for col in original_columns]
        df = pd.DataFrame(data_lines[1:], columns=normalized_columns)
    else:
        df = pd.DataFrame()
    return df, header_lines


def convert_cbioportal_study(study_dir, output_dir):
    """
    Convert a cBioPortal-formatted dataset folder into a normalized dataset format.

    This includes:
    - Reading meta_study.txt to create dataset.json
    - Combining data_clinical_sample.txt and data_clinical_patient.txt into data.parquet
    - Generating data_dictionary.json based on annotated headers

    Args:
        study_dir (str or Path): Path to the cBioPortal dataset directory.
        output_dir (str or Path): Destination directory to write the normalized dataset files.
    """
    study_dir = Path(study_dir)
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Parse metadata
    meta_path = study_dir / "meta_study.txt"
    if not meta_path.exists():
        raise FileNotFoundError("meta_study.txt not found")

    dataset_meta = parse_meta_study(meta_path)

    # Load and merge clinical sample and patient data
    clinical_files = ["data_clinical_sample.txt", "data_clinical_patient.txt", "data_clinical_patient.tsv", "data_clinical_sample.tsv"]
    dfs = []
    headers = []
    names = {}
    descs = {}
    types = {}
    orders = {}
    dtype_dict = {}
    for fname in clinical_files:
        fpath = study_dir / fname
        if fpath.exists():
            df, header = read_clinical_file(fpath)

            dtype_dict.update(dict(zip(df.columns, header[2])))
            names.update(dict(zip(df.columns, header[0])))
            descs.update(dict(zip(df.columns, header[1])))
            types.update(dict(zip(df.columns, header[2])))
            orders.update(dict(zip(df.columns, header[3])))
            dfs.append(df)

    print(dtype_dict)
    headers = [names, descs, types, orders]

    if not dfs:
        raise ValueError("No clinical files found")

    combined_df = pd.concat(dfs, axis=1)
    combined_df = combined_df.loc[:, ~combined_df.columns.duplicated()]

    type_mapping = {
        "NUMBER": "float64",
        "STRING": "string",
        "BOOLEAN": "boolean",
    }

    # 明确所有需要当作缺失值处理的值
    missing_values = {"NA", "N/A", "", "null", "NULL", "[Not Available]", "Na"}

    for col in combined_df.columns:
        dtype = dtype_dict.get(col, "STRING")
        target_dtype = type_mapping.get(dtype, "string")

        try:
            # 替换伪缺失值为 np.nan
            combined_df[col] = combined_df[col].replace(list(missing_values), np.nan)

            # 转换为目标类型
            combined_df[col] = combined_df[col].astype(target_dtype)

        except Exception as e:
            print(f"⚠️ Failed to convert column '{col}' to {dtype}: {e}")

    dataset_meta["total"] = len(combined_df)
    dataset_meta["is_filebased"] = False

    # Save Parquet
    combined_df.to_parquet(output_dir / "data.parquet", index=False)

    # Save data_dictionary.json using header info if available
    dictionary = build_data_dictionary_from_header(combined_df, headers)
    with open(output_dir / "data_dictionary.json", "w") as f:
        json.dump(dictionary, f, indent=2)

    # Save dataset metadata
    with open(output_dir / "dataset.json", "w") as f:
        json.dump(dataset_meta, f, indent=2)

    print(f"✅ Converted study saved to {output_dir}")


@click.command()
@click.argument("study_dir", type=click.Path(exists=True, file_okay=False))
@click.argument("output_dir", type=click.Path())
def cli(study_dir, output_dir):
    """
    CLI entry point to convert a cBioPortal dataset.

    STUDY_DIR is the path to a cBioPortal-format study folder.
    OUTPUT_DIR is the output directory to save data.parquet, data_dictionary.json, dataset.json.
    """
    build_mappings()
    
    try:
        convert_cbioportal_study(study_dir, output_dir)
    except Exception as e:
        print(f"⚠️ Failed to convert the dataset: {e}\n")
    
    # Check if the dataset is valid
    dataset_dir = Path(output_dir)
    if not dataset_dir.exists():
        print(f"⚠️ The dataset is invalid: {dataset_dir}")
        return
    
    if not (dataset_dir / "data.parquet").exists():
        # Delete the dataset directory
        shutil.rmtree(dataset_dir)
        print(f"⚠️ The dataset is invalid: {dataset_dir}")
        return
    
    if not (dataset_dir / "data_dictionary.json").exists():
        # Delete the dataset directory
        shutil.rmtree(dataset_dir)
        print(f"⚠️ The dataset is invalid: {dataset_dir}")
        return
    
    if not (dataset_dir / "dataset.json").exists():
        # Delete the dataset directory
        shutil.rmtree(dataset_dir)
        print(f"⚠️ The dataset is invalid: {dataset_dir}")
        return
    
    print(f"✅ The dataset is valid: {dataset_dir}\n")


if __name__ == "__main__":
    cli()
