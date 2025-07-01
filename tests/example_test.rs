//extern crate galaxy_flow;
//#[cfg(feature = "res_depend_test")]
mod tests {
    use galaxy_flow::err::report_gxl_error;
    use galaxy_flow::execution::VarSpace;
    use galaxy_flow::util::path::{WorkDir, WorkDirWithLock};
    use galaxy_flow::{
        components::gxl_spc::GxlSpace, err::RunResult, expect::ShellOption, infra::once_init_log,
        GxLoader,
    };
    use orion_error::TestAssert;

    fn test_opt() -> ShellOption {
        ShellOption {
            quiet: false,
            ..Default::default()
        }
    }

    use std::env;
    use std::path::PathBuf;

    pub struct ScopedRunDir {
        original_dir: PathBuf,
    }

    impl ScopedRunDir {
        pub fn new(new_dir: &str) -> std::io::Result<Self> {
            let original_dir = env::current_dir()?;
            env::set_current_dir(new_dir)?;
            Ok(Self { original_dir })
        }
    }

    impl Drop for ScopedRunDir {
        fn drop(&mut self) {
            let _ = env::set_current_dir(&self.original_dir);
        }
    }

    #[ignore = "reason"]
    #[tokio::test(flavor = "current_thread")]
    async fn example_read() -> RunResult<()> {
        //once_init_log();
        let _dir = WorkDirWithLock::change("./examples/read");
        let vars = VarSpace::sys_init().assert();
        let mut loader = GxLoader::new();
        let spc =
            GxlSpace::try_from(loader.parse_file("./_gal/work.gxl", false, test_opt(), &vars)?)
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

    #[ignore = "reason"]
    #[tokio::test(flavor = "current_thread")]
    async fn example_assert() -> RunResult<()> {
        //once_init_log();
        let _dir = WorkDirWithLock::change("./examples/assert");
        let vars = VarSpace::sys_init().assert();
        let mut loader = GxLoader::new();
        let spc =
            GxlSpace::try_from(loader.parse_file("./_gal/work.gxl", false, test_opt(), &vars)?)
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
    #[ignore = "reason"]
    #[tokio::test(flavor = "current_thread")]
    async fn example_template() -> RunResult<()> {
        //jonce_init_log();
        let _dir = WorkDirWithLock::change("./examples/template");
        let mut loader = GxLoader::new();
        let vars = VarSpace::sys_init().assert();
        let spc =
            GxlSpace::try_from(loader.parse_file("./_gal/work.gxl", false, test_opt(), &vars)?)
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
        let spc =
            GxlSpace::try_from(loader.parse_file("./_gal/work.gxl", false, test_opt(), &vars)?)
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
        let spc =
            GxlSpace::try_from(loader.parse_file("./_gal/work.gxl", false, test_opt(), &vars)?)
                .assert();
        let dryrun = true;
        let _ = spc
            .exec(
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
