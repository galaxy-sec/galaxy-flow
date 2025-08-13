mod thread_integration_test;

extern crate galaxy_flow;

use galaxy_flow::execution::VarSpace;
use galaxy_flow::infra::once_init_log;
use galaxy_flow::types::AnyResult;
use galaxy_flow::GxLoader;
use orion_error::TestAssert;

#[tokio::test]
async fn conf_base_test() -> AnyResult<()> {
    once_init_log();
    let loader = GxLoader::new();
    let vars = VarSpace::sys_init().assert();

    let spc = loader
        .parse_file("./tests/material/ability.gxl", false, &vars)
        .await?
        .assemble()
        .assert();
    spc.exec(
        vec!["default".into()],
        vec!["test".into()],
        Some(false),
        false,
        VarSpace::default(),
        None,
    )
    .await?;
    Ok(())
}

#[tokio::test]
async fn conf_web_test() {
    once_init_log();
    let vars = VarSpace::sys_init().assert();
    let loader = GxLoader::new();

    let spc = loader
        .parse_file("./tests/material/run_web.gxl", false, &vars)
        .await
        .unwrap()
        .assemble()
        .assert();
    spc.exec(
        vec!["dev".into()],
        vec!["api".into(), "api2".into()],
        Some(false),
        false,
        VarSpace::default(),
        None,
    )
    .await
    .unwrap();
}
