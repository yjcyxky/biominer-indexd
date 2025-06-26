import pytest
import tempfile
import json
import pandas as pd
from pathlib import Path
from unittest.mock import patch, MagicMock
from biominer_idxd_convertor.cbioportal2dataset import (
    parse_meta_study, build_data_dictionary_from_header, 
    read_clinical_file, get_file_hash,
    generate_deterministic_guid, md5sum_to_baseid, convert_cbioportal_study
)

def test_get_file_hash():
    with tempfile.NamedTemporaryFile(mode='w', delete=False) as f:
        f.write('test content')
        f.flush()
        hash_md5 = get_file_hash(f.name, 'md5')
        hash_sha1 = get_file_hash(f.name, 'sha1')
        assert len(hash_md5) == 32
        assert len(hash_sha1) == 40
        assert hash_md5 != hash_sha1

def test_generate_deterministic_guid():
    with tempfile.NamedTemporaryFile(mode='w', delete=False) as f:
        f.write('test content')
        f.flush()
        guid = generate_deterministic_guid(f.name)
        assert guid.startswith('biominer.fudan-pgx/')
        # 相同内容应生成相同 GUID
        guid2 = generate_deterministic_guid(f.name)
        assert guid == guid2

def test_md5sum_to_baseid():
    baseid = md5sum_to_baseid('d41d8cd98f00b204e9800998ecf8427e')
    assert len(baseid) == 36  # UUID length
    assert baseid.count('-') == 4

def test_parse_meta_study():
    meta_content = """cancer_study_identifier: test_study
name: Test Study
description: A test study
type_of_cancer: BRCA
citation: Test citation
pmid: 12345
groups: PUBLIC;GDAC
"""
    with tempfile.NamedTemporaryFile(mode='w', delete=False) as f:
        f.write(meta_content)
        f.flush()
        
        with patch('biominer_idxd_convertor.cbioportal2dataset.code_to_disease_mapping', {}):
            result = parse_meta_study(f.name, 'TestOrg')
            
            assert result['key'] == 'test_study'
            assert result['name'] == 'Test Study'
            assert result['description'] == 'A test study'
            assert result['citation'] == 'Test citation'
            assert result['pmid'] == '12345'
            assert result['groups'] == ['PUBLIC', 'GDAC']
            assert 'org:TestOrg' in result['tags']

def test_read_clinical_file():
    clinical_content = """#Patient ID	Sex	Age
#Patient ID	Sex	Age
#STRING	STRING	NUMBER
#1	1	1
patient_id	sex	age
P001	Male	45
P002	Female	52
"""
    with tempfile.NamedTemporaryFile(mode='w', delete=False) as f:
        f.write(clinical_content)
        f.flush()

        df, headers = read_clinical_file(f.name)

        # 检查数据行数（应该是2行：P001和P002）
        assert len(df) == 2
        assert 'patient_id' in df.columns
        assert 'sex' in df.columns
        assert 'age' in df.columns
        assert len(headers) >= 4

def test_build_data_dictionary_from_header():
    df = pd.DataFrame({
        'patient_id': ['P001', 'P002'],
        'age': [45, 52],
        'status': ['Alive', 'Dead']
    })
    
    headers = [
        {'patient_id': 'Patient ID', 'age': 'Age', 'status': 'Status'},
        {'patient_id': 'Patient identifier', 'age': 'Age in years', 'status': 'Vital status'},
        {'patient_id': 'STRING', 'age': 'NUMBER', 'status': 'STRING'},
        {'patient_id': 1, 'age': 2, 'status': 3}
    ]
    
    dictionary = build_data_dictionary_from_header(df, headers)
    
    assert len(dictionary) == 3
    assert dictionary[0]['key'] == 'patient_id'
    assert dictionary[0]['name'] == 'Patient ID'
    assert dictionary[0]['data_type'] == 'STRING'
    assert dictionary[1]['data_type'] == 'NUMBER'

@patch('biominer_idxd_convertor.cbioportal2dataset.build_mappings')
@patch('biominer_idxd_convertor.cbioportal2dataset.make_tarball')
@patch('biominer_idxd_convertor.cbioportal2dataset.make_datafile')
@patch('biominer_idxd_convertor.cbioportal2dataset.make_datafile_tsv')
def test_convert_cbioportal_study_integration(mock_tsv, mock_datafile, mock_tarball, mock_build):
    with tempfile.TemporaryDirectory() as tmpdir:
        study_dir = Path(tmpdir) / 'test_study'
        study_dir.mkdir()
        
        # Create meta_study.txt
        meta_content = """cancer_study_identifier: test_study
name: Test Study
description: A test study
type_of_cancer: BRCA
"""
        (study_dir / 'meta_study.txt').write_text(meta_content)
        
        # Create clinical file
        clinical_content = """#Patient ID	Sex	Age
#Patient ID	Sex	Age
#STRING	STRING	NUMBER
#1	1	1
P001	Male	45
P002	Female	52
"""
        (study_dir / 'data_clinical_patient.txt').write_text(clinical_content)
        
        output_dir = Path(tmpdir) / 'output'
        
        with patch('biominer_idxd_convertor.cbioportal2dataset.code_to_disease_mapping', {}):
            with patch('biominer_idxd_convertor.cbioportal2dataset.code_to_organ_mapping', {}):
                result = convert_cbioportal_study(study_dir, output_dir, 'TestOrg', 'v0.0.1')
                
                assert result.exists()
                assert (result / 'metadata_table.parquet').exists()
                assert (result / 'metadata_dictionary.json').exists()
                assert (result / 'dataset.json').exists() 
