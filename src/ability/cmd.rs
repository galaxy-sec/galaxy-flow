use crate::{ability::prelude::*, expect::LogicScope};

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
impl AsyncRunnableTrait for GxCmd {
    async fn async_exec(&self, ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        self.execute_impl(&self.dto.cmd, ctx, vars_dict)
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
    fn execute_impl(&self, cmd: &String, mut ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        ctx.append("gx.cmd");
        let mut action = Action::from("gx.cmd");
        trace!(target:ctx.path(),"cmd:{}", cmd);
        let exp = EnvExpress::from_env_mix(vars_dict.global().clone());
        let exe_cmd = exp.eval(cmd)?;

        let mut expect = self.dto.expect.clone();
        // 若未设置全局输出模式，则使用局部模式
        if let Some(quiet) = ctx.quiet() {
            expect.quiet = quiet;
        }

        let res = gxl_sh!(
            LogicScope::Outer,
            ctx.tag_path("cmd").as_str(),
            &exe_cmd,
            &expect,
            &exp
        );
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
    use crate::{ability::*, traits::Setter};

    #[tokio::test]
    async fn cmd_test() {
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/tests/material");
        let res = GxCmd::new(
          "if test ! -L  ${CONF_ROOT}/ability.bak; then ln -s ${CONF_ROOT}/ability.gxl ${CONF_ROOT}/ability.bak;  fi ".into()
          ) ;
        let _ = res.async_exec(context, def).await.assert("dryrun");
    }

    #[tokio::test]
    async fn cmd_test_err() {
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/example/conf");
        //syntax error;
        let res = GxCmd::new(
          "if test ! -L  ${CONF_ROOT}/used/link2.txt ; then ln -s ${CONF_ROOT}/options/link.txt  ${CONF_ROOT}/used/link2.txt ; i ".into()
          ) ;
        let result = res.async_exec(context, def).await;
        assert!(result.is_err())
    }
}
