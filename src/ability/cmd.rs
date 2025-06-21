use colored::Colorize;

use crate::{
    ability::prelude::*, execution::runnable::AsyncDryrunRunnableTrait, expect::LogicScope,
};

#[derive(Clone, Debug, Default, Builder, PartialEq, Getters)]
pub struct GxCmd {
    dto: GxCmdDto,
}
#[derive(Clone, Debug, Builder, PartialEq, Default)]
pub struct GxCmdDto {
    pub cmd: String,
    pub expect: ShellOption,
}
impl GxCmdDto {
    pub fn update(&mut self, vars_dict: &VarSpace) -> ExecResult<()> {
        let ee = EnvExpress::from_env_mix(vars_dict.global().clone());
        self.cmd = ee.eval(&self.cmd)?;
        Ok(())
    }
}
#[async_trait]
impl AsyncDryrunRunnableTrait for GxCmd {
    async fn async_exec_with_dryrun(
        &self,
        ctx: ExecContext,
        vars_dict: VarSpace,
        is_dryrun: bool,
    ) -> VTResult {
        if *ctx.dryrun() && is_dryrun {
            let mut action = Action::from("gx.cmd");
            let buffer = format!(
                "Warning: It is currently in a trial operation environment!\n{}: {}",
                self.dto().cmd,
                "执行成功"
            );
            println!("{}", buffer.yellow().bold());
            action.stdout = buffer;
            action.finish();
            Ok((vars_dict, ExecOut::Action(action)))
        } else {
            self.execute_impl(&self.dto.cmd, ctx, vars_dict)
        }
    }
}
impl ComponentMeta for GxCmd {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.cmd")
    }
}

impl GxCmd {
    pub fn new(forword: String) -> Self {
        let dto = GxCmdDto {
            cmd: forword,
            ..Default::default()
        };
        Self::dto_new(dto)
    }
    pub fn dto_new(dto: GxCmdDto) -> Self {
        GxCmd { dto }
    }
    fn execute_impl(&self, cmd: &String, mut ctx: ExecContext, vars_dict: VarSpace) -> VTResult {
        ctx.append("gx.cmd");
        let mut action = Action::from("gx.cmd");
        trace!(target:ctx.path(),"cmd:{}", cmd);
        let exp = EnvExpress::from_env_mix(vars_dict.global().clone());
        //let exe_cmd = ee.parse(cmd)?;

        let mut expect = self.dto.expect.clone();
        expect.outer_print = *ctx.cmd_print();
        let res = gxl_sh!(
            LogicScope::Outer,
            ctx.tag_path("cmd").as_str(),
            &cmd,
            &expect,
            &exp
        );
        match res {
            Ok((stdout, stderr)) => {
                let out = String::from_utf8(stdout).map_err(|e| ExecReason::Io(e.to_string()))?;
                let err = String::from_utf8(stderr).map_err(|e| ExecReason::Io(e.to_string()))?;
                action.stdout = out.clone();
                if !action.stdout.is_empty() {
                    action.stdout = format!("{}\n{}", out, err);
                } else {
                    action.stdout = err;
                }
            }
            Err(error) => {
                action.stdout = error.to_string();
            }
        }
        action.finish();
        Ok((vars_dict, ExecOut::Action(action)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ability::*, traits::Setter};

    #[tokio::test]
    async fn cmd_test() {
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/example/conf");
        let res = GxCmd::new(
          "if test ! -L  ${CONF_ROOT}/used/link2.txt ; then ln -s ${CONF_ROOT}/options/link.txt  ${CONF_ROOT}/used/link2.txt ; fi ".into()
          ) ;
        res.async_exec_with_dryrun(context, def, false).await;
    }
}
