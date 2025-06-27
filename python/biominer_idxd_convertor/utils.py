import re
import numpy as np
import pandas as pd

def normalize_column_name(col: str, lower: bool = True) -> str:
    """
    Normalize column name: replace non-alphanumeric underscores with underscores, add prefix if first character is not a letter.
    """
    col = re.sub(r"\W", "_", col.strip())
    if not re.match(r"^[A-Za-z_]", col):
        col = "_" + col
    if lower:
        return col.lower()
    return col

def title_case(s: str) -> str:
    """
    Convert a string to title case.
    """
    return s.title()

def replace_missing_values(series, missing_values=None):
    """
    Replace pseudo-missing values with np.nan.
    """
    if missing_values is None:
        missing_values = {
            "NA",
            "N/A",
            "",
            " ",
            "null",
            "NULL",
            "[Not Available]",
            "Na",
            "[Not Applicable]",
            "[Discrepancy]",
        }
    return series.replace(list(missing_values), np.nan)

def deduplicate_column_names(df: pd.DataFrame) -> pd.DataFrame:
    """
    Deduplicate column names.
    """
    return df.loc[:, ~df.columns.duplicated()]
