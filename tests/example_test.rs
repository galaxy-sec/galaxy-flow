//extern crate galaxy_flow;
//#[cfg(feature = "res_depend_test")]
mod tests {
    use galaxy_flow::err::report_gxl_error;
    use galaxy_flow::execution::VarSpace;
    use galaxy_flow::util::path::WorkDirWithLock;
    use galaxy_flow::{err::RunResult, expect::ShellOption, infra::once_init_log, GxLoader};
    use orion_error::TestAssert;

    fn test_opt() -> ShellOption {
        ShellOption {
            quiet: false,
            ..Default::default()
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn example_read() -> RunResult<()> {
        //once_init_log();
        let _dir = WorkDirWithLock::change("./examples/read");
        let vars = VarSpace::sys_init().assert();
        let mut loader = GxLoader::new();
        let spc = loader
            .parse_file("./_gal/work.gxl", false, test_opt(), &vars)?
            .assemble()
            .assert();
        spc.exec(
            vec!["default".into()],
            vec!["conf".into()],
            Some(false),
            false,
            VarSpace::default(),
        )
        .await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn example_shell() -> RunResult<()> {
        //once_init_log();
        let _dir = WorkDirWithLock::change("./examples/shell");
        let vars = VarSpace::sys_init().assert();
        let mut loader = GxLoader::new();
        let spc = loader
            .parse_file("./_gal/work.gxl", false, test_opt(), &vars)?
            .assemble()
            .assert();
        spc.exec(
            vec!["default".into()],
            vec!["conf".into()],
            Some(false),
            false,
            VarSpace::default(),
        )
        .await?;
        Ok(())
    }
    #[tokio::test(flavor = "current_thread")]
    async fn example_assert() -> RunResult<()> {
        //once_init_log();
        let _dir = WorkDirWithLock::change("./examples/assert");
        let vars = VarSpace::sys_init().assert();
        let mut loader = GxLoader::new();
        let spc = loader
            .parse_file("./_gal/work.gxl", false, test_opt(), &vars)?
            .assemble()
            .assert();
        spc.exec(
            vec!["default".into()],
            vec!["assert_main".into()],
            Some(false),
            false,
            VarSpace::default(),
        )
        .await?;
        Ok(())
    }
    #[tokio::test(flavor = "current_thread")]
    async fn example_template() -> RunResult<()> {
        //jonce_init_log();
        let _dir = WorkDirWithLock::change("./examples/template");
        let mut loader = GxLoader::new();
        let vars = VarSpace::sys_init().assert();
        let spc = loader
            .parse_file("./_gal/work.gxl", false, test_opt(), &vars)?
            .assemble()
            .assert();
        spc.exec(
            vec!["default".into()],
            vec!["conf".into()],
            Some(false),
            false,
            VarSpace::default(),
        )
        .await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn example_translate() -> RunResult<()> {
        once_init_log();
        let _dir = WorkDirWithLock::change("./examples/transaction");
        let mut loader = GxLoader::new();
        let vars = VarSpace::sys_init().assert();
        let spc = loader
            .parse_file("./_gal/work.gxl", false, test_opt(), &vars)?
            .assemble()
            .assert();
        let result = spc
            .exec(
                vec!["default".into()],
                vec!["trans1".into()],
                Some(false),
                false,
                VarSpace::default(),
            )
            .await;
        match result {
            Ok(_) => {
                panic!("need fail!");
            }
            Err(e) => {
                report_gxl_error(e);
            }
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn example_dryrun() -> RunResult<()> {
        once_init_log();
        let _dir = WorkDirWithLock::change("./examples/dryrun");
        let mut loader = GxLoader::new();
        let vars = VarSpace::sys_init().assert();
        let spc = loader
            .parse_file("./_gal/work.gxl", false, test_opt(), &vars)?
            .assemble()
            .assert();
        let dryrun = true;
        spc.exec(
            vec!["default".into()],
            vec!["start".into()],
            Some(false),
            dryrun,
            VarSpace::default(),
        )
        .await?;

        let dryrun = false;
        let fail = spc
            .exec(
                vec!["default".into()],
                vec!["start".into()],
                Some(false),
                dryrun,
                VarSpace::default(),
            )
            .await;
        assert!(fail.is_err());
        Ok(())
    }
}
