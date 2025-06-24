import os
import json
import pandas as pd
from pathlib import Path
import click
import numpy as np
import requests
import shutil
import re
import tarfile
import uuid
import hashlib
from dataclasses import dataclass
from typing import Optional

namespace_prefix = "biominer.fudan-pgx"
oncotree_url = (
    "https://oncotree.mskcc.org/api/tumorTypes/tree/?&version=oncotree_latest_stable"
)
code_to_disease_mapping = {}
code_to_organ_mapping = {}


def build_mappings():
    global code_to_disease_mapping, code_to_organ_mapping
    print("Building mappings...")
    filepath = Path(__file__).parent
    if os.path.exists(filepath / "code_to_disease_mapping.json") and os.path.exists(
        filepath / "code_to_organ_mapping.json"
    ):
        with open(filepath / "code_to_disease_mapping.json", "r") as f:
            code_to_disease_mapping = json.load(f)
        with open(filepath / "code_to_organ_mapping.json", "r") as f:
            code_to_organ_mapping = json.load(f)
        return

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


@dataclass
class URL:
    url: str
    created_at: int
    status: str  # "pending" | "processing" | "validated" | "failed"
    uploader: str
    file: Optional[str]

@dataclass
class Hash:
    hash_type: str  # "md5" | "sha1" | "sha256" | "sha512" | "crc32" | "crc64" | "etag"
    hash: str
    file: Optional[str]
@dataclass
class Alias:
    name: str
    file: Optional[str]

@dataclass
class Tag:
    field_name: str
    field_value: str
    file: Optional[str]

@dataclass
class DataFile:
    guid: str
    filename: str
    size: int
    created_at: int
    updated_at: int
    status: str  # "pending" | "processing" | "validated" | "failed"
    baseid: str
    rev: str
    version: int
    uploader: str
    access: str  # public or private
    acl: Optional[str]
    urls: Optional[list[URL]]
    hashes: Optional[list[Hash]]
    aliases: Optional[list[Alias]]
    tags: Optional[list[Tag]]


# Step 1: 读取文件内容并计算哈希
def get_file_hash(path, hash_type: str = "sha1") -> str:
    with open(path, "rb") as f:
        file_data = f.read()
    if hash_type == "sha1":
        return hashlib.sha1(file_data).hexdigest()
    elif hash_type == "sha256":
        return hashlib.sha256(file_data).hexdigest()
    elif hash_type == "sha512":
        return hashlib.sha512(file_data).hexdigest()
    elif hash_type == "md5":
        return hashlib.md5(file_data).hexdigest()
    else:
        raise ValueError(f"Invalid hash type: {hash_type}")


# Step 2: 用 UUIDv5 生成 GUID
def generate_deterministic_guid(file_path) -> str:
    file_hash = get_file_hash(file_path, "sha1")
    namespace = uuid.NAMESPACE_URL
    return f"{namespace_prefix}/{uuid.uuid5(namespace, file_hash)}"


def md5sum_to_baseid(md5sum: str) -> str:
    """
    将 MD5 值转换为 UUIDv5 格式，用作 baseid。
    返回值为字符串，如：'c7c7d092-32b0-486c-aff2-7282416419ff'
    """
    namespace = uuid.NAMESPACE_URL  # 可选其他命名空间，例如 NAMESPACE_DNS
    return str(uuid.uuid5(namespace, md5sum))


def make_tarball(study_dir, tarball_path):
    """
    Make a tarball of the dataset.
    """
    study_dir = Path(study_dir)
    tarball_path.parent.mkdir(parents=True, exist_ok=True)

    with tarfile.open(tarball_path, "w:gz") as tar:
        tar.add(study_dir, arcname=os.path.basename(study_dir))


