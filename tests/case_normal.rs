extern crate galaxy_flow;

use galaxy_flow::GxLoader;
use galaxy_flow::cmd::GxlCmd;
use galaxy_flow::execution::VarSpace;
use galaxy_flow::infra::once_init_log;
use galaxy_flow::types::AnyResult;
use log::info;
use orion_error::TestAssert;

#[tokio::test]
async fn gxl_normal_test() -> AnyResult<()> {
    once_init_log();
    let vars = VarSpace::sys_init().assert();
    let loader = GxLoader::new();

    let spc = loader
        .parse_file("./tests/material/case_normal.gxl", false, &vars)
        .await?
        .assemble()
        .assert();
    info!("------------------");
    spc.exec(
        GxlCmd::default()
            .with_env("dev".into())
            .with_flows("api,start".into()),
        VarSpace::default(),
        None,
    )
    .await?;
    Ok(())
}
