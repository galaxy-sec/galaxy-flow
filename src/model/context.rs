use std::collections::HashMap;
use std::env;

use std::sync::Arc;

use getset::{CopyGetters, Getters};

use crate::cmd::GxlCmd;
use crate::friendly::AppendAble;

#[derive(Debug, Clone, Default, Getters, CopyGetters)]
pub struct ExecContext {
    #[getset(get = "pub")]
    env_vars: HashMap<String, String>,
    #[getset(get = "pub")]
    abs_path: String,
    #[getset(get = "pub")]
    cur_path: String,
    //#[getset(get_copy = "pub")]
    //quiet: Option<bool>,
    //#[getset(get = "pub")]
    //dryrun: bool,
    #[getset(get = "pub")]
    gxl_cmd: Arc<GxlCmd>,
}
impl ExecContext {
    pub fn new(cmd: GxlCmd) -> Self {
        let cur_path = env::current_dir().unwrap();
        let cur_path = cur_path.as_path().to_str().unwrap();

        ExecContext {
            abs_path: String::from(""),
            cur_path: String::from(cur_path),
            gxl_cmd: Arc::new(cmd),
            ..Default::default()
        }
    }
    pub fn dryrun(&self) -> bool {
        self.gxl_cmd().dryrun
    }
    pub fn quiet(&self) -> bool {
        self.gxl_cmd().quiet
    }

    pub fn path(&self) -> &str {
        self.abs_path.as_str()
    }
    pub fn tag_path(&self, tag: &str) -> String {
        format!("{}:{}", tag, self.abs_path)
    }

    pub fn with_subcontext(mut self, arg: &str) -> Self {
        self.append(arg);
        self
    }
}
impl AppendAble<&str> for ExecContext {
    fn append(&mut self, now: &str) {
        self.append(now.to_string());
    }
}

impl AppendAble<&String> for ExecContext {
    fn append(&mut self, now: &String) {
        self.append(now.clone());
    }
}

impl AppendAble<String> for ExecContext {
    fn append(&mut self, now: String) {
        if self.abs_path.is_empty() {
            self.abs_path = now;
        } else {
            self.abs_path = format!("{}/{}", self.abs_path, now);
        }
    }
}
