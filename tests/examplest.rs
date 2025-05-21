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

    #[tokio::test]
    async fn example_read() -> RunResult<()> {
        once_init_log();
        let _dir = ScopedRunDir::new("./examples/read");
        let mut loader = GxLoader::new();
        let spc =
            GxlSpace::try_from(loader.parse_file("./_gal/work.gxl", false, test_opt())?).assert();
        spc.exec(vec!["default"], vec!["conf"], false).await?;
        Ok(())
    }

    #[tokio::test]
    async fn example_assert() -> RunResult<()> {
        once_init_log();
        let _dir = ScopedRunDir::new("./examples/assert");
        let mut loader = GxLoader::new();
        let spc =
            GxlSpace::try_from(loader.parse_file("./_gal/work.gxl", false, test_opt())?).assert();
        spc.exec(vec!["default"], vec!["assert_main"], false)
            .await?;
        Ok(())
    }
    #[tokio::test]
    async fn example_template() -> RunResult<()> {
        once_init_log();
        let _dir = ScopedRunDir::new("./examples/template");
        let mut loader = GxLoader::new();
        let spc =
            GxlSpace::try_from(loader.parse_file("./_gal/work.gxl", false, test_opt())?).assert();
        spc.exec(vec!["default"], vec!["conf"], false).await?;
        Ok(())
    }
}
