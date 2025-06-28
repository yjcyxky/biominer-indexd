import tempfile
import shutil
import json
import pandas as pd
from pathlib import Path
from biominer_idxd_convertor.omics import find_omics_files, convert_omics_file

def test_find_omics_files():
    with tempfile.TemporaryDirectory() as tmpdir:
        d = Path(tmpdir)
        (d / 'data_mutations.txt').write_text('A\tB\n1\t2\n3\t4\n')
        (d / 'meta_mutations.txt').write_text('id_column_name: A\ndata_filename: data_mutations.txt\n')
        pairs = find_omics_files(d)
        assert len(pairs) == 1
        assert pairs[0][0].name == 'data_mutations.txt'
        assert pairs[0][1].name == 'meta_mutations.txt'
        assert pairs[0][2] == 'mutations'

def test_convert_omics_file():
    with tempfile.TemporaryDirectory() as tmpdir:
        d = Path(tmpdir)
        (d / "data_mrna_seq.txt").write_text("Gene\tValue\nTP53\t1.2\nEGFR\t3.4\n")
        (d / "meta_mrna_seq.txt").write_text('id_column_name: Gene\ndata_filename: data_mrna_seq.txt\n')
        outdir = d / 'out'
        outdir.mkdir()
        convert_omics_file(d / "data_mrna_seq.txt", d / "meta_mrna_seq.txt", outdir)
        assert (outdir / 'mrna_seq.parquet').exists()
        assert (outdir / 'mrna_seq_dictionary.json').exists()
        assert (outdir / 'mrna_seq_metadata.json').exists()
        df = pd.read_parquet(outdir / 'mrna_seq.parquet')
        assert 'Gene' in df.columns
        assert 'Value' in df.columns 
