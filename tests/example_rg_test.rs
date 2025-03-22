//extern crate galaxy_flow;
//#[cfg(feature = "res_depend_test")]
mod tests {
    use galaxy_flow::{
        components::gxl_spc::GxlSpace, err::RunResult, expect::ShellOption, infra::once_init_log,
        GxLoader,
    };
    use orion_error::TestAssert;

    fn test_opt() -> ShellOption {
        ShellOption {
            outer_print: false,
            ..Default::default()
        }
    }
    #[test]
    fn test_rg_info() -> RunResult<()> {
        once_init_log();
        let mut loader = GxLoader::new();
        let spc =
            GxlSpace::try_from(loader.parse_file("./example/_gal/prj.gxl", false, test_opt())?)
                .assert();

        //spc.exec(vec!["env.ut", "env.empty"], vec!["info"]).unwrap();
        spc.exec(vec!["env.ut", "env.empty"], vec!["api"], false)?;
        Ok(())
    }

    #[test]
    fn test_rg_conf_clean() -> RunResult<()> {
        once_init_log();
        let mut loader = GxLoader::new();
        let spc =
            GxlSpace::try_from(loader.parse_file("./example/_gal/prj.gxl", false, test_opt())?)
                .assert();

        spc.exec(vec!["env.ut", "env.empty"], vec!["conf", "clean"], false)?;
        Ok(())
    }

    #[test]
    fn test_rg_flow1() -> RunResult<()> {
        once_init_log();
        let mut loader = GxLoader::new();
        let spc =
            GxlSpace::try_from(loader.parse_file("./example/_gal/prj.gxl", false, test_opt())?)
                .assert();

        spc.exec(vec!["env.ut", "env.empty"], vec!["assert_main"], false)?;
        Ok(())
    }

    #[test]
    fn test_rg_flow2() -> RunResult<()> {
        once_init_log();
        let mut loader = GxLoader::new();
        let spc =
            GxlSpace::try_from(loader.parse_file("./example/_gal/prj.gxl", false, test_opt())?)
                .assert();
        spc.exec(vec!["env.ut", "env.empty"], vec!["assert_parent"], false)?;
        Ok(())
    }
}
