use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::ability::prelude::*;
use crate::components::RgVars;
use crate::traits::Setter;
use crate::var::VarDict;

use derive_more::{Display, From};
use ini::Ini;
use orion_common::friendly::New2;
use orion_error::WithContext;
use orion_syspec::vars::ValueDict;
use orion_syspec::{error::ToErr, system::ModulesList, types::Configable};

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
    fn impl_ini(&self, ctx: ExecContext, file_path: &Path) -> ExecResult<VarDict> {
        let file = Ini::load_from_file(file_path).map_err(|e| {
            ExecReason::Args(format!(
                "load ini file:[{}] error: {}",
                file_path.display(),
                e
            ))
        })?;
        let mut vars = RgVars::default();
        for (_, prop) in file.iter() {
            for (k, v) in prop.iter() {
                let str_k = k.trim().to_string();
                let str_v = v.trim().to_string();
                vars.append(RgProp::new(str_k, str_v));
            }
        }
        let mut dict = VarDict::global_new();
        vars.export_props(ctx, &mut dict, "")?;
        Ok(dict)
    }
    pub fn execute(&self, mut ctx: ExecContext, mut def: VarSpace) -> VTResult {
        ctx.append("gx.read_file");
        let exp = EnvExpress::from_env_mix(def.globle().clone());
        let file = self.file.clone();
        let file_path = PathBuf::from(exp.eval(&file)?);
        let mut cur_dict = if file_path.extension() == PathBuf::from("*.ini").extension() {
            self.impl_ini(ctx, &file_path)?
        } else if file_path.extension() == PathBuf::from("*.json").extension() {
            self.impl_json(ctx, &file_path)?
        } else if file_path.extension() == PathBuf::from("*.yml").extension() {
            self.impl_entity(ctx, &file_path)?
        } else {
            return ExecReason::Args(format!("not support format :{}", file_path.display()))
                .err_result();
        };
        if let Some(name) = &self.name {
            cur_dict.set_name(name);
            def.nameds_mut().insert(name.clone(), cur_dict);
        } else {
            def.globle_mut().merge_dict(cur_dict);
        }
        Ok((def, ExecOut::Ignore))
    }

    fn impl_toml_mlist(&self, _ctx: ExecContext, file_path: &Path) -> ExecResult<VarDict> {
        let data: ModulesList = ModulesList::from_conf(file_path).owe_data()?;
        let x = data.export();
        Ok(VarDict::from(x))
    }

    fn impl_toml_vdict(&self, _ctx: ExecContext, file_path: &Path) -> ExecResult<VarDict> {
        let data = ValueDict::from_conf(file_path).owe_data()?;
        Ok(VarDict::from(data))
    }

    fn impl_entity(&self, ctx: ExecContext, file_path: &PathBuf) -> ExecResult<VarDict> {
        let mut err_ctx = WithContext::want("load toml exchange data");
        err_ctx.with_path("path", file_path);
        if let Some(x) = &self.entity {
            let entity = ReadEntity::from_str(x.as_str()).owe_data()?;
            match entity {
                ReadEntity::MList => self.impl_toml_mlist(ctx, file_path),
                ReadEntity::VDict => self.impl_toml_vdict(ctx, file_path),
            }
        } else {
            ExecReason::Miss("entity".into()).err_result()
        }
    }
    fn impl_json(&self, _ctx: ExecContext, file_path: &PathBuf) -> ExecResult<VarDict> {
        let mut err_ctx = WithContext::want("load toml exchange data");
        err_ctx.with_path("path", file_path);
        let content = std::fs::read_to_string(PathBuf::from(file_path))
            .owe_data()
            .with(&err_ctx)?;
        let data: serde_json::Value = serde_json::from_str(content.as_str())
            .owe_data()
            .with(&err_ctx)?;
        let mut dict = VarDict::default();
        match data {
            serde_json::Value::Array(values) => {
                for (i, v) in values.iter().enumerate() {
                    dict.set(i.to_string(), v.to_string())
                }
            }
            serde_json::Value::Object(map) => {
                for (k, v) in map.iter() {
                    dict.set(k.as_str(), v.to_string())
                }
            }
            _ => {
                return ExecReason::NoVal("json format error".into())
                    .err_result()
                    .with(err_ctx)
            }
        }
        Ok(dict)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ability::{read::integra::ReadMode, *};

    #[tokio::test]
    async fn read_ini_test() {
        let (context, mut def) = ability_env_init();
        def.globle_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/examples/read");
        let mut dto = FileDTO::default();
        dto.file = String::from("${CONF_ROOT}/var.ini");
        let res = GxRead::from(ReadMode::from(dto));
        res.async_exec(context, def).await.unwrap();
    }
}
