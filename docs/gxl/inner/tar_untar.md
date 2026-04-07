# gx.tar / gx.untar

## gx.tar

```gxl
gx.tar(src: "./src", file: "./dist/src.tar.gz");
```

## gx.untar

```gxl
gx.untar(file: "./dist/src.tar.gz", dst: "./dist/unpack");
```

说明：
- `gx.untar` 解压前会处理目标路径（存在时清理）。