def make_datafile(tarball_path, dataset_meta: dict) -> DataFile:
    """
    Make a datafile.tsv file for the dataset.
    """
    md5sum = get_file_hash(tarball_path, "md5")
    print(f"✅ MD5 sum: {md5sum}")
    filename = dataset_meta.get("key") + ".tar.gz"
    guid = generate_deterministic_guid(tarball_path)
    print(f"✅ GUID: {guid}")
    size = tarball_path.stat().st_size
    created_at = int(tarball_path.stat().st_mtime)
    updated_at = created_at
    status = "pending"
    baseid = md5sum_to_baseid(md5sum)
    print(f"✅ BaseID: {baseid}")
    rev = guid[:8]
    version = 1
    uploader = "Jingcheng Yang"
    access = "public"
    acl = None
    # TODO: The upper case of the filename might cause issues
    urls = [
        URL(
            url=f"minio://processed-data/OmicsDatasets/{filename}",
            created_at=created_at,
            status="pending",
            uploader=uploader,
            file=guid,
        )
    ]
    hashes = [
        Hash(hash_type="md5", hash=md5sum, file=guid)
    ]
    aliases = []
    tags = []

    return DataFile(
        guid=guid,
        filename=filename,
        size=size,
        created_at=created_at,
        updated_at=updated_at,
        status=status,
        baseid=baseid,
        rev=rev,
        version=version,
        uploader=uploader,
        access=access,
        acl=acl,
        urls=urls,
        hashes=hashes,
        aliases=aliases,
        tags=tags,
    )


def make_datafile_tsv(datafile: DataFile, output_dir: Path) -> Path:
    """
    Make a datafile.tsv file for the dataset.
    """
    datafile_path = output_dir / "datafile.tsv"

    flat = {
        "guid": datafile.guid,
        "filename": datafile.filename,
        "size": str(datafile.size),
        "created_at": str(datafile.created_at),
        "updated_at": str(datafile.updated_at),
        "status": datafile.status,
        "baseid": datafile.baseid,
        "rev": datafile.rev,
        "version": str(datafile.version),
        "uploader": datafile.uploader,
        "access": datafile.access or "",
        "acl": datafile.acl or "",
    }

    # 扁平化 URL 列表
    if datafile.urls:
        for i, url in enumerate(datafile.urls):
            flat[f"url_{i}_url"] = url.url
            flat[f"url_{i}_created_at"] = str(url.created_at)
            flat[f"url_{i}_status"] = url.status
            flat[f"url_{i}_uploader"] = url.uploader

    # 扁平化 Hash 列表
    if datafile.hashes:
        for i, h in enumerate(datafile.hashes):
            flat[f"hash_{i}_hash_type"] = h.hash_type
            flat[f"hash_{i}_hash"] = h.hash

    # 扁平化 Alias 列表
    if datafile.aliases:
        for i, a in enumerate(datafile.aliases):
            flat[f"alias_{i}_name"] = a.name

    # 扁平化 Tag 列表
    if datafile.tags:
        for i, t in enumerate(datafile.tags):
            flat[f"tag_{i}_field_name"] = t.field_name
            flat[f"tag_{i}_field_value"] = t.field_value

    df = pd.DataFrame([flat])
    df.to_csv(datafile_path, index=False, sep="\t")
    return datafile_path


