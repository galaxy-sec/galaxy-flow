# gx.artifact（当前未作为内置 block 能力接入）

当前 `BlockAction` 与 `stc_blk` 未直接识别 `gx.artifact`。

如果需要 artifact 流程，请通过外部模块/活动调用方式实现，例如：

```gxl
os.artifact(file: "./target/a", dst: "./artifacts");
```

建议将该能力放在 `_gal/mods` 中以模块调用方式维护。
