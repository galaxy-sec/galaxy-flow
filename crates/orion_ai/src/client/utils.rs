use std::{env::home_dir, path::PathBuf};

use log::info;
use orion_conf::Configable;
use orion_error::ErrorOwe;
use orion_sec::sec::{NoSecConv, SecFrom, SecValueObj, SecValueType};
use orion_variate::vars::{EnvDict, ValueDict};
use unicase::UniCase;

use crate::AiResult;

pub fn sec_value_default_path() -> PathBuf {
    galaxy_dot_path().join("sec_value.yml")
}
pub fn galaxy_dot_path() -> PathBuf {
    home_dir()
        .map(|x| x.join(".galaxy"))
        .unwrap_or(PathBuf::from("./"))
}
/// 加载API密钥字典
pub fn load_key_dict(key: &str) -> Option<EnvDict> {
    let space = load_secfile().unwrap();
    if std::env::var(key).is_err() && space.get(&UniCase::from(key)).is_none() {
        println!("miss api token {key}");
        return None;
    }
    let dict = EnvDict::from(space.no_sec());
    Some(dict)
}
