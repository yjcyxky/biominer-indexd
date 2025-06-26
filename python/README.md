# convertor

A Python package to convert cBioPortal-format datasets into a standardized structure for downstream AI-driven bioinformatics analysis. Outputs include unified `dataset.json`, `metadata_dictionary.json`, `metadata_table.parquet`, `datafile.tsv`, and omics files in Parquet/JSON format, all ready for packaging and DataFile object generation.

## Features
- Clinical and omics data conversion and normalization
- Automatic file structure and metadata generation
- CLI tool: `biominer-idxd convert` and `biominer-idxd bconvert`
- Modular, testable, and extensible Python package
- 100% test coverage

## Installation

```bash
pip install .
```

## Command Line Usage

```bash
biominer-idxd convert <study_dir> <output_dir> --organization <name> --version <v>
biominer-idxd bconvert <study_dir> <output_dir> --organization <name> --version <v>
```

- `<study_dir>`: Path to the cBioPortal-format study folder
- `<output_dir>`: Output directory for standardized files
- `--organization`: Organization name (default: Unassigned)
- `--version`: Output version (default: v0.0.1)

## Development & Testing

- All main logic is in the `convertor/` directory, split by module.
- To run all tests:

```bash
pytest --cov=convertor
```

- 100% test coverage is required. See `convertor/tests/` for examples.

## Project Structure

- `convertor/cli.py`: CLI entry (Click)
- `convertor/cbioportal2dataset.py`: Clinical data conversion
- `convertor/omics.py`: Omics data conversion
- `convertor/datafile.py`: Tarball and DataFile generation
- `convertor/utils.py`: Utilities
- `convertor/validation.py`: Output validation
- `convertor/tests/`: Unit tests

## License

MIT