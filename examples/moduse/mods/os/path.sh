#!/bin/bash
if test $# -ne 1; then
  echo "错误：请提供 1 个参数作为目标目录路径。"
  exit 2
fi

DST="$1"

if [[ -z "$DST" ]]; then
  echo "错误：目录路径不能为空。"
  exit 3
fi

mkdir -p "$DST"
exit $?
