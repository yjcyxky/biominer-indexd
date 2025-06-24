#!/bin/bash

data_dir=$1
output_dir=$2

if [ -z "$data_dir" ]; then
    echo "Usage: $0 <data_dir> <output_dir>"
    exit 1
fi

if [ ! -d "$data_dir" ]; then
    echo "Data directory $data_dir does not exist"
    exit 1
fi

if [ -z "$output_dir" ]; then
    echo "Usage: $0 <data_dir> <output_dir>"
    exit 1
fi

if [ ! -d "$output_dir" ]; then
    echo "Output directory $output_dir does not exist"
    exit 1
fi

mkdir -p $output_dir


for dataset in $data_dir/*; do
    if [ -d "$dataset" ]; then
        dataset_dir=$output_dir/$(basename $dataset)
        echo "Processing $dataset -> $dataset_dir"

        python examples/cbioportal2dataset.py $dataset $dataset_dir --version v0.0.1
    fi
done