use super::prelude::*;
use crate::ability::version::Version;
use crate::model::expect::ShellOption;
use crate::types::Hold;
use crate::util::GitTools;

#[derive(Clone, Getters)]
pub struct ParsCTX {
    git: Hold<GitTools>,
    fpath: String,
    gal_ver: Version,
    host: String,
    sh_opt: ShellOption,
}
impl ParsCTX {
    pub fn new(parent: &str, git: Hold<GitTools>, gal_ver: Version, expect: ShellOption) -> Self {
        ParsCTX {
            fpath: parent.into(),
            git,
            gal_ver,
            host: String::from("space"),
            sh_opt: expect,
        }
    }
    pub fn set_host(&mut self, host: String) {
        self.host = host;
    }

    pub fn path(&self) -> &str {
        self.fpath.as_str()
    }
    pub fn version(&self) -> Version {
        self.gal_ver.clone()
    }
}

impl AppendAble<&String> for ParsCTX {
    fn append(&mut self, now: &String) {
        self.fpath = format!("{}:{}", self.fpath, now)
    }
}

impl AppendAble<&str> for ParsCTX {
    fn append(&mut self, now: &str) {
        self.fpath = format!("{}:{}", self.fpath, now)
    }
}
