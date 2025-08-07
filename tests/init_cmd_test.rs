extern crate galaxy_flow;
extern crate shells;

use std::path::PathBuf;

use galaxy_flow::infra::once_init_log;
use galaxy_flow::util::path::WorkDir;
use galaxy_flow::GxLoader;
use orion_error::TestAssertWithMsg;
use orion_infra::path::ensure_path;
use orion_variate::addr::GitRepository;

// use shells;
#[ignore]
#[tokio::test]
async fn init_test() {
    once_init_log();
    let loader = GxLoader::new();

    let path = PathBuf::from("./tests/temp/init");
    ensure_path(&path).assert("path");
    let _work_path = WorkDir::change(&path);
    let addr =
        GitRepository::from("https://github.com/galaxy-sec/gal-init.git").with_branch("main");
    loader.init(addr, "example").await.assert("init");
}
