use std::process::Command;

use tempfile::tempdir;

fn gx_bin() -> &'static str {
    env!("CARGO_BIN_EXE_gx")
}

fn write_minimal_project(dir: &std::path::Path) {
    let gal_dir = dir.join("_gal");
    std::fs::create_dir_all(&gal_dir).expect("_gal dir should be created");
    let menu_conf = r#"
mod env : base {
  env e1 {
    X="1" ;
  }
}
mod main : base {
  flow f1 { }
  flow f2 { }
}
mod base {}
"#;
    std::fs::write(gal_dir.join("work.gxl"), menu_conf).expect("work.gxl should be written");
    std::fs::write(gal_dir.join("adm.gxl"), menu_conf).expect("adm.gxl should be written");
}

#[test]
fn doc_markdown_outputs_pure_markdown() {
    let output = Command::new(gx_bin())
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["doc", "--markdown", "gx.cmd"])
        .output()
        .expect("gx doc should run");

    assert!(output.status.success(), "gx doc should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with("# gx.cmd"),
        "markdown output should start with markdown title, got: {stdout}"
    );
    assert!(
        !stdout.contains("topic:"),
        "markdown output should not contain topic metadata, got: {stdout}"
    );
    assert!(
        !stdout.contains("source:"),
        "markdown output should not contain source metadata, got: {stdout}"
    );
}

#[test]
fn adm_returns_non_zero_when_all_flows_fail() {
    let output = Command::new(gx_bin())
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["adm", "__codex_missing_flow__"])
        .output()
        .expect("gx adm should run");

    assert!(
        !output.status.success(),
        "gx adm should return non-zero when all flows fail"
    );
}

#[test]
fn mod_update_loads_user_conf() {
    let temp_home = tempdir().expect("temp home should be created");
    let galaxy_dir = temp_home.path().join(".galaxy");
    std::fs::create_dir_all(&galaxy_dir).expect(".galaxy dir should be created");
    std::fs::write(galaxy_dir.join("conf.toml"), "not = [valid")
        .expect("invalid conf should be written");

    let output = Command::new(gx_bin())
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("HOME", temp_home.path())
        .args(["mod", "update", "-d", "2"])
        .output()
        .expect("gx mod update should run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Failed to parse config file"),
        "gx mod update should load ~/.galaxy/conf.toml, stderr: {stderr}"
    );
}

#[test]
fn mod_update_reports_progress_to_stderr_only() {
    let temp_workdir = tempdir().expect("temp workdir should be created");
    write_minimal_project(temp_workdir.path());

    let output = Command::new(gx_bin())
        .current_dir(temp_workdir.path())
        .args(["mod", "update"])
        .output()
        .expect("gx mod update should run");

    assert!(output.status.success(), "gx mod update should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.trim().is_empty(),
        "gx mod update should keep stdout empty, got: {stdout}"
    );
    assert!(
        stderr.contains("no git extern modules to update in ./_gal/work.gxl"),
        "gx mod update should report work config summary to stderr, got: {stderr}"
    );
    assert!(
        stderr.contains("no git extern modules to update in ./_gal/adm.gxl"),
        "gx mod update should report adm config summary to stderr, got: {stderr}"
    );
    assert!(
        stderr.contains("project modules updated: no git extern modules found"),
        "gx mod update should report completion to stderr, got: {stderr}"
    );
}

#[test]
fn quiet_run_keeps_results_but_not_executor_explanations() {
    let output = Command::new(gx_bin())
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["run", "--quiet", "conf"])
        .output()
        .expect("gx run --quiet should run");

    assert!(output.status.success(), "gx run --quiet should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("execute flow:"),
        "quiet run should not leak executor explanations to stdout, got: {stdout}"
    );
}

#[test]
fn init_project_routes_human_messages_to_stderr() {
    let temp_home = tempdir().expect("temp home should be created");
    let temp_workdir = tempdir().expect("temp workdir should be created");

    let output = Command::new(gx_bin())
        .current_dir(temp_workdir.path())
        .env("HOME", temp_home.path())
        .args(["init", "project"])
        .output()
        .expect("gx init project should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim().is_empty(),
        "gx init project should not emit human messages to stdout, got: {stdout}"
    );
}

#[test]
fn run_without_flow_shows_menu_on_stderr_and_returns_zero() {
    let temp_workdir = tempdir().expect("temp workdir should be created");
    write_minimal_project(temp_workdir.path());

    let output = Command::new(gx_bin())
        .current_dir(temp_workdir.path())
        .args(["run"])
        .output()
        .expect("gx run should run");

    assert!(
        output.status.success(),
        "gx run without flow should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.trim().is_empty(),
        "gx run without flow should not emit menu to stdout, got: {stdout}"
    );
    assert!(
        stderr.contains("prj work menu"),
        "gx run without flow should print menu to stderr, got: {stderr}"
    );
}

#[test]
fn quiet_run_without_flow_suppresses_menu_and_still_succeeds() {
    let temp_workdir = tempdir().expect("temp workdir should be created");
    write_minimal_project(temp_workdir.path());

    let output = Command::new(gx_bin())
        .current_dir(temp_workdir.path())
        .args(["run", "--quiet"])
        .output()
        .expect("gx run --quiet should run");

    assert!(
        output.status.success(),
        "gx run --quiet without flow should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.trim().is_empty(),
        "gx run --quiet without flow should keep stdout empty, got: {stdout}"
    );
    assert!(
        stderr.trim().is_empty(),
        "gx run --quiet without flow should suppress human menu output, got: {stderr}"
    );
}

#[test]
fn adm_without_flow_shows_menu_on_stderr_and_returns_zero() {
    let temp_workdir = tempdir().expect("temp workdir should be created");
    write_minimal_project(temp_workdir.path());

    let output = Command::new(gx_bin())
        .current_dir(temp_workdir.path())
        .args(["adm"])
        .output()
        .expect("gx adm should run");

    assert!(
        output.status.success(),
        "gx adm without flow should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.trim().is_empty(),
        "gx adm without flow should not emit menu to stdout, got: {stdout}"
    );
    assert!(
        stderr.contains("prj work menu"),
        "gx adm without flow should print menu to stderr, got: {stderr}"
    );
}
