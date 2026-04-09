# Repository Guidelines

## Project Structure & Module Organization
Galaxy Flow is a Rust workspace with two binaries in `app/` (`gflow`, `gprj`) and shared logic in `src/`. Core modules mirror the workflow runtime: `src/ability` drives execution, `src/parser`/`src/model` own the DSL, and `src/util` centralizes helpers. Workspace crates (`crates/orion_parse`, `crates/orion_cond`) extend parsing and condition evaluation. Reference diagrams live under `docs/structure/`. Integration tests sit in `tests/`, and runnable samples in `examples/`.

## Build, Test, and Development Commands
- `cargo check` — fast validation for compiler regressions; run before committing.
- `cargo build --workspace` — builds both binaries for local smoke testing.
- `cargo run --bin gflow -- --help` / `cargo run --bin gprj -- --help` — sanity-check CLI wiring after argument changes.
- `cargo fmt --all` and `cargo clippy --all-targets --all-features` — apply formatting and linting across the workspace.
- `cargo test --workspace` — runs unit and integration suites; add `--features res_depend_test` for resource-aware paths.

## Coding Style & Naming Conventions
Adopt `rustfmt` defaults (4-space indent, trailing commas in multiline literals). Use snake_case for modules, files, and functions; CamelCase for types and traits. Keep binaries thin by delegating orchestration to the library crate. Document any new public API surface in `docs/` or `AI_*` guides so flow authors can discover it.

## Testing Guidelines
Colocate unit tests inside `#[cfg(test)]` modules and name integration files `tests/<feature>_test.rs`. Leverage `rstest` for parameterized coverage and `tempfile` when touching the filesystem. New abilities need positive and failure assertions plus a CLI smoke run (`cargo run --bin gflow`). Keep AI-facing tests deterministic by mocking through `mockall` or feature guards instead of external calls.

## Commit & Pull Request Guidelines
Commit subjects stay short and imperative (e.g., `Add role parameter to ai_chat function`) and cover one logical change. Reference issues with `Refs #123` in the body when applicable. Pull requests should summarise workflow impact, list manual test commands, and attach screenshots for CLI UX updates. Request review from the owners of affected modules and wait for CI green before merge.

## Configuration & Security Tips
AI credentials load from environment variables consumed by `orion-ai`; never hardcode secrets or vendor URLs. Validate `.gxl` workflow samples with the parser before publishing. When introducing configuration keys, update `src/conf/` defaults and note deployment expectations in `docs/` to keep environments reproducible.
