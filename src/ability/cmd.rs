use crate::{ability::prelude::*, expect::LogicScope};

#[derive(Clone, Debug, Default, Builder, PartialEq)]
pub struct GxCmd {
    dto: GxCmdDto,
}
#[derive(Clone, Debug, Builder, PartialEq, Default)]
pub struct GxCmdDto {
    pub forword: String,
    pub expect: ShellOption,
}
impl GxCmdDto {
    pub fn update(&mut self, def: &VarSpace) -> ExecResult<()> {
        let ee = EnvExpress::from_env_mix(def.globle().clone());
        self.forword = ee.eval(&self.forword)?;
        Ok(())
    }
}
#[async_trait]
impl AsyncRunnableTrait for GxCmd {
    async fn async_exec(&self, ctx: ExecContext, def: VarSpace) -> VTResult {
        self.execute_impl(&self.dto.forword, ctx, def)
    }
}
impl ComponentMeta for GxCmd {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::build_ability("gx.cmd")
    }
}

impl GxCmd {
    pub fn new(forword: String) -> Self {
        let dto = GxCmdDto {
            forword,
            ..Default::default()
        };
        Self::dto_new(dto)
    }
    pub fn dto_new(dto: GxCmdDto) -> Self {
        GxCmd { dto }
    }
    fn execute_impl(&self, cmd: &String, mut ctx: ExecContext, def: VarSpace) -> VTResult {
        ctx.append("gx.cmd");
        let mut task = Task::from("gx.cmd");
        trace!(target:ctx.path(),"cmd:{}", cmd);
        let exp = EnvExpress::from_env_mix(def.globle().clone());
        //let exe_cmd = ee.parse(cmd)?;

        let mut expect = self.dto.expect.clone();
        expect.outer_print = *ctx.cmd_print();
        rg_sh!(
            LogicScope::Outer,
            ctx.tag_path("cmd").as_str(),
            &cmd,
            &expect,
            &exp
        )?;
        task.finish();
        Ok((def, ExecOut::Task(task)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ability::*, traits::Setter};

    #[tokio::test]
    async fn cmd_test() {
        let (context, mut def) = ability_env_init();
        def.globle_mut()
            .set("CONF_ROOT", "${RG_PRJ_ROOT}/example/conf");
        let res = GxCmd::new(
          "cd ${CONF_ROOT}/used ; if test ! -L  ./link2.txt ; then ln -s ${CONF_ROOT}/options/link.txt  ./link2.txt ; fi ".into()
          ) ;
        res.async_exec(context, def).await.unwrap();
    }
}
