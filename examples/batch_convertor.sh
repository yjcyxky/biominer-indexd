#!/bin/bash

data_dir=$1

if [ -z "$data_dir" ]; then
    echo "Usage: $0 <data_dir>"
    exit 1
fi

if [ ! -d "$data_dir" ]; then
    echo "Data directory $data_dir does not exist"
    exit 1
fi

for dataset in $data_dir/*; do
    if [ -d "$dataset" ]; then
        dataset_dir=examples/datasets/$(basename $dataset)
        echo "Processing $dataset -> $dataset_dir"

        if [ -d "$dataset_dir" ]; then
            echo "Dataset directory already exists"
        else
            python examples/cbioportal2dataset.py $dataset $dataset_dir
        fi
    fi
done