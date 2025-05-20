use crate::ability::prelude::*;
use crate::components::RgVars;
use crate::expect::{LogicScope, ShellOption};

use ini::Ini;
use orion_common::friendly::New2;
use orion_error::WithContext;
use orion_exchange::vars::{ValueDict, ValueType};
use std::io::{self};
use std::path::PathBuf;

#[derive(Debug, Default, Builder, PartialEq, Clone, Getters)]
pub struct GxRead {
    dto: RgReadDto,
}
#[derive(Debug, PartialEq, Clone, Default)]
pub enum ReadMode {
    #[default]
    UNDEF,
    CMD,
    INI,
    STDIN,
    OxcToml,
}
#[derive(Clone, Debug, PartialEq, Default, Builder)]
pub struct RgReadDto {
    pub mode: ReadMode,
    #[builder(default = "None")]
    pub name: Option<String>,
    #[builder(default = "None")]
    pub cmd: Option<String>,
    #[builder(default = "None")]
    pub stdin: Option<String>,
    #[builder(default = "None")]
    pub ini: Option<String>,
    #[builder(default = "None")]
    pub oxc_toml: Option<String>,
    pub expect: ShellOption,
}

#[async_trait]
impl AsyncRunnableTrait for GxRead {
    async fn async_exec(&self, ctx: ExecContext, def: &mut VarsDict) -> EOResult {
        self.execute_impl(&self.dto, ctx, def)
    }
}

impl ComponentRunnable for GxRead {
    fn meta(&self) -> RgoMeta {
        RgoMeta::build_ability("gx.read")
    }
}

impl GxRead {
    pub fn new() -> Self {
        let dto = RgReadDto::default();
        Self::dto_new(dto)
    }
    pub fn dto_new(dto: RgReadDto) -> Self {
        GxRead { dto }
    }
    fn execute_impl(&self, dto: &RgReadDto, mut ctx: ExecContext, def: &mut VarsDict) -> EOResult {
        ctx.append("gx.read");
        let mut task = Task::from("gx.read");
        let exp = EnvExpress::from_env_mix(def.clone());

        match dto.mode {
            ReadMode::CMD => {
                let cmd = dto
                    .cmd
                    .clone()
                    .ok_or(ExecError::from(ExecReason::Exp(String::from("no cmd"))))?;
                let name = dto
                    .name
                    .clone()
                    .ok_or(ExecReason::Exp(String::from("no name")))?;
                let cmd = exp.eval(&cmd)?;
                let data = rg_sh!(LogicScope::Outer, ctx.path(), &cmd, &self.dto.expect, &exp)?;
                let data_str = String::from_utf8(data)
                    .map_err(|msg| ExecReason::Exp(format!("bad result {}", msg)))?;
                let mut vars = RgVars::default();
                vars.append(RgProp::new(name, data_str.trim().to_string()));
                vars.export_props(ctx, def, "")?;
            }
            ReadMode::INI => {
                let ini = dto
                    .ini
                    .clone()
                    .ok_or(ExecReason::Exp(String::from("no ini")))?;
                let ini = exp.eval(&ini)?;
                let ifile = Ini::load_from_file(ini.clone()).map_err(|e| {
                    ExecReason::Args(format!("load ini file:[{}] error: {}", ini, e))
                })?;
                let mut vars = RgVars::default();
                for (_, prop) in ifile.iter() {
                    for (k, v) in prop.iter() {
                        let str_k = k.trim().to_string();
                        let str_v = v.trim().to_string();
                        vars.append(RgProp::new(str_k, str_v));
                    }
                }
                vars.export_props(ctx, def, "")?;
            }

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
                vars.export_props(ctx, def, "")?;
            }
            ReadMode::STDIN => {
                let msg = dto
                    .stdin
                    .clone()
                    .ok_or(ExecReason::Exp(String::from("stdin prompt msg")))?;
                let name = dto
                    .name
                    .clone()
                    .ok_or(ExecReason::Exp(String::from("no name")))?;
                let msg = exp.eval(&msg)?;
                println!("{}", msg);
                let mut buffer = String::new();
                let stdin = io::stdin(); // We get `Stdin` here.
                stdin.read_line(&mut buffer).owe_data()?;
                let mut vars = RgVars::default();
                vars.append(RgProp::new(name, buffer.trim().to_string()));
                vars.export_props(ctx, def, "")?;
            }
            _ => return Err(ExecReason::Exp(String::from("not implementation")).into()),
        }
        task.finish();
        Ok(ExecOut::Task(task))
    }
}
#[cfg(test)]
mod tests {
    use orion_error::TestAssert;

    use super::*;
    use crate::ability::*;

    #[tokio::test]
    async fn read_cmd_test() {
        let (context, mut def) = ability_env_init();
        def.set("CONF_ROOT", "${RG_PRJ_ROOT}/example/conf");
        let mut dto = RgReadDto::default();
        dto.mode = ReadMode::CMD;
        dto.name = Some(format!("RG"));
        dto.cmd = Some(format!("echo galaxy-1.0"));
        let res = GxRead::dto_new(dto);
        res.async_exec(context, &mut def).await.unwrap();
    }
    #[tokio::test]
    async fn read_ini_test() {
        let (context, mut def) = ability_env_init();
        def.set("CONF_ROOT", "${RG_PRJ_ROOT}/examples/read");
        let mut dto = RgReadDto::default();
        dto.mode = ReadMode::INI;
        dto.ini = Some(String::from("${CONF_ROOT}/var.ini"));
        let res = GxRead::dto_new(dto);
        res.async_exec(context, &mut def).await.unwrap();
    }
    #[ignore]
    #[tokio::test]
    async fn read_stdin_test() {
        let (context, mut def) = ability_env_init();
        def.set("CONF_ROOT", "${RG_PRJ_ROOT}/example/conf");
        let mut dto = RgReadDto::default();
        dto.mode = ReadMode::STDIN;
        dto.stdin = Some(String::from("please input you name"));
        dto.name = Some(String::from("name"));
        let res = GxRead::dto_new(dto);
        res.async_exec(context, &mut def).await.unwrap();
    }

    #[test]
    fn test_read() {
        let mut builder = RgReadDtoBuilder::default();
        builder.expect(ShellOption::default());
        builder.mode(ReadMode::UNDEF);
        builder.name(Some("key".to_string()));
        builder.build().assert();
        //build.
    }
}
