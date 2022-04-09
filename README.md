<h2 align="center">BioMiner Indexd</h2>
<p align="center">BioMiner Indexd is a hash-based data indexing and tracking service providing globally unique identifiers. Similar to [Indexd](https://github.com/uc-cdis/indexd), but with a more.</p>

<p align="center">
<img alt="GitHub Workflow Status" src="https://img.shields.io/github/workflow/status/yjcyxky/biominer-indexd/test-pack-and-release?label=Build Status">
<img src="https://img.shields.io/github/license/yjcyxky/biominer-indexd.svg?label=License" alt="License"> 
<a href="https://github.com/yjcyxky/biominer-indexd/releases"><img alt="Latest Release" src="https://img.shields.io/github/release/yjcyxky/biominer-indexd.svg?label=Latest%20Release"/></a>
</p>

## Features
- [x] Manage & retrieve files: index each file by UUID (e.g. biominer.fudan-pgx/b14563ac-dbc1-49e8-b484-3dad89de1a54) and record all repository locations, file names, MD5 values, DOI numbers, repository links, version numbers, sizes, etc. of files

- [x] Track file location: for the same file released in multiple repositories (OSS, S3, GSA, NODE, SRA, ENA.), provide a mechanism to register & track file location.

- [x] Manage multi-version files: For different versions of Pipeline analysis to generate multiple versions of Level2/3 files, Biominer Indexd provide Base UUID indexing of different versions of files (i.e., get the Base UUID, you can query all the historical versions of a file in the system)

- [ ] Bulk get download links: query specified files by UUID/MD5 and get download links of specified repositories.

## Quick Start
- Get Latest BioMiner Indexd: [Download](https://github.com/yjcyxky/biominer-indexd/releases)
- Install PostgreSQL (Recommended version: 10.x)
- Set Environment Variables

      ```bash
      export DATABASE_URL=postgres:://user:password@localhost:5432/biominer_indexd
      # NOTE: BIOMIER_REGISTRY_ID only allow to set one time. If you want to change it, you need to rebuild the database.
      export BIOMIER_REGISTRY_ID=fudan-pgx
      ```

- Start BioMiner Indexd

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
        -d, --database-url <database-url>    Database url, such as postgres:://user:pass@host:port/dbname. You can also set
                                            it with env var: DATABASE_URL
        -H, --host <host>                    127.0.0.1 or 0.0.0.0 [default: 127.0.0.1]  [possible values: 127.0.0.1,
                                            0.0.0.0]
        -p, --port <port>                    Which port [default: 3000]
  ```

## For Developers
### Prerequisites

1. Install sqlx-cli

  ```bash
  cargo install sqlx-cli
  ```

2. Install docker

  ```bash
  sudo apt-get install docker.io
  ```