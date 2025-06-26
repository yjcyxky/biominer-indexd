import pytest
import tempfile
import json
from pathlib import Path
from unittest.mock import patch

@pytest.fixture
def temp_dir():
    """提供临时目录的 fixture"""
    with tempfile.TemporaryDirectory() as tmpdir:
        yield Path(tmpdir)

@pytest.fixture
def sample_cbioportal_study(temp_dir):
    """提供示例 cBioPortal 数据集的 fixture"""
    study_dir = temp_dir / 'test_study'
    study_dir.mkdir()
    
    # meta_study.txt
    meta_content = """cancer_study_identifier: test_study
name: Test Cancer Study
description: A test cancer study
type_of_cancer: BRCA
citation: Test citation
pmid: 12345
groups: PUBLIC
"""
    (study_dir / 'meta_study.txt').write_text(meta_content)
    
    # data_clinical_patient.txt
    clinical_content = """#Patient ID	Sex	Age
#Patient ID	Sex	Age
#STRING	STRING	NUMBER
#1	1	1
P001	Male	45
P002	Female	52
"""
    (study_dir / 'data_clinical_patient.txt').write_text(clinical_content)
    
    return study_dir

@pytest.fixture
def sample_omics_files(temp_dir):
    """提供示例 omics 文件的 fixture"""
    # data_mutations.txt
    mutations_content = """Hugo_Symbol	Entrez_Gene_Id	Tumor_Sample_Barcode
TP53	7157	P001
EGFR	1956	P002
"""
    (temp_dir / 'data_mutations.txt').write_text(mutations_content)
    
    # meta_mutations.txt
    mutations_meta = {
        "id_column_name": "Hugo_Symbol",
        "data_filename": "data_mutations.txt",
        "genetic_alteration_type": "MUTATION_EXTENDED"
    }
    (temp_dir / 'meta_mutations.txt').write_text(json.dumps(mutations_meta))
    
    return temp_dir

@pytest.fixture(autouse=True)
def mock_build_mappings():
    """自动 mock build_mappings 避免网络请求"""
    with patch('biominer_idxd_convertor.cbioportal2dataset.build_mappings'):
        with patch('biominer_idxd_convertor.cbioportal2dataset.code_to_disease_mapping', {}):
            with patch('biominer_idxd_convertor.cbioportal2dataset.code_to_organ_mapping', {}):
                yield

@pytest.fixture
def cli_runner():
    """提供 Click CLI 测试运行器"""
    from click.testing import CliRunner
    return CliRunner() 