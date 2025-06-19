use std::collections::HashMap;
use std::env;

use orion_common::friendly::AppendAble;

#[derive(Debug, Clone, Default, Getters)]
pub struct ExecContext {
    env_vars: HashMap<String, String>,
    abs_path: String,
    cur_path: String,
    cmd_print: bool,
    dryrun: bool,
}
impl ExecContext {
    pub fn new(out: bool, dryrun: bool) -> Self {
        let cur_path = env::current_dir().unwrap();
        let cur_path = cur_path.as_path().to_str().unwrap();

        ExecContext {
            abs_path: String::from(""),
            cur_path: String::from(cur_path),
            env_vars: HashMap::new(),
            cmd_print: out,
            dryrun: dryrun
        }
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
