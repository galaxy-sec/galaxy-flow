extern crate galaxy_flow;

use galaxy_flow::components::gxl_spc::GxlSpace;
use galaxy_flow::expect::ShellOption;
use galaxy_flow::infra::once_init_log;
use galaxy_flow::types::AnyResult;
use galaxy_flow::{err::*, GxLoader};
use orion_error::TestAssert;

#[tokio::test]
async fn conf_simple_test() -> AnyResult<()> {
    once_init_log();
    let mut loader = GxLoader::new();
    let opt = ShellOption {
        outer_print: false,
        ..Default::default()
    };
    let spc =
        GxlSpace::try_from(loader.parse_file("./tests/material/case_simple.gxl", false, opt)?)
            .assert();
    spc.exec(vec!["dev".into()], vec!["api".into()], false)
        .await?;
    Ok(())
}

#[ignore]
#[tokio::test]
async fn conf_cond_test() -> RunResult<()> {
    once_init_log();
    let mut loader = GxLoader::new();
    let opt = ShellOption {
        outer_print: false,
        ..Default::default()
    };
    let spc =
        GxlSpace::try_from(loader.parse_file("./tests/material/case_cond.gxl", false, opt)?)
            .assert();
    spc.exec(
        vec!["dev".into()],
        vec!["api".into(), "start".into()],
        false,
    )
    .await?;
    Ok(())
}
