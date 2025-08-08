# Changelog

All notable changes to the Galaxy Flow project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.10.0-alpha.1] - 2024-08-07
### ✨ 新增功能

#### 1. 核心功能增强
- **RedirectService 集成**：重定向服务系统

#### 2. 环境初始化增强 (Galaxy::env_init)
- 新增 `~/.galaxy/redirect.yml` 自动生成功能

#### 3. 命令行增强
- `gprj init env` 支持完整的RedirectService环境配置
- 模板系统优化，支持分支和标签筛选

## 📞 支持
如有问题，请运行：
```bash
gm init env --debug 3
```
然后到 [Issues](https://github.com/galaxy-sec/galaxy-flow/issues) 反馈
