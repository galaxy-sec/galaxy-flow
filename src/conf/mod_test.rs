#[cfg(test)]
mod tests {
    use orion_error::TestAssertWithMsg;
    use orion_syspec::{tools::make_clean_path, types::Tomlable};
    use std::path::PathBuf;

    use crate::conf::{GxlConf, ReportCenterConf};

    #[test]
    fn test_gxl_conf_serialization() {
        let conf = GxlConf::new(ReportCenterConf::new("example.com", 8080), true);
        let path = PathBuf::from("./temp/conf");
        make_clean_path(&path).assert("clean path");
        let file = path.join("conf.toml");
        conf.save_toml(&file).assert("save toml");
        let loaded = GxlConf::from_toml(&file).assert("load toml");
        assert_eq!(loaded.report_svr(), conf.report_svr());
    }
}
