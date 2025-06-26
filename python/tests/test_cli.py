import pytest
from click.testing import CliRunner
from biominer_idxd_convertor.cli import cli
from unittest.mock import patch, MagicMock
from pathlib import Path

def test_cli_help():
    runner = CliRunner()
    result = runner.invoke(cli, ['--help'])
    assert result.exit_code == 0
    assert 'biominer-idxd command line tool' in result.output

def test_convert_help():
    runner = CliRunner()
    result = runner.invoke(cli, ['convert', '--help'])
    assert result.exit_code == 0
    assert 'Convert cBioPortal dataset to standard format' in result.output

def test_bconvert_help():
    runner = CliRunner()
    result = runner.invoke(cli, ['bconvert', '--help'])
    assert result.exit_code == 0
    assert 'Backup bulk conversion command' in result.output

@patch('biominer_idxd_convertor.cli.build_mappings')
@patch('biominer_idxd_convertor.cli.convert_cbioportal_study')
@patch('biominer_idxd_convertor.cli.Path')
@patch('biominer_idxd_convertor.omics.convert_all_omics')
def test_convert_command(mock_omics, mock_path, mock_convert, mock_build):
    runner = CliRunner()
    mock_convert.return_value = Path('/tmp/output')
    mock_path.return_value = Path('/tmp/output')
    
    with runner.isolated_filesystem():
        # 创建测试目录
        import os
        os.makedirs('test_study')
        os.makedirs('test_output')
        
        result = runner.invoke(cli, ['convert', 'test_study', 'test_output', '--organization', 'TestOrg'])
        
        assert result.exit_code == 0
        mock_build.assert_called_once()
        mock_convert.assert_called_once_with('test_study', 'test_output', 'TestOrg', 'v0.0.1')
        mock_omics.assert_called_once()

@patch('biominer_idxd_convertor.cli.convert')
def test_bconvert_command(mock_convert):
    runner = CliRunner()
    
    with runner.isolated_filesystem():
        import os
        os.makedirs('test_study/sub_study_1')
        os.makedirs('test_study/sub_study_2')
        os.makedirs('test_output')
        
        result = runner.invoke(cli, ['bconvert', 'test_study', 'test_output'])
        
        assert result.exit_code == 0
        assert mock_convert.call_count == 2

def test_convert_missing_directory():
    runner = CliRunner()
    result = runner.invoke(cli, ['convert', 'nonexistent_dir', 'output'])
    assert result.exit_code != 0 