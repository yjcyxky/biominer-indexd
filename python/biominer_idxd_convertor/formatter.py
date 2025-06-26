import pandas as pd


def format_cna(df: pd.DataFrame):
    print("\n\nHandling CNA")
    print("Before: ", df.head())

    # 1. 重命名和去掉不需要的列
    if "Hugo_Symbol" in df.columns:
        df = df.rename(columns={"Hugo_Symbol": "gene_symbol"})
    if "Entrez_Gene_Id" in df.columns:
        df = df.drop(columns=["Entrez_Gene_Id"])
    
    # gene_symbol must be unique, let's deduplicate it
    print("✅ Keeping only unique gene_symbol")
    df = df.drop_duplicates(subset=["gene_symbol"])
    
    # Remove all rows where gene_symbol is NaN
    print("✅ Removing rows where gene_symbol is NaN")
    df = df.dropna(subset=["gene_symbol"])

    # 2. 设置 gene_symbol 为索引并转置，使行为样本，列为基因
    df = df.set_index("gene_symbol").T

    # 3. 重置索引，行为 sample_id，列为基因
    df.index.name = "sample_id"
    df = df.reset_index()

    print("After: ", df.head())
    return df


def format_mutation(df: pd.DataFrame):
    return df


def format_sv(df: pd.DataFrame):
    return df


def format_mrna_seq(df: pd.DataFrame):
    print("\n\nHandling mRNA Seq")
    print("Before: ", df.head())

    # 1. 重命名和去掉不需要的列
    if "Hugo_Symbol" in df.columns:
        df = df.rename(columns={"Hugo_Symbol": "gene_symbol"})
    if "Entrez_Gene_Id" in df.columns:
        df = df.drop(columns=["Entrez_Gene_Id"])

    # gene_symbol must be unique, let's deduplicate it
    print("✅ Keeping only unique gene_symbol")
    df = df.drop_duplicates(subset=["gene_symbol"])
    
    # Remove all rows where gene_symbol is NaN
    print("✅ Removing rows where gene_symbol is NaN")
    df = df.dropna(subset=["gene_symbol"])

    # 2. 设置 gene_symbol 为索引并转置，使行为样本，列为基因
    df = df.set_index("gene_symbol").T

    # 3. 重置索引，行为 sample_id，列为基因
    df.index.name = "sample_id"
    df = df.reset_index()

    print("After: ", df.head())
    return df


def format_methylation(df: pd.DataFrame):
    print("\n\nHandling Methylation")
    print("Before: ", df.head())

    # 1. 重命名和去掉不需要的列
    if "Hugo_Symbol" in df.columns:
        df = df.rename(columns={"Hugo_Symbol": "gene_symbol"})
    if "Entrez_Gene_Id" in df.columns:
        df = df.drop(columns=["Entrez_Gene_Id"])

    # gene_symbol must be unique, let's deduplicate it
    df = df.drop_duplicates(subset=["gene_symbol"])

    # 2. 设置 gene_symbol 为索引并转置，使行为样本，列为基因
    df = df.set_index("gene_symbol").T

    # 3. 重置索引，行为 sample_id，列为基因
    df.index.name = "sample_id"
    df = df.reset_index()

    print("After: ", df.head())
    return df
