for d in gdc/*/; do
  dir="${d%/}"                    # 去掉结尾斜杠
  base=$(basename "$dir")        # 提取子目录名，例如 dataset1
  tar -czf "datasets/${base}.tar.gz" -C "$dir" .  # 只打包目录内容
  echo "✅ Packed $dir -> datasets/${base}.tar.gz"
done