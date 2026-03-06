# gx.download / gx.upload

## gx.download

```gxl
gx.download(
  url: "https://example.com/a.txt",
  local_file: "./temp/a.txt",
  username: "user",
  password: "pass"
);
```

## gx.upload

```gxl
gx.upload(
  url: "https://example.com/upload",
  local_file: "./temp/a.txt",
  method: "put",
  username: "user",
  password: "pass"
);
```

说明：
- `local_file` 的父目录必须已存在。
- `gx.download` 若 `local_file` 是目录，会按 URL 文件名落盘。
