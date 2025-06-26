import os
from pathlib import Path
import pandas as pd
import json
import re
import numpy as np
from .utils import normalize_column_name, replace_missing_values, title_case
from .formatter import format_cna, format_mutation, format_sv, format_mrna_seq, format_methylation

white_list = [
    "data_cna*",
    "meta_cna*",
    "data_mutation*",
    "meta_mutation*",
    "data_mrna_seq*",
    "meta_mrna_seq*",
    "data_sv*",
    "meta_sv*",
    "data_methylation*",
    "meta_methylation*",
]

id_column_map = {
    "COPY_NUMBER_ALTERATION": "sample_id",
    "MUTATION_EXTENDED": "Tumor_Sample_Barcode",
    "STRUCTURAL_VARIANT": "Sample_Id",
    "MRNA_EXPRESSION": "sample_id",
    "METHYLATION": "sample_id",
}

format_fn_map = {
    "COPY_NUMBER_ALTERATION": format_cna,
    "MUTATION_EXTENDED": format_mutation,
    "STRUCTURAL_VARIANT": format_sv,
    "MRNA_EXPRESSION": format_mrna_seq,
    "METHYLATION": format_methylation,
}

def is_in_white_list(f: str):
    for item in white_list:
        if re.match(item, f):
            return True
    return False


def find_omics_files(study_dir: Path):
    """
    Traverse the directory, automatically identify all data_*.txt and meta_*.txt files, and return a list of pairs [(data_path, meta_path, prefix)].
    """
    data_files = {}
    meta_files = {}
    for f in os.listdir(study_dir):
        if re.match(r"data_(.+)\.txt$", f) and is_in_white_list(f):
            prefix = re.match(r"data_(.+)\.txt$", f).group(1)
            data_files[prefix] = study_dir / f
        elif re.match(r"meta_(.+)\.txt$", f) and is_in_white_list(f):
            prefix = re.match(r"meta_(.+)\.txt$", f).group(1)
            meta_files[prefix] = study_dir / f
    pairs = []
    for prefix in data_files:
        if prefix in meta_files:
            pairs.append((data_files[prefix], meta_files[prefix], prefix))
    return pairs


def infer_dtype(series):
    # 确保输入是 Series
    if isinstance(series, pd.DataFrame):
        if series.shape[1] == 1:
            series = series.iloc[:, 0]
        else:
            raise ValueError("infer_dtype expects a Series, not a DataFrame with multiple columns")
    
    # 处理空 Series 或全为 NaN 的 Series
    if len(series) == 0 or series.dropna().empty:
        return "STRING"

    # 获取非空值
    non_null_series = series.dropna()
    if len(non_null_series) == 0:
        return "STRING"

    try:
        pd.to_numeric(non_null_series)
        return "NUMBER"
    except Exception:
        if set(non_null_series.unique()) <= {
            "True",
            "False",
            "true",
            "false",
            "0",
            "1",
        }:
            return "BOOLEAN"
        return "STRING"


def build_dictionary(df: pd.DataFrame) -> list:
    fields = []
    for col in df.columns:
        # Get the Series from the DataFrame if it is a DataFrame with multiple columns, get the first column
        col_series = df[col] if isinstance(df[col], pd.Series) else df.iloc[:, df.columns.get_loc(col)]
        dtype = infer_dtype(col_series)
        allowed_values = col_series.dropna().unique().tolist()
        # Ensure the numeric type is serializable
        if dtype == "NUMBER":
            min_val = col_series.min()
            max_val = col_series.max()
            minmax = [
                float(min_val) if pd.notna(min_val) else None,
                float(max_val) if pd.notna(max_val) else None,
            ]
        else:
            minmax = []

        fields.append(
            {
                "key": normalize_column_name(col, lower=False),
                "name": col,
                "description": "",
                "data_type": dtype,
                "notes": "",
                "allowed_values": allowed_values if dtype != "NUMBER" else minmax,
                "order": 0,
            }
        )
    return fields


def parse_meta_file(meta_path: Path) -> dict:
    """
    Parse meta_*.txt file in key-value format.

    Args:
        meta_path (Path): Path to meta_*.txt file

    Returns:
        dict: Parsed metadata dictionary
    """
    new_metadata = {}
    metadata = {}
    with open(meta_path, "r") as f:
        for line in f:
            if ":" in line:
                key, val = line.strip().split(":", 1)
                metadata[key.strip()] = val.strip()

    new_metadata["title"] = metadata.get("stable_id", "").upper()
    new_metadata["description"] = metadata.get("profile_description", "")
    new_metadata["datatype"] = metadata.get("datatype", "")
    new_metadata["genetic_alteration_type"] = metadata.get("genetic_alteration_type", "")
    new_metadata["id_column_name"] = id_column_map.get(new_metadata["genetic_alteration_type"], "sample_id")
    return new_metadata


def convert_omics_file(data_path: Path, meta_path: Path, output_dir: Path):
    """
    Read data_*.txt and meta_*.txt, output parquet, dictionary, and metadata.
    """
    prefix = data_path.stem.replace("data_", "")
    # 1. Read meta_*.txt
    meta = parse_meta_file(meta_path)
    # 2. Read data_*.txt
    try:
        df = pd.read_csv(data_path, sep="\t", dtype=str)
    except pd.errors.EmptyDataError:
        # 处理空文件
        df = pd.DataFrame()

    if not df.empty:
        df = df.apply(lambda col: replace_missing_values(col), axis=0)
        # 3. Type inference
        for col in df.columns:
            dtype = infer_dtype(df[col])
            if dtype == "NUMBER":
                df[col] = pd.to_numeric(df[col], errors="coerce")
            elif dtype == "BOOLEAN":
                df[col] = df[col].map(
                    {
                        "True": True,
                        "true": True,
                        "1": True,
                        "False": False,
                        "false": False,
                        "0": False,
                    }
                )

    # 4. parquet output
    parquet_path = output_dir / f"{prefix}.parquet"
    if meta["genetic_alteration_type"] in format_fn_map:
        df = format_fn_map[meta["genetic_alteration_type"]](df)

    # 5. dictionary output
    dictionary = build_dictionary(df)

    df.columns = [normalize_column_name(c, lower=False) for c in df.columns]
    df.to_parquet(parquet_path, index=False)
    df.to_csv(output_dir / f"{prefix}.tsv", sep="\t", index=False)

    with open(output_dir / f"{prefix}_dictionary.json", "w") as f:
        json.dump(dictionary, f, indent=2)
    # 6. metadata output
    with open(output_dir / f"{prefix}_metadata.json", "w") as f:
        json.dump(meta, f, indent=2)


def convert_all_omics(study_dir: Path, output_dir: Path):
    """
    Main entry: batch process all omics files, output to output_dir/datafiles/
    """
    pairs = find_omics_files(study_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    for data_path, meta_path, prefix in pairs:
        convert_omics_file(data_path, meta_path, output_dir)
