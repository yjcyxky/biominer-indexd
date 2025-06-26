# convertor 中文说明

convertor 是一个用于将 cBioPortal 格式数据集自动转换为标准结构的 Python 包，适用于下游 AI 生信分析流程。输出包括统一格式的 `dataset.json`、`metadata_dictionary.json`、`metadata_table.parquet`、`datafile.tsv`，以及标准化的 omics Parquet/JSON 文件，支持一键打包和 DataFile 对象生成。

## 主要特性
- 临床与 omics 数据自动转换与归一化
- 自动生成标准文件结构和元数据
- 命令行工具：`biominer-idxd convert` 和 `biominer-idxd bconvert`
- 模块化、易扩展、100%测试覆盖

## 安装方法

```bash
pip install .
```

## 命令行用法

```bash
biominer-idxd convert <study_dir> <output_dir> --organization <机构名> --version <版本号>
biominer-idxd bconvert <study_dir> <output_dir> --organization <机构名> --version <版本号>
```

- `<study_dir>`：cBioPortal 格式的研究目录路径
- `<output_dir>`：标准化输出目录
- `--organization`：机构名称（默认 Unassigned）
- `--version`：输出版本号（默认 v0.0.1）

## 开发与测试

- 所有主逻辑位于 `convertor/` 目录，按模块划分
- 运行全部测试：

```bash
pytest --cov=convertor
```

- 要求 100% 测试覆盖率，见 `convertor/tests/` 示例

## 目录结构说明

- `convertor/cli.py`：命令行入口（Click）
- `convertor/cbioportal2dataset.py`：临床数据转换
- `convertor/omics.py`：omics 数据转换
- `convertor/datafile.py`：打包与 DataFile 生成
- `convertor/utils.py`：工具函数
- `convertor/validation.py`：输出结构校验
- `convertor/tests/`：单元测试

## 许可证

MIT 