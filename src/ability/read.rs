use std::io;
use std::path::PathBuf;

use crate::ability::prelude::*;
use crate::components::RgVars;
use crate::execution::runnable::ComponentMeta;
use crate::expect::{LogicScope, ShellOption};

use derive_more::From;
use ini::Ini;
use orion_common::friendly::New2;
use orion_syspec::error::ToErr;

#[derive(Debug, Default, Builder, PartialEq, Clone, Getters, From)]
pub struct GxRead {
    imp: ReadMode,
}
#[derive(Debug, PartialEq, Clone, Default, From)]
pub enum ReadMode {
    #[default]
    UNDEF,
    CMD(CmdDTO),
    FILE(FileDTO),
    STDIN(StdinDTO),
}

#[derive(Clone, Debug, PartialEq, Default, Builder)]
pub struct StdinDTO {
    pub name: String,
    pub prompt: String,
}

#[derive(Clone, Debug, PartialEq, Default, Builder)]
pub struct FileDTO {
    pub file: String,
    #[builder(default = "None")]
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default, Builder)]
pub struct CmdDTO {
    pub name: String,
    pub cmd: String,
    pub expect: ShellOption,
}

#[async_trait]
impl AsyncRunnableTrait for GxRead {
    async fn async_exec(&self, ctx: ExecContext, def: VarsDict) -> VTResult {
        self.execute_impl(ctx, def)
    }
}

impl ComponentMeta for GxRead {
    fn com_meta(&self) -> RgoMeta {
        RgoMeta::build_ability("gx.read")
    }
}
impl FileDTO {
    fn execute(&self, mut ctx: ExecContext, mut def: VarsDict) -> VTResult {
        ctx.append("gx.read_ini");
        let exp = EnvExpress::from_env_mix(def.clone());
        let file = self.file.clone();
        let file_path = PathBuf::from(exp.eval(&file)?);
        if file_path.extension() != PathBuf::from("*.ini").extension() {
            return ExecReason::Args(format!("not support format :{}", file_path.display()))
                .err_result();
        }
        let file = Ini::load_from_file(file_path.clone()).map_err(|e| {
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
        if let Some(name) = &self.name {
            let mut name_dict = VarsDict::new(name);
            vars.export_props(ctx, &mut name_dict, "")?;
            return Ok((name_dict, ExecOut::Ignore));
        } else {
            vars.export_props(ctx, &mut def, "")?;
            return Ok((def, ExecOut::Ignore));
        }
    }
}
impl StdinDTO {
    fn execute(&self, mut ctx: ExecContext, mut def: VarsDict) -> VTResult {
        ctx.append("gx.read_ini");
        let exp = EnvExpress::from_env_mix(def.clone());
        let msg = self.prompt.clone();
        let name = self.name.clone();
        let msg = exp.eval(&msg)?;
        println!("{}", msg);
        let mut buffer = String::new();
        let stdin = io::stdin(); // We get `Stdin` here.
        stdin.read_line(&mut buffer).owe_data()?;
        let mut vars = RgVars::default();
        vars.append(RgProp::new(name, buffer.trim().to_string()));
        vars.export_props(ctx, &mut def, "")?;
        return Ok((def, ExecOut::Ignore));
    }
}

impl CmdDTO {
    fn execute(&self, mut ctx: ExecContext, mut def: VarsDict) -> VTResult {
        ctx.append("gx.read_cmd");
        let exp = EnvExpress::from_env_mix(def.clone());
        let cmd = self.cmd.clone();
        let name = self.name.clone();
        let cmd = exp.eval(&cmd)?;
        let data = rg_sh!(LogicScope::Outer, ctx.path(), &cmd, &self.expect, &exp)?;
        let data_str = String::from_utf8(data)
            .map_err(|msg| ExecReason::Exp(format!("bad result {}", msg)))?;
        let mut vars = RgVars::default();
        vars.append(RgProp::new(name, data_str.trim().to_string()));
        vars.export_props(ctx, &mut def, "")?;
        return Ok((def, ExecOut::Ignore));
    }
}

impl GxRead {
    fn execute_impl(&self, ctx: ExecContext, dict: VarsDict) -> VTResult {
        match &self.imp {
            ReadMode::CMD(cmd_dto) => cmd_dto.execute(ctx, dict),
            ReadMode::FILE(ini_dto) => ini_dto.execute(ctx, dict),
            ReadMode::STDIN(stdin_dto) => stdin_dto.execute(ctx, dict),
            _ => return Err(ExecReason::Exp(String::from("not implementation")).into()),
        }
    }
}
/*
ReadMode::OxcToml => {
    let mut err_ctx = WithContext::want("load toml exchange data");
    let toml = dto
        .oxc_toml
        .clone()
        .ok_or(ExecReason::Exp(String::from("no toml")))?;
    let toml = exp.eval(&toml)?;
    err_ctx.with("toml-path", toml.clone());
    let toml_content = std::fs::read_to_string(PathBuf::from(toml.as_str()))
        .owe_data()
        .with(&err_ctx)?;
    let data: ValueDict = toml::from_str(toml_content.as_str())
        .owe_data()
        .with(&err_ctx)?;
    let mut vars = RgVars::default();
    for (k, var_def) in data.dict() {
        match var_def {
            ValueType::String(v) => {
                let str_k = k.clone();
                let str_v = v.value().to_string();
                vars.append(RgProp::new(str_k, str_v));
            }
            ValueType::Bool(v) => {
                let str_k = k.clone();
                let str_v = v.value().to_string();
                vars.append(RgProp::new(str_k, str_v));
            }
            ValueType::Int(v) => {
                let str_k = k.clone();
                let str_v = v.value().to_string();
                vars.append(RgProp::new(str_k, str_v));
            }
            ValueType::Float(v) => {
                let str_k = k.clone();
                let str_v = v.value().to_string();
                vars.append(RgProp::new(str_k, str_v));
            }
        }
    }
    vars.export_props(ctx, &mut def, "")?;
}
*/
#[cfg(test)]
mod tests {

    use super::*;
    use crate::ability::*;

    #[tokio::test]
    async fn read_cmd_test() {
        let (context, mut def) = ability_env_init();
        def.set("CONF_ROOT", "${RG_PRJ_ROOT}/example/conf");
        let mut dto = CmdDTO::default();
        dto.name = format!("RG");
        dto.cmd = format!("echo galaxy-1.0");
        let res = GxRead::from(ReadMode::from(dto));
        res.async_exec(context, def).await.unwrap();
    }
    #[tokio::test]
    async fn read_ini_test() {
        let (context, mut def) = ability_env_init();
        def.set("CONF_ROOT", "${RG_PRJ_ROOT}/examples/read");
        let mut dto = FileDTO::default();
        dto.file = String::from("${CONF_ROOT}/var.ini");
        let res = GxRead::from(ReadMode::from(dto));
        res.async_exec(context, def).await.unwrap();
    }
    #[ignore]
    #[tokio::test]
    async fn read_stdin_test() {
        let (context, mut def) = ability_env_init();
        def.set("CONF_ROOT", "${RG_PRJ_ROOT}/example/conf");
        let mut dto = StdinDTO::default();
        dto.prompt = String::from("please input you name");
        dto.name = String::from("name");
        let res = GxRead::from(ReadMode::from(dto));
        res.async_exec(context, def).await.unwrap();
    }
}
