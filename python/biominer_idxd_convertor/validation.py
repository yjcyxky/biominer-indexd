from pathlib import Path

def validate_output_dir(output_dir: Path) -> bool:
    """
    Validate the output directory structure and all required files exist.
    """
    required_files = [
        "metadata_table.parquet",
        "metadata_dictionary.json",
        "dataset.json",
        "datafile.tsv",
    ]
    for fname in required_files:
        if not (output_dir / fname).exists():
            print(f"❌ Missing file: {fname}")
            return False
    datafiles_dir = output_dir / "datafiles"
    if not datafiles_dir.exists() or not datafiles_dir.is_dir():
        print("❌ Missing datafiles directory")
        return False
    parquet_files = list(datafiles_dir.glob("*.parquet"))
    if not parquet_files:
        print("❌ No parquet files in datafiles directory")
        return False
    print("✅ Output directory structure validated")
    return True 