extern crate galaxy_flow;

use galaxy_flow::execution::VarSpace;
use galaxy_flow::infra::once_init_log;
use galaxy_flow::types::AnyResult;
use galaxy_flow::{err::*, GxLoader};
use orion_error::TestAssert;

#[tokio::test]
async fn conf_simple_test() -> AnyResult<()> {
    once_init_log();
    let vars = VarSpace::sys_init().assert();
    let mut loader = GxLoader::new();

    let spc = loader
        .parse_file("./tests/material/case_simple.gxl", false, &vars)
        .await?
        .assemble()
        .assert();
    spc.exec(
        vec!["dev".into()],
        vec!["api".into()],
        Some(false),
        false,
        VarSpace::default(),
    )
    .await?;
    Ok(())
}

#[ignore]
#[tokio::test]
async fn conf_cond_test() -> RunResult<()> {
    once_init_log();
    let mut loader = GxLoader::new();
    let vars = VarSpace::sys_init().assert();

    let spc = loader
        .parse_file("./tests/material/case_cond.gxl", false, &vars)
        .await?
        .assemble()
        .assert();
    spc.exec(
        vec!["dev".into()],
        vec!["api".into(), "start".into()],
        Some(false),
        false,
        VarSpace::default(),
    )
    .await?;
    Ok(())
}
