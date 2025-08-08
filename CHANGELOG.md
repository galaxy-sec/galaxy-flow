# Changelog

All notable changes to the Galaxy Flow project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.10.1] - 2025-08-09

### Added
- **Galaxy Environment Initialization**: Added environment setup functionality for Galaxy platform, enabling automated configuration and initialization of Galaxy environments.
- **Network Access Control Service**: Replaced the legacy redirect service with a new network access control service, providing better security and access management.
- **Artifact Download Redirection Support**: Added redirect capabilities for artifact downloading, improving reliability and flexibility in artifact retrieval.
- **Project Root Detection**: Added GXL_PRJ_ROOT environment variable for automatic project root discovery.

### Changed
- **Dependency Updates**: Upgraded `orion_variate` from v0.6.0 to v0.6.1, bringing enhanced variable handling capabilities and minor bug fixes.
- **Workflow Enhancements**: Updated CI/CD workflows with improved build and release processes.

### Security
- **Access Control Enhancement**: Implemented network access control service replacing the legacy redirect service, improving security posture and access management for external services.

### Fixed
- **Test Cases**: Resolved various test case issues and improved test reliability.

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
- Environment variable names have been standardized. Update scripts using `GXL_CMD_ARGS` to `GXL_CMD_ARG`
- Task configuration files should be updated to use new YAML format for task reports
- For using the new task system, refer to `examples/task-*.gxl` files for configuration patterns
- Redirect URL request headers have been upgraded from legacy format to new access control format
- Legacy `undo_flow` has been replaced with `undo_hold` for transaction operations
- Command execution syntax updated with new block structure pattern, requiring updates to existing `.gxl` file syntax

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
