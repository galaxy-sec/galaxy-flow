extern crate galaxy_flow;

use galaxy_flow::components::gxl_spc::GxlSpace;
use galaxy_flow::expect::ShellOption;
use galaxy_flow::infra::once_init_log;
use galaxy_flow::types::AnyResult;
use galaxy_flow::GxLoader;
use log::info;
use orion_error::TestAssert;

#[test]
fn gxl_normal_test() -> AnyResult<()> {
    once_init_log();
    let mut loader = GxLoader::new();
    let expect = ShellOption {
        outer_print: false,
        ..Default::default()
    };
    let spc =
        GxlSpace::try_from(loader.parse_file("./tests/material/case_normal.gxl", false, expect)?)
            .assert();
    info!("------------------");
    spc.exec(vec!["dev"], vec!["api", "start"], false)?;
    Ok(())
}
