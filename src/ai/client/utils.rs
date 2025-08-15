use crate::execution::VarSpace;
use orion_variate::vars::EnvDict;

/// 加载API密钥字典
pub fn load_key_dict(key: &str) -> Option<EnvDict> {
    let space = VarSpace::sys_init().unwrap();
    if std::env::var(key).is_err() && space.get(key).is_none() {
        println!("miss api token {key}");
        return None;
    }
    let dict = EnvDict::from(&space);
    Some(dict)
}