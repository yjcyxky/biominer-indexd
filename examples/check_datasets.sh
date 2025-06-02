#!/bin/bash

json_file="datasets/index.json"

# 提取所有key并去重
keys=$(jq -r '.[].key' "$json_file")

for key in $keys; do
  if [ -d "datasets/$key" ]; then
    echo "✅ 目录存在: $key"
  else
    echo "❌ 目录不存在: datasets/$key"
  fi
done