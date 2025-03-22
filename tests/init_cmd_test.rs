extern crate galaxy_flow;
extern crate shells;

use galaxy_flow::expect::ShellOption;
use galaxy_flow::infra::once_init_log;
use galaxy_flow::util::ModRepo;
use galaxy_flow::GxLoader;

// use shells;
#[ignore]
#[test]
fn rg_test() {
    once_init_log();
    let loader = GxLoader::new();
    let expect = ShellOption {
        outer_print: false,
        ..Default::default()
    };
    let cmd = format!("rm -rf ./tmp/");
    let (_code, _stdout, _stderr) = shells::execute_with("bash", &cmd);
    let repo = ModRepo::new("https://galaxy-sec.org/free/galaxy/rg-tpl.git", "develop").unwrap();
    loader
        .init(repo, "./tmp/", true, "simple", expect.clone())
        .unwrap();

    let cmd = format!("rm -rf ./tmp/");
    let (_code, _stdout, _stderr) = shells::execute_with("bash", &cmd);
    let repo = ModRepo::new("https://galaxy-sec.org/free/galaxy/rg-tpl.git", "beta").unwrap();
    loader
        .init(repo, "./tmp/", true, "simple", expect.clone())
        .unwrap();
}
