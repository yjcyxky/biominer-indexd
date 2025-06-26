import tarfile
import hashlib
import uuid
from pathlib import Path
from dataclasses import dataclass
from typing import Optional
import pandas as pd
import time

namespace_prefix = "biominer.fudan-pgx"

@dataclass
class DataFile:
    guid: str
    filename: str
    size: int
    created_at: int
    updated_at: int
    status: str
    baseid: str
    rev: str
    version: int
    uploader: str
    access: str
    acl: Optional[str]
    # Omit urls, hashes, aliases, tags, etc., to be added later

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

def generate_deterministic_guid(file_path) -> str:
    file_hash = get_file_hash(file_path, "sha1")
    namespace = uuid.NAMESPACE_URL
    return f"{namespace_prefix}/{uuid.uuid5(namespace, file_hash)}"

def md5sum_to_baseid(md5sum: str) -> str:
    namespace = uuid.NAMESPACE_URL
    return str(uuid.uuid5(namespace, md5sum))

def make_tarball(study_dir: Path, tarball_path: Path):
    """
    Pack the entire dataset directory into a tar.gz file.
    """
    tarball_path.parent.mkdir(parents=True, exist_ok=True)
    with tarfile.open(tarball_path, "w:gz") as tar:
        tar.add(study_dir, arcname=study_dir.name)

def make_datafile(tarball_path: Path, dataset_meta: dict) -> DataFile:
    """
    Generate a DataFile object.
    """
    md5sum = get_file_hash(tarball_path, "md5")
    filename = dataset_meta.get("key", "dataset") + ".tar.gz"
    guid = generate_deterministic_guid(tarball_path)
    size = tarball_path.stat().st_size
    created_at = int(tarball_path.stat().st_mtime)
    updated_at = created_at
    status = "pending"
    baseid = md5sum_to_baseid(md5sum)
    rev = guid[:8]
    version = 1
    uploader = dataset_meta.get("uploader", "BioMiner")
    access = "public"
    acl = None
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
    )

def make_datafile_tsv(datafile: DataFile, output_dir: Path) -> Path:
    """
    Generate a datafile.tsv file.
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
    df = pd.DataFrame([flat])
    df.to_csv(datafile_path, index=False, sep="\t")
    return datafile_path 