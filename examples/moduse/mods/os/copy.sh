#!/bin/sh

# 参数检查
if [ $# -ne 2 ]; then
    echo "Usage: $0 <source> <destination>" >&2
    exit 2
fi

SRC="$1"
DST="$2"

# 检查源是否存在
if [ ! -e "$SRC" ]; then
    echo "Error: Source '$SRC' does not exist." >&2
    exit 1
fi

# 执行复制操作（兼容带空格和特殊字符的文件名）
cp -r -- "$SRC" "$DST"

# 返回复制操作的退出状态码
exit $?
