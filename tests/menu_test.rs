extern crate galaxy_flow;
use galaxy_flow::components::gxl_spc::GxlSpace;
use galaxy_flow::expect::ShellOption;
use galaxy_flow::infra::once_init_log;
use galaxy_flow::menu::*;
use galaxy_flow::traits::ExecLoadTrait;
use galaxy_flow::GxLoader;
use orion_error::TestAssert;

#[test]
fn menu_normal() {
    once_init_log();
    let mut loader = GxLoader::new();
    let expect = ShellOption {
        outer_print: false,
        ..Default::default()
    };
    let spc = GxlSpace::try_from(
        loader
            .parse_file("./tests/material/menu.gxl", false, expect)
            .unwrap(),
    )
    .assert();
    let menu = spc.menu().unwrap();
    let mut expect = GxMenu::default();
    expect.envs.push(MenuItem::new(format!("g_e1"), None, None));
    expect.envs.push(MenuItem::new(format!("b_e1"), None, None));
    expect.envs.push(MenuItem::new(format!("e1"), None, None));
    expect
        .flows
        .push(MenuItem::new(format!("g_f1"), None, None));
    expect
        .flows
        .push(MenuItem::new(format!("g_f2"), None, None));
    expect
        .flows
        .push(MenuItem::new(format!("b_f1"), None, None));
    expect
        .flows
        .push(MenuItem::new(format!("b_f2"), None, None));
    expect.flows.push(MenuItem::new(format!("f1"), None, None));
    expect.flows.push(MenuItem::new(format!("f2"), None, None));
    assert_eq!(menu, expect);
}

#[test]
fn menu_simple() {
    once_init_log();
    let mut loader = GxLoader::new();
    let expect = ShellOption {
        outer_print: false,
        ..Default::default()
    };
    let spc = GxlSpace::try_from(
        loader
            .parse_file("./tests/material/simple_menu.gxl", false, expect)
            .unwrap(),
    )
    .assert();
    let menu = spc.menu().unwrap();
    let mut expect = GxMenu::default();
    expect.envs.push(MenuItem::new(format!("e1"), None, None));
    expect.flows.push(MenuItem::new(format!("f1"), None, None));
    expect.flows.push(MenuItem::new(format!("f2"), None, None));
    assert_eq!(menu, expect);
}
