<h2 align="center">BioMiner Indexd</h2>
<p align="center">BioMiner Indexd is a hash-based data indexing and tracking service providing globally unique identifiers. <br/>Similar to [Indexd](https://github.com/uc-cdis/indexd), but with a more.</p>

<p align="center">
<a href="README.md">English</a> | <a href="README_CN.md">中文</a>
</p>

<p align="center">
<img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/yjcyxky/biominer-indexd/release.yaml?label=Build Status">
<img src="https://img.shields.io/github/license/yjcyxky/biominer-indexd.svg?label=License" alt="License"> 
<a href="https://github.com/yjcyxky/biominer-indexd/releases"><img alt="Latest Release" src="https://img.shields.io/github/release/yjcyxky/biominer-indexd.svg?label=Latest%20Release"/></a>
</p>

<p align="center">NOTE: NOT READY FOR PRODUCTION YET.</p>

## Features
- [x] Manage & retrieve files: index each file by UUID (e.g. biominer.fudan-pgx/b14563ac-dbc1-49e8-b484-3dad89de1a54) and record all repository locations, file names, MD5 values, DOI numbers, repository links, version numbers, sizes, etc. of files

- [x] Track file location: provide a mechanism to register & track file location, for the same file released in multiple repositories (OSS, S3, GSA, NODE, SRA, ENA.)

- [x] Manage multi-version files: provide Base UUID indexing of different versions of files (i.e., get the Base UUID, you can query all the historical versions of a file in the system) for different versions of Pipeline analysis to generate multiple versions of Level2/3 files.

- [ ] Track file status: whether the file is in the index, or has been deleted, or has been updated, or can be downloaded.

- [ ] Bulk get download links: query specified files by UUID/MD5 and get download links of specified repositories. It is better to use with [biopoem](https://github.com/yjcyxky/biopoem).

- [ ] More features...

## Quick Start

BioMiner Indexd supports two database modes:
1. Remote PostgreSQL mode (recommended for production)
2. Local PostgreSQL mode (recommended for development, testing and production with small data size)

### Remote PostgreSQL Mode

- Get BioMiner Indexd ([Download Latest Version](https://github.com/yjcyxky/biominer-indexd/releases))
- Install PostgreSQL (Recommended version: 10.x)
- Set Environment Variables

  ```bash
  export DATABASE_URL=postgres:://user:password@remote-host:5432/biominer_indexd
  # NOTE: BIOMIER_REGISTRY_ID only allows to be set once. If you want to change it, you need to rebuild the database.
  export BIOMIER_REGISTRY_ID=fudan-pgx
  ```

### Local PostgreSQL Mode

- Get BioMiner Indexd ([Download Latest Version](https://github.com/yjcyxky/biominer-indexd/releases))
- Set Environment Variables

  ```bash
  # Using local PostgreSQL database, data will be stored in your local PostgreSQL instance
  # NOTE: BIOMIER_REGISTRY_ID only allows to be set once. If you want to change it, you need to rebuild the database.
  export BIOMIER_REGISTRY_ID=fudan-pgx
  ```

### Start Service

  ```bash
  $ biominer-indexd --help
    Biominer Indexd 0.1.0
    Jingcheng Yang <yjcyxky@163.com>
    An Index Engine for Omics Data Files

    USAGE:
        biominer-indexd [FLAGS] [OPTIONS]

    FLAGS:
        -D, --debug      Activate debug mode short and long flags (-D, --debug) will be deduced from the field's name
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
        -d, --database-url <database-url>    Database url, such as postgres:://user:pass@host:port/dbname. You can also set it with env var: DATABASE_URL
        -l, --local-postgres                  Activate local postgres mode
        -H, --host <host>                    127.0.0.1 or 0.0.0.0 [default: 127.0.0.1]  [possible values: 127.0.0.1, 0.0.0.0]
        -p, --port <port>                    Which port [default: 3000]
        -c, --config <config>                The path of the repo config file. [default: /etc/indexd.json]
  ```

## For Developers

1. Install Development Dependencies

  ```bash
  # Ubuntu
  sudo apt-get install postgresql-client

  # MacOS
  brew install postgresql
  ```

2. Install sqlx-cli

  ```bash
  cargo install sqlx-cli
  ```

1. Testing

  ```bash
  # Test with PostgreSQL
  # This will build a testing database with docker and run `cargo test`
  make test

  # Generate test coverage report
  cargo install cargo-tarpaulin
  cargo tarpaulin --all-features --workspace --out Html
  ```

5. Build & Run

  ```bash
  # Remote PostgreSQL mode
  export DATABASE_URL=postgres://user:password@remote-host:5432/biominer_indexd
  cargo run -- --help

  # Local PostgreSQL mode
  cargo run -- --local-postgres --help
  ```

## Build

1. Build Frontend

  ```bash
  # All frontend files will output to assets directory.
  cd studio && yarn build:embed && cd ..
  ```

2. Build Indexd
   
  ```bash
  # For MacOSX
  cargo build --release

  # For Linux
  cargo build --release --target=x86_64-unknown-linux-musl
  ```

3. [Optional] For BioMiner Service
   
   ```bash
   cp target/x86_64-unknown-linux-musl/release/biominer-indexd ../biominer/docker/packages/
   ```

## Contributing
Comming soon...

## License
Copyright © 2022 Jingcheng Yang

Distributed under the terms of the GNU Affero General Public License v3.0.