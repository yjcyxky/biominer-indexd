import pytest
import pandas as pd
import numpy as np
from biominer_idxd_convertor.utils import normalize_column_name, replace_missing_values

def test_normalize_column_name():
    assert normalize_column_name('A B') == 'a_b'
    assert normalize_column_name('1abc') == '_1abc'
    assert normalize_column_name('Gene-Name') == 'gene_name'
    assert normalize_column_name('X@Y') == 'x_y'
    assert normalize_column_name('Z') == 'z'

def test_replace_missing_values():
    s = pd.Series(['A', 'NA', '', 'B', 'null', 'C', '[Not Available]'])
    s2 = replace_missing_values(s)
    assert s2.isna().sum() == 4
    assert s2.dropna().tolist() == ['A', 'B', 'C'] 