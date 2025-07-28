extern crate galaxy_flow;

use galaxy_flow::execution::VarSpace;
use galaxy_flow::infra::once_init_log;
use galaxy_flow::types::AnyResult;
use galaxy_flow::GxLoader;
use orion_error::TestAssert;

#[tokio::test]
async fn prj_conf() -> AnyResult<()> {
    once_init_log();
    let mut loader = GxLoader::new();

    let vars = VarSpace::sys_init().assert();
    let spc = loader
        .parse_file("./_gal/work.gxl", false, &vars)
        .await?
        .assemble()
        .assert();
    spc.exec(
        vec!["default".into()],
        vec!["conf".into()],
        Some(false),
        false,
        VarSpace::sys_init()?,
        None,
    )
    .await?;
    Ok(())
}
