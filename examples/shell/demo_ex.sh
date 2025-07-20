
#!/bin/bash

# 定义数组
DEV_LANG=$1
echo DEV_LANG:$DEV_LANG
echo "OK-$DEV_LANG" > ${SYS_OUT:-/tmp/output.txt}