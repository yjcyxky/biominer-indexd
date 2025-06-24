import os
import json
import re
import csv
from pathlib import Path
from typing import List, Tuple, Optional
from minio import Minio
from minio.error import S3Error
from urllib.parse import urlparse


MINIO_PREFIXES = ("minio://", "s3://")  # 可扩展其他协议前缀


def extract_minio_urls_from_tsv(tsv_path: Path) -> List[str]:
    urls = []
    with tsv_path.open("r", encoding="utf-8") as f:
        reader = csv.DictReader(f, delimiter="\t")
        for row in reader:
            for value in row.values():
                if isinstance(value, str) and value.startswith(MINIO_PREFIXES):
                    urls.append(value.strip())
    return urls


def check_url_uploaded(url: str) -> bool:
    """
    这里替换成你真正的逻辑，例如通过 MinIO client、HTTP HEAD 请求等验证是否文件存在。
    暂时 mock 为 True。
    """
    if url.startswith("minio://"):
        access_key = os.getenv("MINIO_ACCESS_KEY")
        secret_key = os.getenv("MINIO_SECRET_KEY")
        endpoint = os.getenv("MINIO_ENDPOINT")
        if not access_key or not secret_key or not endpoint:
            raise ValueError("MINIO_ACCESS_KEY, MINIO_SECRET_KEY and MINIO_ENDPOINT must be set")

        minio_client = Minio(
            endpoint=endpoint,
            access_key=access_key,
            secret_key=secret_key,
            secure=True,
        )

        parsed = urlparse(url)
        bucket = parsed.netloc
        object_path = parsed.path.lstrip("/")

        try:
            print(f"🔍 Checking {bucket} {object_path}")
            stat = minio_client.stat_object(bucket, object_path)
            etag = stat.etag.lower().strip('"')
            print(f"✅ {bucket} {object_path} exists, etag: {etag}")
            return True
        except S3Error as e:
            if e.code == "NoSuchKey" or e.code == "NoSuchObject":
                return False
            else:
                raise e
    else:
        return False

def validate_dataset_key(dataset_dir: Path) -> Optional[str]:
    dataset_json_path = dataset_dir / "dataset.json"
    if not dataset_json_path.exists():
        return f"❌ Missing dataset.json in {dataset_dir}"

    with dataset_json_path.open("r", encoding="utf-8") as f:
        try:
            data = json.load(f)
        except json.JSONDecodeError:
            return f"❌ Invalid JSON in {dataset_json_path}"

    expected_key = dataset_dir.name
    if data.get("key") != expected_key:
        return f"❌ Key mismatch in {dataset_dir}: found '{data.get('key')}', expected '{expected_key}'"
    return None


def validate_dataset(dataset_dir: Path) -> List[str]:
    errors: List[str] = []

    # 1. Validate dataset key
    key_error = validate_dataset_key(dataset_dir)
    if key_error:
        errors.append(key_error)

    # 2. Validate URLs in datafile.tsv
    tsv_path = dataset_dir / "datafile.tsv"
    if not tsv_path.exists():
        errors.append(f"❌ Missing datafile.tsv in {dataset_dir}")
    else:
        urls = extract_minio_urls_from_tsv(tsv_path)
        for url in urls:
            if not check_url_uploaded(url):
                errors.append(f"❌ File not uploaded: {url}")

    return errors


def validate_all_datasets(root_dir: Path) -> None:
    dataset_dirs = [d for d in root_dir.iterdir() if d.is_dir()]
    all_errors: List[Tuple[str, List[str]]] = []

    for dataset_dir in dataset_dirs:
        print(f"\nValidating dataset: {dataset_dir}")
        errors = validate_dataset(dataset_dir)
        if errors:
            all_errors.append((dataset_dir.name, errors))

    if not all_errors:
        print("✅ All datasets passed validation.")
    else:
        for dataset_name, errs in all_errors:
            print(f"\nDataset: {dataset_name}")
            for err in errs:
                print(f"  {err}")


if __name__ == "__main__":
    import sys

    if len(sys.argv) != 2:
        print("Usage: python validate_datasets.py <datasets_root_dir>")
        exit(1)

    root = Path(sys.argv[1])
    if not root.is_dir():
        print(f"❌ Provided path is not a directory: {root}")
        exit(1)

    validate_all_datasets(root)
