use crate::{
    ability::prelude::{Action, TaskValue},
    evaluator::VarParser,
    execution::runnable::AsyncRunnableArgsTrait,
    meta::MetaInfo,
    model::components::prelude::*,
    primitive::GxlAParams,
};
use async_trait::async_trait;
use orion_common::friendly::AppendAble;

use crate::{
    components::{gxl_mod::meta::ModMeta, gxl_spc::GxlSpace},
    context::ExecContext,
    error::AResult,
    execution::VarSpace,
    expect::{LogicScope, ShellOption},
    gxl_sh,
    meta::GxlMeta,
    traits::DependTrait,
    types::Property,
};

use super::meta::ActivityMeta;

#[derive(Debug, Builder, Clone, Getters, PartialEq)]
pub struct Activity {
    meta: ActivityMeta,
    //dto: ActivityDTO,
    assembled: bool,
}
#[derive(Clone, Debug, PartialEq, Builder, Default)]
pub struct ActivityDTO {
    pub name: String,
    pub executer: String,
    pub expect: ShellOption,
    pub default_param: Option<String>,
    pub props: Vec<Property>,
}

impl ActivityDTO {
    pub fn check(&self) -> bool {
        !self.executer.is_empty()
    }
}

impl ActivityDTO {
    pub fn append_prop(&mut self, prop: Property) {
        self.props.push(prop);
    }
}

#[async_trait]
impl AsyncRunnableArgsTrait for Activity {
    async fn async_exec(&self, ctx: ExecContext, vars: VarSpace, args: &GxlAParams) -> TaskResult {
        self.exec_cmd(ctx, vars, args)
    }
}
impl ComponentMeta for Activity {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from(self.meta().clone())
    }
}

impl Activity {
    pub fn new(meta: ActivityMeta) -> Self {
        Activity {
            meta,
            assembled: false,
        }
    }
    fn execute_impl(
        &self,
        mut ctx: ExecContext,
        vars_dict: VarSpace,
        args: &GxlAParams,
    ) -> TaskResult {
        ctx.append(self.meta().full_name());
        debug!(target: ctx.path(),"actcall");
        let mut action = Action::from(self.meta().full_name());
        //let mut map = def.export();
        let dict = vars_dict.merge_args_to(self.meta().params(), args)?;

        let mut r_with = WithContext::want("run shell");
        let exp = EnvExpress::from_env_mix(dict.global().clone());
        let cmd = exp
            .eval(dict.must_get("executer")?.to_string().as_str())
            .with(&r_with)?;
        r_with.with("exec", cmd.clone());

        //let mut opt = dict.get("expect").clone();
        let mut opt = ShellOption::new();
        // 若未设置全局的输出模式，则使用局部模式
        if let Some(quiet) = ctx.quiet() {
            opt.quiet = quiet;
        }

        gxl_sh!(
            LogicScope::Outer,
            ctx.path(),
            &cmd,
            &opt,
            &exp,
            dict.global()
        )
        .with(&r_with)?;
        action.finish();
        Ok(TaskValue::from((vars_dict, ExecOut::Action(action))))
    }
    pub fn exec_cmd(&self, ctx: ExecContext, vars_dict: VarSpace, args: &GxlAParams) -> TaskResult {
        self.execute_impl(ctx, vars_dict, args)
    }

    pub(crate) fn bind(&mut self, mod_meta: ModMeta) {
        self.meta.set_host(mod_meta);
    }
}

impl DependTrait<&GxlSpace> for Activity {
    fn assemble(self, _mod_name: &str, _src: &GxlSpace) -> AResult<Self> {
        let mut ins = self.clone();
        ins.assembled = true;
        Ok(ins)
    }
}

#[cfg(test)]
mod tests {
    use orion_error::TestAssert;
    use orion_sec::sec::{SecFrom, SecValueType};

    use crate::{
        ability::ability_env_init,
        context::ExecContext,
        primitive::{GxlAParam, GxlFParam},
        util::OptionFrom,
    };

    use super::*;

    #[tokio::test]
    async fn test_exec_cmd_basic_success() {
        ability_env_init();

        // Create activity meta
        let meta =
            ActivityMeta::build("test_activity").with_params(vec![GxlFParam::new("executer")
                .with_default_value(
                    SecValueType::nor_from("./src/model/components/gxl_act/echo.sh".to_string())
                        .to_opt(),
                )]);
        let activity = Activity::new(meta);

        // Create context
        let ctx = ExecContext::new(Some(false), false);

        // Create var space with executer
        let vars = VarSpace::default();

        let mut args = GxlAParams::new();
        activity.exec_cmd(ctx.clone(), vars.clone(), &args).assert();
        args.insert(
            "executer".to_string(),
            GxlAParam::from_val("executer", "./src/model/components/gxl_act/echo2.sh"),
        );

        activity.exec_cmd(ctx, vars, &args).assert();
    }
}
