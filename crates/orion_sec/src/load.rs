use orion_variate::vars::EnvDict;

pub fn load_sec_dict() -> AiResult<EnvDict> {
    let space = load_secfile()?;
    let dict = EnvDict::from(space.no_sec());
    Ok(dict)
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
