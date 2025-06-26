import tempfile
import os
import json
from pathlib import Path
from biominer_idxd_convertor.datafile import make_tarball, make_datafile, make_datafile_tsv

def test_make_tarball_and_datafile():
    with tempfile.TemporaryDirectory() as tmpdir:
        d = Path(tmpdir)
        (d / 'a.txt').write_text('hello')
        tarball = d / 'test.tar.gz'
        make_tarball(d, tarball)
        assert tarball.exists()
        meta = {'key': 'testset', 'uploader': 'tester'}
        datafile = make_datafile(tarball, meta)
        assert datafile.filename.endswith('.tar.gz')
        assert datafile.guid.startswith('biominer.')
        outdir = d / 'out'
        outdir.mkdir()
        tsv_path = make_datafile_tsv(datafile, outdir)
        assert tsv_path.exists()
        lines = tsv_path.read_text().splitlines()
        assert lines[0].startswith('guid')
        assert 'testset.tar.gz' in lines[1] 