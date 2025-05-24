use crate::{
    traits::{Getter, Setter},
    var::VarDict,
    ExecReason, ExecResult,
};
use regex::{Captures, Regex};
use std::env;
#[allow(unused_imports)]
use std::io::prelude::*;

pub trait Parser<T> {
    fn eval(&self, content: T) -> ExecResult<String>;
    fn sec_eval(&self, content: T) -> ExecResult<String>;
}

pub struct EnvExpress {
    regex: Regex,
    data: VarDict,
}
impl Default for EnvExpress {
    fn default() -> Self {
        EnvExpress::from_env()
    }
}

impl EnvExpress {
    pub fn new(data: VarDict) -> EnvExpress {
        let regex = Regex::new(r"(\$\{([[:alnum:]_\.]+)\})").expect(" EnvExpress Regex new fail!");
        EnvExpress { regex, data }
    }
    #[allow(dead_code)]
    pub fn from_env() -> EnvExpress {
        let mut data = VarDict::global_new();
        for (key, value) in env::vars() {
            data.set(&key, value);
        }
        EnvExpress::new(data)
    }
    pub fn from_env_mix(map: VarDict) -> EnvExpress {
        //debug!("map: {:?}", &map);
        let mut data = VarDict::global_new();
        for (key, value) in env::vars() {
            data.set(&key, value);
        }
        data.merge_dict(map);
        EnvExpress::new(data)
    }
    pub fn insert(&mut self, key: String, val: String) {
        self.data.set(&key, val);
    }
    pub fn eval_val(&self, key: &str) -> Option<&String> {
        self.data.get(key).map(|v| v.value())
    }
    pub fn sec_val(&self, key: &str) -> Option<String> {
        self.data.get(key).map(|v| v.to_string())
    }
    pub fn safe_eval_val(&self, key: &str) -> String {
        if let Some(val) = self.eval_val(key) {
            val.clone()
        } else {
            //error!("{}", self.data);
            format!("__NO[{}]__", key)
        }
    }
    pub fn sec_eval_val(&self, key: &str) -> String {
        if let Some(val) = self.sec_val(key) {
            return val.clone();
        }
        format!("__NO[{}]__", key)
    }
}

impl Parser<&String> for EnvExpress {
    fn eval(&self, content: &String) -> ExecResult<String> {
        self.eval(content.as_str())
    }
    fn sec_eval(&self, content: &String) -> ExecResult<String> {
        self.sec_eval(content.as_str())
    }
}

impl Parser<&str> for EnvExpress {
    fn eval(&self, content: &str) -> ExecResult<String> {
        let fun = |caps: &Captures| self.safe_eval_val(&caps[2]);
        let mut target = content.to_string();
        loop {
            if self.regex.find(target.as_str()).is_none() {
                break;
            }
            target = self.regex.replace_all(target.as_str(), &fun).to_string();
        }
        if target.contains("__NO") {
            return Err(ExecReason::NoVal(target).into());
        }
        Ok(target)
    }
    fn sec_eval(&self, content: &str) -> ExecResult<String> {
        let fun = |caps: &Captures| self.sec_eval_val(&caps[2]);
        let mut target = content.to_string();
        loop {
            if self.regex.find(target.as_str()).is_none() {
                break;
            }
            target = self.regex.replace_all(target.as_str(), &fun).to_string();
        }
        if target.contains("__NO") {
            return Err(ExecReason::NoVal(target).into());
        }
        Ok(target)
    }
}

#[cfg(test)]
mod tests {
    use crate::str_map;

    use super::*;
    #[test]
    pub fn regex_test() {
        let re = Regex::new(r"(\$\{([[:alnum:]]+)\})").unwrap();
        let fun = |caps: &Captures| format!("{}", &caps[2]);
        let newc = re.replace_all("${HOME}/bin", &fun);
        assert_eq!("HOME/bin", newc);
        let newc = re.replace_all("${HOME}/bin/${USER}", &fun);
        assert_eq!("HOME/bin/USER", newc);
        let newc = re.replace_all("${HOME/bin", &fun);
        assert_eq!("${HOME/bin", newc);

        let newc = re.replace_all("{HOME}/bin", &fun);
        assert_eq!("{HOME}/bin", newc);
    }
    #[test]
    pub fn eval_test() {
        let data = str_map!(
        "HOME" => "/home/galaxy",
        "USER" => "galaxy"
        );

        let ex = EnvExpress::new(VarDict::from(data));
        assert_eq!(
            ex.eval("${HOME}/bin").unwrap(),
            String::from("/home/galaxy/bin")
        );
        assert!(ex.eval("${HOME}/bin").unwrap() != String::from("/home/galaxy1/bin"));
        assert_eq!(
            ex.eval("${HOME}/${USER}/bin").unwrap(),
            String::from("/home/galaxy/galaxy/bin")
        );
        assert_eq!(
            ex.eval("${HOME2}"),
            Err(ExecReason::NoVal("__NO[HOME2]__".to_string()).into())
        );
        assert_eq!(ex.eval(&format!("HOME2")).unwrap(), String::from("HOME2"));

        let content = format!("HOME2");
        assert_eq!(ex.eval(&content).unwrap(), String::from("HOME2"));
    }
    #[test]
    pub fn eval_test2() {
        let data = str_map!(
        "HOME"      => "/home/galaxy",
        "USER"      => "galaxy",
        "CUR_DIR"   => "/home",
        );
        let ex = EnvExpress::from_env_mix(VarDict::from(data));
        assert_eq!(
            ex.eval("${HOME}/bin").unwrap(),
            String::from("/home/galaxy/bin")
        );
        assert_eq!(
            ex.eval("${CUR_DIR}/bin").unwrap(),
            String::from("/home/bin")
        );
    }
}
