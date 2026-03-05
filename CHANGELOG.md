# Changelog

All notable changes to the Galaxy Flow project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.12.4] - 2026-03-05

### Added
- **Self-update command set**: Added `gprj self status/check/update/rollback/auto` for upgrade management.
- **Self-update runtime module**: Added `src/self_update/*` covering policy/state storage, manifest download, checksum verification, install/rollback, and health check.
- **Repository manifests**: Added `updates/stable/manifest.json`, `updates/alpha/manifest.json`, `updates/beta/manifest.json`, and `updates/README.md`.

### Changed
- **Release channels**: Unified channels to `stable|alpha|beta` (removed `pre` compatibility mode).
- **Default manifest source**: `manifest_base_url` now points to this repository raw path under `updates/`.
- **Version automation**: Switched project version bump flow to `gx.patch_file` marker-based patching in `_gal/adm.gxl`.

### Fixed
- **Update semantics**: `--dry-run` no longer requires `--yes`.
- **State persistence**: Expanded failure-state recording for post-manifest update stages.
- **Lock safety**: Added stale-lock recovery with PID liveness checks.
- **Rollback robustness**: Hardened backup id validation and prevented path traversal via `rollback --id`.
- **Installer safety**: Enforced unique binary detection and ignored symlink hits in package scanning.
- **Temp artifacts**: Added automatic cleanup of self-update temporary directories.

## [v0.12.0] - 2026-02-25

### Changed
- **依赖升级**: 核心依赖迁移至 crates.io 注册版本
  - `orion_conf`: `~0.1` → `0.4` (trait 重命名: `Yamlable`→`YamlIO`, `Tomlable`→`TomlIO`, `JsonAble`→`JsonIO`, `IniAble`→`IniIO`; 方法重命名: `from_yml`→`load_yaml`, `save_yml`→`save_yaml` 等)
  - `orion-sec`: `v0.2.0` (git) → `0.3` (registry)
  - `orion-infra`: `v0.3.1` (git) → `0.4` (registry)
  - `orion-variate`: `v0.9.1` (git) → `0.10` (registry)
- **模块迁移**: `orion-variate` 中的 `addr`、`types`、`update`、`archive` 模块迁移至新增依赖 `orion-accessor = 0.5.4`

### Removed
- **临时移除 orion-ai**: 因 `orion-ai` v0.2.1 依赖旧版 `orion-variate`/`orion-sec` 产生版本冲突，暂时移除 AI 相关功能，待 `orion-ai` 升级后恢复
  - 移除 `gx.ai_chat`、`gx.ai_task`、`gx.ai_regist` 解析与执行
  - `ai_diagnose` 暂时禁用
  - Galaxy 环境初始化不再生成 AI 配置文件

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
## [0.10.1] - 2025-08-09

### Added
- **Galaxy Environment Initialization**: Added environment setup functionality for Galaxy platform, enabling automated configuration and initialization of Galaxy environments.
- **Network Access Control Service**: Replaced the legacy redirect service with a new network access control service, providing better security and access management.
- **Artifact Download Redirection Support**: Added redirect capabilities for artifact downloading, improving reliability and flexibility in artifact retrieval.
- **Removed gx.artifact Ability**: Removed the legacy gx.artifact capability in favor of more flexible download alternatives (breaking change).
- **Project Root Detection**: Added GXL_PRJ_ROOT environment variable for automatic project root discovery.

### Changed
- **Dependency Updates**: Upgraded `orion_variate` from v0.6.2 (tagged release), bringing enhanced variable handling capabilities and improved stability over branch-based dependencies.
- **Workflow Enhancements**: Updated CI/CD workflows with improved build and release processes.

### Security
- **Access Control Enhancement**: Implemented network access control service replacing the legacy redirect service, improving security posture and access management for external services.

### Fixed
- **Test Cases**: Resolved various test case issues and improved test reliability.
- **Code Cleanup**: Removed deprecated artifact module and related parser code, reducing complexity and maintenance burden.

## [0.10.0] - 2025-08-07

