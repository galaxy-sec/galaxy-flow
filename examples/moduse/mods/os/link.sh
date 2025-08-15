#!/bin/sh

# 确保传入两个参数
if [ $# -ne 2 ]; then
    echo "用法: $0 <源路径> <目标路径>" >&2
    exit 1
fi

SRC="$1"
DST="$2"

# 若目标是符号链接则删除
if [ -L "$DST" ]; then
    if ! unlink "$DST" 2>/dev/null; then
        echo "错误: 无法删除已存在的符号链接 '$DST'" >&2
        exit 1
    fi
fi

# 创建符号链接并检查结果
if ln -s "$SRC" "$DST"; then
    echo "成功创建符号链接: '$DST' -> '$SRC'"
    exit 0
else
    echo "错误: 无法创建符号链接 '$DST'" >&2
    exit 1
fi
