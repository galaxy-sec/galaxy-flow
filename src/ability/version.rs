use crate::ability::prelude::*;

use crate::execution::runnable::ComponentMeta;
use crate::parser::domain::take_version;
use crate::traits::Setter;
use std::cmp::Ordering;
use std::fmt;
use std::fs::{self, File};
use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub struct Version {
    main: i32,
    feature: i32,
    bugfix: i32,
    build: Option<i32>,
}

impl Version {}
impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.main == other.main
            && self.feature == other.feature
            && self.bugfix == other.bugfix
            && self.build == other.build
        {
            Some(Ordering::Equal)
        } else if self.main > other.main {
            Some(Ordering::Greater)
        } else if self.main < other.main {
            Some(Ordering::Less)
        } else if self.feature > other.feature {
            Some(Ordering::Greater)
        } else if self.feature < other.feature {
            Some(Ordering::Less)
        } else if self.bugfix > other.bugfix {
            Some(Ordering::Greater)
        } else if self.bugfix < other.bugfix {
            Some(Ordering::Less)
        } else if self.build > other.build {
            Some(Ordering::Greater)
        } else if self.build < other.build {
            Some(Ordering::Less)
        } else {
            None
        }
    }
}

impl Version {
    pub fn new(main: i32, feature: i32, bugfix: i32, build: Option<i32>) -> Self {
        Version {
            main,
            feature,
            bugfix,
            build,
        }
    }
    pub fn auto(&mut self, inc: &VerInc) {
        match *inc {
            VerInc::Main => {
                self.main += 1;
                self.feature = 0;
                self.bugfix = 0;
                self.build = self.build.map(|x| x + 1);
            }
            VerInc::Feature => {
                self.feature += 1;
                self.bugfix = 0;
                self.build = self.build.map(|x| x + 1);
            }
            VerInc::Bugfix => {
                self.bugfix += 1;
                self.build = self.build.map(|x| x + 1);
            }
            VerInc::Build => {
                self.build = self.build.map(|x| x + 1);
            }
            _ => {}
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.build {
            Some(build) => write!(
                f,
                "{}.{}.{}.{}",
                self.main, self.feature, self.bugfix, build
            ),
            None => write!(f, "{}.{}.{}", self.main, self.feature, self.bugfix),
        }
    }
}

#[derive(Debug, Builder, PartialEq, Clone)]
pub struct RgVersion {
    file: String,
    export: String,
    verinc: VerInc,
}
impl RgVersion {
    pub fn new(file: String) -> RgVersion {
        RgVersion {
            file,
            export: "VERSION".into(),
            verinc: VerInc::Build,
        }
    }
}
#[async_trait]
impl AsyncRunnableTrait for RgVersion {
    async fn async_exec(&self, mut ctx: ExecContext, mut dict: VarSpace) -> VTResult {
        ctx.append("version");
        let exp = EnvExpress::from_env_mix(dict.global().clone());
        let file_path = exp.eval(&self.file)?;
        debug!(target: ctx.path(),"version file:{}", file_path);
        let data = fs::read_to_string(file_path.as_str())
            .owe_biz()
            .with(format!("version file ({}) ", file_path))?;
        if let Ok((a, b, c, d)) = take_version(&mut data.as_str()) {
            let mut ver = Version::new(a, b, c, d);
            ver.auto(&self.verinc);
            dict.global_mut()
                .set(&self.export.to_uppercase(), format!("{}", &ver));
            let mut file = File::create(file_path.as_str()).owe_res()?;
            file.write_all(ver.to_string().as_bytes()).owe_res()?;
            Ok((dict, ExecOut::Ignore))
        } else {
            Err(ExecReason::Depend("version file parse failed!".to_string()).into())
        }
    }
}

pub fn parse_version(data: &str) -> ExecResult<Version> {
    let mut xdata = data;
    let (a, b, c, d) =
        take_version(&mut xdata).owe(ExecReason::Args("version parse failed".to_string()))?;
    Ok(Version::new(a, b, c, d))
}

impl ComponentMeta for RgVersion {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.ver")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerInc {
    Build,
    Bugfix,
    Feature,
    Main,
    Null,
}

#[cfg(test)]
mod tests {
    use fs::File;

    use crate::types::AnyResult;

    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn version_test() {
        let mut file = File::create("./tests/tmp_version.txt").unwrap();
        file.write_all(b"0.1.0.0").unwrap();
        let ver = RgVersion::new("./tests/tmp_version.txt".into());
        let ctx = ExecContext::default();
        let def = VarSpace::default();
        ver.async_exec(ctx.clone(), def).await.unwrap();
    }

    #[test]
    fn basic_version_test() -> AnyResult<()> {
        let versions = vec![
            (
                "3.4.5",
                Version {
                    main: 3,
                    feature: 4,
                    bugfix: 5,
                    build: None,
                },
            ),
            (
                "6.7.8.9",
                Version {
                    main: 6,
                    feature: 7,
                    bugfix: 8,
                    build: Some(9),
                },
            ),
        ];

        for (input, expected) in versions {
            let parsed = parse_version(input)?;
            assert_eq!(parsed, expected);
        }
        Ok(())
    }

    #[test]
    fn version_increment_test() {
        // 主版本递增
        let mut ver = Version {
            main: 1,
            feature: 1,
            bugfix: 1,
            build: Some(0),
        };
        ver.auto(&VerInc::Main);
        assert_eq!(ver.main, 2);
        assert_eq!(ver.feature, 0);
        assert_eq!(ver.bugfix, 0);

        // 特性版本递增
        let mut ver = Version {
            main: 1,
            feature: 2,
            bugfix: 1,
            build: None,
        };
        ver.auto(&VerInc::Feature);
        assert_eq!(ver.feature, 3);
        assert_eq!(ver.bugfix, 0);

        // 修复版本递增
        let mut ver = Version {
            main: 1,
            feature: 2,
            bugfix: 3,
            build: Some(4),
        };
        ver.auto(&VerInc::Bugfix);
        assert_eq!(ver.bugfix, 4);
        assert_eq!(ver.feature, 2);

        // 构建版本递增
        let mut ver = Version {
            main: 1,
            feature: 2,
            bugfix: 3,
            build: Some(5),
        };
        ver.auto(&VerInc::Build);
        assert_eq!(ver.build.unwrap(), 6);
    }
}
