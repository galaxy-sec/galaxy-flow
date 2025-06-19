extern crate galaxy_flow;

use galaxy_flow::err::RunResult;
use galaxy_flow::expect::ShellOption;
use galaxy_flow::infra::once_init_log;
use galaxy_flow::types::AnyResult;
use galaxy_flow::util::ModRepo;
use galaxy_flow::GxLoader;
use galaxy_flow::{components::gxl_spc::GxlSpace, execution::VarSpace};
use orion_error::{ErrorConv, TestAssert};
use std::fs::remove_dir_all;

#[tokio::test]
async fn conf_base_test() -> AnyResult<()> {
    once_init_log();
    let mut loader = GxLoader::new();
    let vars = VarSpace::sys_init().assert();
    let expect = ShellOption {
        outer_print: false,
        ..Default::default()
    };
    let spc = GxlSpace::try_from(loader.parse_file(
        "./tests/material/ability.gxl",
        false,
        expect,
        &vars,
    )?)
    .assert();
    spc.exec(
        vec!["default".into()],
        vec!["test".into()],
        false,
        false,
        VarSpace::default(),
    )
    .await?;
    Ok(())
}

#[tokio::test]
async fn conf_web_test() {
    once_init_log();
    let vars = VarSpace::sys_init().assert();
    let mut loader = GxLoader::new();
    let sh_opt = ShellOption {
        outer_print: false,
        ..Default::default()
    };
    let spc = GxlSpace::try_from(
        loader
            .parse_file("./tests/material/run_web.gxl", false, sh_opt, &vars)
            .unwrap(),
    )
    .assert();
    spc.exec(
        vec!["dev".into()],
        vec!["api".into(), "api2".into()],
        false,
        false,
        VarSpace::default(),
    )
    .await
    .unwrap();
}

#[ignore]
#[test]
fn prj_init_test() -> RunResult<()> {
    once_init_log();
    let vars = VarSpace::sys_init().assert();
    let sh_opt = ShellOption {
        outer_print: true,
        ..Default::default()
    };
    let mut gx = GxLoader::new();
    let repo =
        ModRepo::new("https://galaxy-sec.org/free/loader/rg-tpl.git", "stable").err_conv()?;
    let rg_root = "./tmp/test";
    if std::path::Path::new(rg_root).exists() {
        remove_dir_all(rg_root).expect(rg_root);
    }
    gx.init(repo, rg_root, true, "open_pages", sh_opt.clone())?;
    let rg_conf = format!("{}/_rg/work.gxl", rg_root);
    gx.parse_file(rg_conf.as_str(), false, sh_opt.clone(), &vars)?;

    Ok(())
}
