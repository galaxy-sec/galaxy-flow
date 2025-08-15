use std::{env::home_dir, path::PathBuf};

use log::info;
use orion_common::serde::Configable;
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
pub fn load_secfile() -> AiResult<SecValueObj> {
    let env_path = std::env::var("GAL_SEC_FILE_PATH").map(PathBuf::from);
    let default = sec_value_default_path();
    let path = env_path.unwrap_or(default);
    let mut vars_dict = SecValueObj::new();
    if path.exists() {
        let dict = ValueDict::from_conf(&path).owe_logic()?;
        info!(target: "exec","  load {}", path.display());
        for (k, v) in dict.iter() {
            vars_dict.insert(
                UniCase::from(format!("SEC_{}", k.to_uppercase())),
                SecValueType::sec_from(v.clone()),
            );
        }
    }
    Ok(vars_dict)
}
