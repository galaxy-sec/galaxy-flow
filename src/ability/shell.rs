use chrono::Local;
use orion_common::serde::*;
use rand::Rng;
use std::path::PathBuf;

use crate::{ability::prelude::*, expect::LogicScope, traits::Setter, var::VarDict};
use getset::{Getters, MutGetters, Setters, WithSetters};
use orion_error::UvsLogicFrom;
use orion_variate::vars::ValueDict;
#[derive(Clone, Debug, Default, PartialEq, Getters, Setters, WithSetters, MutGetters)]
pub struct GxShell {
    #[getset(get = "pub", set = "pub", get_mut, set_with)]
    arg_file: Option<PathBuf>,
    #[getset(get = "pub", set = "pub", get_mut, set_with)]
    out_var: Option<String>,
    #[getset(get = "pub", set = "pub", get_mut, set_with)]
    shell: String,
    #[getset(get, set = "pub", get_mut, set_with)]
    expect: ShellOption,
}
#[async_trait]
impl AsyncRunnableTrait for GxShell {
    async fn async_exec(&self, ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        self.execute_impl(ctx, vars_dict)
    }
}
impl ComponentMeta for GxShell {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.cmd")
    }
}

impl GxShell {
    pub fn new<S: Into<String>>(shell: S) -> Self {
        Self {
            shell: shell.into(),
            ..Default::default()
        }
    }
    fn execute_impl(&self, mut ctx: ExecContext, mut vars_dict: VarSpace) -> TaskResult {
        ctx.append("gx.shell");
        let mut action = Action::from("gx.shell");
        trace!(target:ctx.path(),"shell:{}", self.shell);
        let exp = EnvExpress::from_env_mix(vars_dict.global().clone());
        let ext_cmd = exp.eval(self.shell.as_str())?;
        let mut expect = self.expect.clone();

        // 若未设置全局输出模式，则使用局部模式
        if let Some(quiet) = ctx.quiet() {
            expect.quiet = quiet;
        }
        if let Some(arg_file) = &self.arg_file {
            let dict = if arg_file.extension() == PathBuf::from("data.json").extension() {
                ValueDict::from_json(arg_file).owe_data()?
            } else if arg_file.extension() == PathBuf::from("data.yml").extension()
                || arg_file.extension() == PathBuf::from("data.yaml").extension()
            {
                ValueDict::from_yml(arg_file).owe_data()?
            } else if arg_file.extension() == PathBuf::from("data.toml").extension() {
                ValueDict::from_toml(arg_file).owe_data()?
            } else if arg_file.extension() == PathBuf::from("data.ini").extension() {
                ValueDict::from_ini(arg_file).owe_data()?
            } else {
                return ExecError::from_logic(format!(
                    "unsupport this format {}",
                    arg_file.display()
                ))
                .err();
            };
            vars_dict.global_mut().merge_dict(VarDict::from(dict));
        }
        let res = if let Some(out_var) = &self.out_var {
            let out_data_path = PathBuf::from(format!(
                "/tmp/gx_out_{out_var}_{}_{}",
                Local::now().format("%Y%m%d_%H%M%S"),
                rand::rng().random::<u32>()
            ));

            if out_data_path.exists() {
                std::fs::remove_file(&out_data_path).map_err(|e| ExecReason::Io(e.to_string()))?;
            }
            std::fs::create_dir_all(out_data_path.parent().unwrap())
                .map_err(|e| ExecReason::Io(e.to_string()))?;
            std::fs::File::create(&out_data_path).map_err(|e| ExecReason::Io(e.to_string()))?;
            // 修改命令以将输出写入 FIFO

            vars_dict
                .global_mut()
                .set(out_var, format!("{}", out_data_path.display()));
            //let exe_cmd = format!("{ext_cmd}");
            let res = gxl_sh!(
                LogicScope::Outer,
                ctx.tag_path("cmd").as_str(),
                &ext_cmd,
                &expect,
                &exp,
                vars_dict.global()
            );
            let file_out = std::fs::read_to_string(&out_data_path)
                .map_err(|e| ExecReason::Io(e.to_string()))?;
            vars_dict
                .global_mut()
                .set(out_var.as_str(), file_out.trim());
            std::fs::remove_file(out_data_path).owe_logic()?;
            res
        } else {
            gxl_sh!(
                LogicScope::Outer,
                ctx.tag_path("cmd").as_str(),
                &ext_cmd,
                &expect,
                &exp,
                vars_dict.global()
            )
        };

        match res {
            Ok((stdout, stderr)) => {
                let out = String::from_utf8(stdout).map_err(|e| ExecReason::Io(e.to_string()))?;
                let err = String::from_utf8(stderr).map_err(|e| ExecReason::Io(e.to_string()))?;
                action.stdout = out.clone();
                if !action.stdout.is_empty() {
                    action.stdout = format!("{out}\n{err}",);
                } else {
                    action.stdout = err;
                }
            }
            Err(error) => {
                action.stdout = error.to_string();
                return Err(error);
            }
        }
        action.finish();
        Ok(TaskValue::from((vars_dict, ExecOut::Action(action))))
    }
}

#[cfg(test)]
mod tests {
    use orion_error::TestAssertWithMsg;

    use super::*;
    use crate::{
        ability::*,
        traits::{Getter, Setter},
        util::OptionFrom,
    };

    #[tokio::test]
    async fn shell_args_json() {
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/tests/material");
        let res = GxShell::new("./tests/material/gx_shell/demo.sh sys app")
            .with_out_var("OUT_FILE".to_opt())
            .with_arg_file("./tests/material/gx_shell/env_args.json".to_opt());

        let TaskValue { vars, .. } = res.async_exec(context, def).await.assert("dryrun");
        assert_eq!(
            vars.global().get_copy("OUT_FILE").map(|x| x.to_string()),
            Some("DATA\ngalaxy".to_string())
        )
    }

    #[tokio::test]
    async fn shell_args_yml() {
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/tests/material");
        let res = GxShell::new("./tests/material/gx_shell/demo.sh sys app")
            .with_out_var("OUT_FILE".to_opt())
            .with_arg_file("./tests/material/gx_shell/env_args.yml".to_opt());

        let TaskValue { vars, .. } = res.async_exec(context, def).await.assert("dryrun");
        assert_eq!(
            vars.global().get_copy("OUT_FILE").map(|x| x.to_string()),
            Some("DATA\ngalaxy".to_string())
        )
    }
}
