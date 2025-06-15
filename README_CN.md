<h2 align="center">BioMiner Indexd</h2>
<p align="center">BioMiner Indexd 是一个基于哈希的数据索引、追踪与发现服务，提供全局唯一标识符。<br/>类似于 [Indexd](https://github.com/uc-cdis/indexd)，但功能更加强大。</p>

<p align="center">
<img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/yjcyxky/biominer-indexd/release.yaml?label=Build Status">
<img src="https://img.shields.io/github/license/yjcyxky/biominer-indexd.svg?label=License" alt="License"> 
<a href="https://github.com/yjcyxky/biominer-indexd/releases"><img alt="Latest Release" src="https://img.shields.io/github/release/yjcyxky/biominer-indexd.svg?label=Latest%20Release"/></a>
</p>

<p align="center">注意：目前尚未准备好用于生产环境。</p>

## 功能特性
- [x] 管理与检索文件：通过 UUID（例如：biominer.fudan-pgx/b14563ac-dbc1-49e8-b484-3dad89de1a54）索引每个文件，并记录所有存储库位置、文件名、MD5值、DOI号、存储库链接、版本号、文件大小等信息

- [x] 追踪文件位置：提供注册和追踪文件位置的机制，支持同一文件在多个存储库（OSS、S3、GSA、NODE、SRA、ENA等）中的发布

- [x] 管理多版本文件：为不同版本的文件提供基础 UUID 索引（即获取基础 UUID 后，可以查询系统中该文件的所有历史版本），适用于不同版本的流程分析生成多个版本的 Level2/3 文件

- [ ] 追踪文件状态：文件是否已索引、是否已删除、是否已更新、是否可下载等状态

- [ ] 批量获取下载链接：通过 UUID/MD5 查询指定文件，获取指定存储库的下载链接。建议与 [biopoem](https://github.com/yjcyxky/biopoem) 配合使用

- [ ] 更多功能...

## 快速开始

BioMiner Indexd 支持两种数据库模式：
1. 远程 PostgreSQL 模式（推荐用于生产环境）
2. 本地 PostgreSQL 模式（推荐用于开发、测试和小数据量的生产环境）

### 远程 PostgreSQL 模式

- 获取 BioMiner Indexd（[下载最新版本](https://github.com/yjcyxky/biominer-indexd/releases)）
- 安装 PostgreSQL（推荐版本：10.x）
- 设置环境变量

  ```bash
  export DATABASE_URL=postgres:://user:password@remote-host:5432/biominer_indexd
  # 注意：BIOMIER_REGISTRY_ID 只能设置一次。如需更改，需要重建数据库。
  export BIOMIER_REGISTRY_ID=fudan-pgx
  ```

### 本地 PostgreSQL 模式

- 获取 BioMiner Indexd（[下载最新版本](https://github.com/yjcyxky/biominer-indexd/releases)）
- 设置环境变量

  ```bash
  # 使用本地 PostgreSQL 数据库，数据将存储在本地 PostgreSQL 实例中
  # 注意：BIOMIER_REGISTRY_ID 只能设置一次。如需更改，需要重建数据库。
  export BIOMIER_REGISTRY_ID=fudan-pgx
  ```

### 启动服务

  ```bash
  $ biominer-indexd --help
    Biominer Indexd 0.1.0
    Jingcheng Yang <yjcyxky@163.com>
    组学数据文件的索引引擎

    使用方法：
        biominer-indexd [选项] [参数]

    选项：
        -D, --debug      激活调试模式
        -h, --help       显示帮助信息
        -V, --version    显示版本信息

    参数：
        -d, --database-url <database-url>    数据库 URL，例如 postgres:://user:pass@host:port/dbname。也可以通过环境变量 DATABASE_URL 设置
        -l, --local-postgres                  激活本地 postgres 模式
        -H, --host <host>                     127.0.0.1 或 0.0.0.0 [默认值: 127.0.0.1]  [可选值: 127.0.0.1, 0.0.0.0]
        -p, --port <port>                     端口号 [默认值: 3000]
        -c, --config <config>                 存储库配置文件路径 [默认值: /etc/indexd.json]
  ```

## 开发者指南

1. 安装开发依赖

  ```bash
  # Ubuntu
  sudo apt-get install postgresql-client

  # MacOS
  brew install postgresql
  ```

2. 安装 sqlx-cli

  ```bash
  cargo install sqlx-cli
  ```

3. 测试

  ```bash
  # 使用 PostgreSQL 进行测试
  # 这将使用 docker 构建测试数据库并运行 `cargo test`
  make test

  # 生成测试覆盖率报告
  cargo install cargo-tarpaulin
  cargo tarpaulin --all-features --workspace --out Html
  ```

4. 构建和运行

  ```bash
  # 远程 PostgreSQL 模式
  export DATABASE_URL=postgres://user:password@remote-host:5432/biominer_indexd
  cargo run -- --help

  # 本地 PostgreSQL 模式
  cargo run -- --local-postgres --help
  ```

## 构建说明

1. 构建前端

  ```bash
  # 所有前端文件将输出到 assets 目录
  cd studio && yarn build:embed && cd ..
  ```

2. 构建 Indexd
   
  ```bash
  # MacOSX 版本
  cargo build --release

  # Linux 版本
  cargo build --release --target=x86_64-unknown-linux-musl
  ```

3. [可选] 用于 BioMiner 服务
   
   ```bash
   cp target/x86_64-unknown-linux-musl/release/biominer-indexd ../biominer/docker/packages/
   ```

## 贡献指南
即将推出...

## 许可证
Copyright © 2022 Jingcheng Yang

基于 GNU Affero 通用公共许可证 v3.0 条款发布。 