def parse_meta_study(meta_path, organization=None):
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
    disease = code_to_disease_mapping.get(metadata.get("type_of_cancer").upper())
    if disease:
        tags.append(f"disease:{disease}")

    organ = code_to_organ_mapping.get(metadata.get("type_of_cancer").upper())
    if organ:
        tags.append(f"organ:{organ}")

    if organization:
        tags.append(f"org:{organization}")
    else:
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
        data_type = (
            type_row[col_key].strip().upper() if col_key in type_row else "STRING"
        )
        data_type = (
            data_type if data_type in ("STRING", "NUMBER", "BOOLEAN") else "STRING"
        )

        allowed_values = df[col_key].dropna().unique().tolist()
        # if len(allowed_values) > 100:
        #     allowed_values = []
        
        def min_max_value(col_key):
            if data_type == "NUMBER":
                return [df[col_key].min(), df[col_key].max()]
            else:
                return []

        fields.append(
            {
                "key": col_key,
                "name": name_row[col_key],
                "description": desc_row[col_key],
                "data_type": data_type,
                "notes": "",
                "allowed_values": allowed_values if data_type != "NUMBER" else min_max_value(col_key),
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


def convert_cbioportal_study(study_dir, output_dir, organization, version="v0.0.1") -> Path:
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
    output_dir = output_dir / version
    output_dir.mkdir(parents=True, exist_ok=True)

    # Parse metadata
    meta_path = study_dir / "meta_study.txt"
    if not meta_path.exists():
        raise FileNotFoundError("meta_study.txt not found")

    dataset_meta = parse_meta_study(meta_path, organization)
    
    key = dataset_meta.get("key")
    dirname = study_dir.name
    
    if key != dirname:
        raise ValueError(f"The key in meta_study.txt ({key}) does not match the directory name ({dirname})")

    # Load and merge clinical sample and patient data
    clinical_files = [
        "data_clinical_sample.txt",
        "data_clinical_patient.txt",
        "data_clinical_patient.tsv",
        "data_clinical_sample.tsv",
    ]
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
    dataset_meta["version"] = version
    dataset_meta["license"] = ""

    # Save Parquet
    combined_df.to_parquet(output_dir / "metadata_table.parquet", index=False)
    print(f"✅ Data saved to {output_dir / 'metadata_table.parquet'}")

    # Save data_dictionary.json using header info if available
    dictionary = build_data_dictionary_from_header(combined_df, headers)
    with open(output_dir / "metadata_dictionary.json", "w") as f:
        json.dump(dictionary, f, indent=2)

    print(f"✅ Data dictionary saved to {output_dir / 'metadata_dictionary.json'}")

    # Save dataset metadata
    with open(output_dir / "dataset.json", "w") as f:
        json.dump(dataset_meta, f, indent=2)

    print(f"✅ Dataset metadata saved to {output_dir / 'dataset.json'}")

    tarball_path = study_dir.parent / "datasets" / f'{dataset_meta.get("key")}.tar.gz'
    if not tarball_path.exists():
        make_tarball(study_dir, tarball_path)
        print(f"✅ Tarball saved to {tarball_path}")

    datafile = make_datafile(tarball_path, dataset_meta)
    datafile_path = make_datafile_tsv(datafile, output_dir)
    print(f"✅ Datafile saved to {datafile_path}")
    
    # Check if the README.md and LICENSE.md exist
    if not (output_dir / "README.md").exists():
        print(f"⚠️ README.md not found, creating a dummy one")
        (output_dir / "README.md").touch()

    if not (output_dir / "LICENSE.md").exists():
        print(f"⚠️ LICENSE.md not found, creating a dummy one")
        (output_dir / "LICENSE.md").touch()

    print(f"✅ Converted study saved to {output_dir}")
    
    return output_dir


@click.command()
@click.argument("study_dir", type=click.Path(exists=True, file_okay=False))
@click.argument("output_dir", type=click.Path())
@click.option("--organization", type=str, default="Unassigned")
@click.option("--version", type=str, default="v0.0.1")
def cli(study_dir, output_dir, organization, version):
    """
    CLI entry point to convert a cBioPortal dataset.

    STUDY_DIR is the path to a cBioPortal-format study folder.
    OUTPUT_DIR is the output directory to save data.parquet, data_dictionary.json, dataset.json.
    """
    build_mappings()
    
    try:
        output_dir = convert_cbioportal_study(study_dir, output_dir, organization, version)
    except Exception as e:
        print(f"⚠️ Failed to convert the dataset: {e}\n")

    # Check if the dataset is valid
    dataset_dir = Path(output_dir)
    if not dataset_dir.exists():
        print(f"⚠️ The dataset is invalid: {dataset_dir}")
        return
    
    if not (dataset_dir / "metadata_table.parquet").exists():
        # Delete the dataset directory
        shutil.rmtree(dataset_dir)
        print(f"⚠️ The dataset is invalid: {dataset_dir}")
        return

    if not (dataset_dir / "metadata_dictionary.json").exists():
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
