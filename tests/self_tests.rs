extern crate galaxy_flow;

use galaxy_flow::components::gxl_spc::GxlSpace;
use galaxy_flow::expect::ShellOption;
use galaxy_flow::infra::once_init_log;
use galaxy_flow::types::AnyResult;
use galaxy_flow::GxLoader;
use orion_error::TestAssert;

#[tokio::test]
async fn prj_conf() -> AnyResult<()> {
    once_init_log();
    let mut loader = GxLoader::new();
    let expect = ShellOption {
        outer_print: false,
        ..Default::default()
    };
    let spc = GxlSpace::try_from(loader.parse_file("./_gal/work.gxl", false, expect)?).assert();
    spc.exec(vec!["default".into()], vec!["conf".into()], false)
        .await?;
    Ok(())
}
