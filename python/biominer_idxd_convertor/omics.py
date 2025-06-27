import os
from pathlib import Path
import pandas as pd
import json
import re
import numpy as np
from .utils import normalize_column_name, replace_missing_values, title_case, deduplicate_column_names
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


def convert_omics_file(data_path: Path, meta_path: Path, output_dir: Path, skip: bool = False):
    """
    Read data_*.txt and meta_*.txt, output parquet, dictionary, and metadata.
    """
    prefix = data_path.stem.replace("data_", "")
    parquet_path = output_dir / f"{prefix}.parquet"
    dictionary_path = output_dir / f"{prefix}_dictionary.json"
    metadata_path = output_dir / f"{prefix}_metadata.json"

    if skip and parquet_path.exists() and dictionary_path.exists() and metadata_path.exists():
        print(f"🔍 Skipping {data_path}")
        return

    # 检查文件是否存在
    if not data_path.exists():
        print(f"❌ Data file not found: {data_path}")
        return
    
    if not meta_path.exists():
        print(f"❌ Metadata file not found: {meta_path}")
        return

    # 1. Read meta_*.txt
    try:
        meta = parse_meta_file(meta_path)
    except Exception as e:
        print(f"❌ Error reading metadata file {meta_path}: {e}")
        return
    # 2. Read data_*.txt
    try:
        print(f"🔍 Reading {data_path}")
        df = pd.read_csv(data_path, sep="\t", dtype=str)
    except pd.errors.EmptyDataError:
        # 处理空文件
        df = pd.DataFrame()
    except pd.errors.ParserError as e:
        print(f"❌ Error parsing file {data_path}: {e}")
        # 尝试从错误消息中提取行号
        error_msg = str(e)
        if "row" in error_msg:
            # 提取行号信息
            row_match = re.search(r'row (\d+)', error_msg)
            if row_match:
                row_num = row_match.group(1)
                print(f"   This usually indicates malformed data (e.g., unclosed quotes) around row {row_num}")
            else:
                print(f"   This usually indicates malformed data (e.g., unclosed quotes)")
        else:
            print(f"   This usually indicates malformed data (e.g., unclosed quotes)")
        print(f"   Please check the file for formatting issues and try again.")
        return
    except Exception as e:
        print(f"❌ Unexpected error reading file {data_path}: {e}")
        return

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
    if meta["genetic_alteration_type"] in format_fn_map:
        df = format_fn_map[meta["genetic_alteration_type"]](df)

    # Normalize column names
    df.columns = [normalize_column_name(c, lower=False) for c in df.columns]
    # Deduplicate column names
    df = deduplicate_column_names(df)

    # 5. dictionary output - after normalization and deduplication
    dictionary = build_dictionary(df)
    
    try:
        df.to_parquet(parquet_path, index=False)
        df.to_csv(output_dir / f"{prefix}.tsv", sep="\t", index=False)
    except Exception as e:
        print(f"❌ Error saving data files for {data_path}: {e}")
        return

    try:
        with open(dictionary_path, "w") as f:
            json.dump(dictionary, f, indent=2)
    except Exception as e:
        print(f"❌ Error saving dictionary file for {data_path}: {e}")
        return
        
    # 6. metadata output
    try:
        with open(metadata_path, "w") as f:
            json.dump(meta, f, indent=2)
    except Exception as e:
        print(f"❌ Error saving metadata file for {data_path}: {e}")
        return

    print(f"✅ Successfully processed {data_path}")


def convert_all_omics(study_dir: Path, output_dir: Path, skip: bool = False):
    """
    Main entry: batch process all omics files, output to output_dir/datafiles/
    """
    pairs = find_omics_files(study_dir)
    if not pairs:
        print(f"⚠️  No omics files found in {study_dir}")
        return
    
    print(f"🔍 Found {len(pairs)} omics file pairs to process")
    output_dir.mkdir(parents=True, exist_ok=True)
    
    success_count = 0
    error_count = 0
    
    for i, (data_path, meta_path, prefix) in enumerate(pairs, 1):
        print(f"\n📁 Processing {i}/{len(pairs)}: {data_path.name}")
        try:
            convert_omics_file(data_path, meta_path, output_dir, skip)
            success_count += 1
        except Exception as e:
            print(f"❌ Unexpected error processing {data_path}: {e}")
            error_count += 1
    
    print(f"\n📊 Processing complete:")
    if success_count > 0:
        print(f"   ✅ Successfully processed: {success_count} / {len(pairs)} files")

    if error_count > 0:
        print(f"   ❌ Errors: {error_count} / {len(pairs)} files")
