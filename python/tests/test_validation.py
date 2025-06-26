import tempfile
from pathlib import Path
import pandas as pd
import json
from biominer_idxd_convertor.validation import validate_output_dir

def test_validate_output_dir():
    with tempfile.TemporaryDirectory() as tmpdir:
        d = Path(tmpdir)
        (d / 'metadata_table.parquet').write_bytes(b'parquet')
        (d / 'metadata_dictionary.json').write_text(json.dumps([{'key': 'a'}]))
        (d / 'dataset.json').write_text(json.dumps({'key': 'test'}))
        (d / 'datafile.tsv').write_text('guid\tfilename\nabc\ttest.tar.gz\n')
        datafiles = d / 'datafiles'
        datafiles.mkdir()
        (datafiles / 'mut.parquet').write_bytes(b'parquet')
        assert validate_output_dir(d) is True
        # 缺少 parquet 文件
        (datafiles / 'mut.parquet').unlink()
        assert validate_output_dir(d) is False 