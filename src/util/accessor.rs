use crate::const_val::gxl_const::NET_ACCS_CTRL_PATH_FILE;
use orion_common::serde::Yamlable;
use orion_variate::{
    addr::{
        access_ctrl::serv::NetAccessCtrl,
        accessor::{UniversalAccessor, UniversalConfig},
    },
    vars::{EnvDict, EnvEvalable},
};
use std::env::home_dir;

pub fn build_accessor(dict: &EnvDict) -> UniversalAccessor {
    if let Some(path) = home_dir().map(|x| x.join(NET_ACCS_CTRL_PATH_FILE)) {
        if path.exists() {
            match NetAccessCtrl::from_yml(&path) {
                Ok(ctrl) => {
                    let ctrl = ctrl.env_eval(dict);
                    return UniversalAccessor::new(UniversalConfig::default().with_ctrl(ctrl));
                }
                Err(e) => {
                    error!("load redirect conf failed!\npath:{} \n{e}", path.display());
                }
            }
        }
    }
    UniversalAccessor::new(UniversalConfig::default())
}