### Added
- **Enhanced Template Engine**: Added support for template variables (gx.tpl) with subdirectory support and improved render system.
- **TOML Configuration Support**: Enhanced TOML file reading capabilities with improved parser and validation (#5).
- **Task System Integration**: Comprehensive task execution framework with:
  - **Task Scheduling and Execution**: Full task lifecycle management with state tracking
  - **Local Task Report Generation**: Automatically generate YAML reports for task execution results
  - **Task Result Persistence**: Structured YAML output with detailed execution metrics
  - **Task Status Synchronization**: Real-time sync with central task management services
  - **Dry Run Functionality**: Pre-execution validation with `@dryrun` annotations and `--dry-run` CLI flag
  - **Retry Mechanisms**: Automatic retry for configuration file loading with backoff strategies
  - **Task Annotations**: Rich metadata support (@task, @dryrun annotations)
  - **HTTP Callback Integration**: Seamless integration with task management centers
- **Output Capture System**: Standard output and error capture with redirection capabilities
- **GXL Environment Variables**: Support for GXL-specific environment variables:
  - `GXL_PRJ_ROOT` for project root detection
  - `GXL_CMD_ARG` for command argument handling
  - `GXL_CMD_ARGS` (deprecated) -> `GXL_CMD_ARG`
- **Transaction Support**: Added transaction capabilities with rollback functionality through `undo_hold` operations
- **WildMatch Pattern Support**: Added wildmatch crate for advanced glob pattern matching
- **Command Block Syntax**: Enhanced command execution with structured block syntax
- **Async Execution Engine**: Refactored core engine to use async/await for better performance

### Changed
- **Refactored Error Handling**: Improved error reporting across the system with better error messages and stack traces
- **Refactored Flow Structure**: Modernized flow execution structure with enhanced async support
- **Pipeline Syntax**: Improved flow pipe syntax supporting more complex operations
- **Configuration Loading**: Enhanced configuration file loading with retry mechanisms and graceful handling

### Deprecated
- **Legacy Redirect Service**: Marked for removal in favor of network access control service
- **Legacy Artifact Service**: The gx.artifact ability has been removed in v0.10.1

### Fixed
- **Command Environment Variables**: Fixed issues with environment variable propagation in command execution
- **Task Configuration**: Proper handling when task configuration files or URLs are missing
- **Log Redirection**: Corrected issues with standard output/err redirection during task execution
- **Data Format Handling**: Fixed datetime format issues in task reporting

### Developer Experience
- **Enhanced CLI**: Added `--dry-run` command line parameter for testing configurations
- **Better Debugging**: Added warning prompts when flows don't exist and improved error messages
- **Testing Framework**: Comprehensive test suite additions including:
  - Task system test cases
  - Configuration loading tests
  - Output capture verification
  - Command execution tests

## [0.9.2-beta.1] - 2024-09-05

Previous stable release. This changelog covers changes from 0.9.2-beta.1 to 0.10.1.

---
**Note**: Version 0.10.x represents a significant evolution from 0.9.x with major architectural improvements, especially in task management, configuration handling, and system integration.

### Migration Notes
- **Environment variable names have been standardized. Update scripts using `GXL_CMD_ARGS` to `GXL_CMD_ARG`
- **Task configuration files should be updated to use new YAML format for task reports
- **For using the new task system, refer to `examples/task-*.gxl` files for configuration patterns
- **Redirect URL request headers have been upgraded from legacy format to new access control format
- **Legacy `undo_flow` has been replaced with `undo_hold` for transaction operations
- **Command execution syntax updated with new block structure pattern, requiring updates to existing `.gxl` file syntax
- **Migrate from gx.artifact**: Replace any gx.artifact usage with gx.download for better flexibility and control

### Upgrade Checklist
- [ ] Update environment variable declarations from `GXL_CMD_ARGS` to `GXL_CMD_ARG`
- [ ] Verify all configuration files use YAML format
- [ ] Validate task report URL configurations for new format compatibility
- [ ] Test compatibility of existing `.gxl` files with new version
- [ ] Confirm transaction rollback operations use new `undo_hold` syntax

---
### Related Issue Links
- Task System Design: #5
- TOML Support: #5
- Output Capture: #31, #32
- Redirect Fixes: #35
- CLI Error Handling: #46
- Download Improvements: #71
- Refactor Redirect Service: #66

### Contributors
Special thanks to the following developers for their contributions during the 0.10.x release cycle: @wukong, @sec-wukong, @tangxy1024, @tangxiangyan, @可乐加冰
>>>>>>> release/0.10
