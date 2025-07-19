use std::{path::PathBuf, str::FromStr};

use crate::sec::SecFrom;
use crate::traits::Setter;
use crate::{ability::prelude::*, sec::SecValueType};

use derive_more::{Display, From};
use orion_common::serde::{IniAble, JsonAble, Yamlable};
use orion_error::{ToStructError, UvsLogicFrom};
use orion_variate::vars::ValueType;

#[derive(Clone, Debug, PartialEq, From, Display)]
//#[display("Java EE: {}")]
pub enum ReadEntity {
    #[display("mod_list")]
    MList,
    #[display("var_dict")]
    VDict,
}
impl FromStr for ReadEntity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mod_list" => Ok(Self::MList),
            "var_dict" => Ok(Self::VDict),
            _ => Err(String::from(s)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Builder)]
pub struct FileDTO {
    pub file: String,
    #[builder(default = "None")]
    pub name: Option<String>,
    #[builder(default = "None")]
    pub entity: Option<String>,
}

impl FileDTO {
    pub fn execute(&self, mut ctx: ExecContext, mut vars_dict: VarSpace) -> TaskResult {
        ctx.append("gx.read_file");
        let exp = EnvExpress::from_env_mix(vars_dict.global().clone());
        let file = self.file.clone();
        let file_path = PathBuf::from(exp.eval(&file)?);
        let values = if file_path.extension() == PathBuf::from("*.ini").extension() {
            ValueType::from_ini(&file_path).owe_data()?
            //self.impl_ini(ctx, &file_path)?
        } else if file_path.extension() == PathBuf::from("*.json").extension() {
            ValueType::from_json(&file_path).owe_data()?
            //self.impl_json(ctx, &file_path)?
        } else if file_path.extension() == PathBuf::from("*.yml").extension() {
            ValueType::from_yml(&file_path).owe_data()?
            //self.impl_entity(ctx, &file_path)?
        } else {
            return ExecReason::Args(format!("not support format :{}", file_path.display()))
                .err_result();
        };
        let sec_values = SecValueType::nor_from(values);
        match sec_values {
            SecValueType::Obj(obj) => {
                if let Some(name) = self.name.clone() {
                    vars_dict.global_mut().set(name, SecValueType::from(obj));
                } else {
                    vars_dict.global_mut().merge(obj);
                }
            }
            SecValueType::List(list) => {
                if let Some(name) = self.name.clone() {
                    vars_dict.global_mut().set(name, SecValueType::from(list));
                } else {
                    return ExecError::from_logic("list cannot set to VarSpace by no name ".into())
                        .err();
                }
            }
            _ => {
                return ExecError::from_logic("read file only support list and map ".into()).err();
            }
        }
        Ok(TaskValue::from((vars_dict, ExecOut::Ignore)))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ability::{read::integra::ReadMode, *};

    #[tokio::test]
    async fn read_ini_test() {
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/examples/read");
        let dto = FileDTO {
            file: String::from("${CONF_ROOT}/var.ini"),
            ..Default::default()
        };
        let res = GxRead::from(ReadMode::from(dto));
        res.async_exec(context, def).await.unwrap();
    }
}
