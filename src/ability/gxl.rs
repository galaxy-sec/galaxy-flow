use orion_error::ErrorConv;

use crate::ability::prelude::*;

use crate::const_val::gxl_const;
use crate::sec::SecValueType;
use crate::{
    runner::{GxlCmd, GxlRunner},
    util::path::WorkDir,
};

#[derive(Clone, Debug, Default, Builder, PartialEq, Getters)]
pub struct GxRun {
    run_path: String,
    gxl_path: String,
    env_conf: String,
    env_isolate: bool,
    flow_cmd: Vec<String>,
}
impl GxRun {
    pub fn new<S>(
        run_path: S,
        gxl_path: S,
        env_conf: S,
        flow_cmd: Vec<S>,
        env_isolate: bool,
    ) -> Self
    where
        S: Into<String> + Clone,
    {
        Self {
            run_path: run_path.into(),
            gxl_path: gxl_path.into(),
            env_conf: env_conf.into(),
            flow_cmd: flow_cmd.iter().map(|x| x.clone().into()).collect(),
            env_isolate,
        }
    }
}
#[async_trait]
impl AsyncRunnableTrait for GxRun {
    async fn async_exec(&self, mut ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        ctx.append("gx.run");
        let mut action = Action::from("gx.run");
        let dryrun = if let Some(SecValueType::Bool(dryrun)) = vars_dict.get(gxl_const::CMD_DRYRUN)
        {
            *dryrun.value()
        } else {
            false
        };
        let mod_update =
            if let Some(SecValueType::Bool(mod_up)) = vars_dict.get(gxl_const::CMD_MODUP) {
                *mod_up.value()
            } else {
                false
            };

        let exp = EnvExpress::from_env_mix(vars_dict.global().clone());
        let cmd = GxlCmd {
            env: exp.eval(&self.env_conf)?,
            flow: self.flow_cmd.clone(),
            debug: 0,
            conf: Some(exp.eval(&self.gxl_path)?),
            log: None,
            quiet: ctx.quiet(),
            cmd_arg: String::new(),
            dryrun,
            mod_update,
        };
        let run_path = exp.eval(&self.run_path)?;
        let _g = WorkDir::change(run_path)
            .owe_res()
            .with(self.run_path().clone())?;
        debug!(target:ctx.path(), "{:#?}", cmd);
        let sub_var_space = VarSpace::inherit_init(vars_dict.clone(), self.env_isolate)?;
        GxlRunner::run(cmd, sub_var_space).await.err_conv()?;
        action.finish();
        Ok(TaskValue::from((vars_dict, ExecOut::Action(action))))
    }
}
impl ComponentMeta for GxRun {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.gxl")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ability::*;

    #[ignore = "will change current run dir"]
    #[tokio::test(flavor = "current_thread")]
    async fn gxl_run_test() {
        let (context, def) = ability_env_init();
        let res = GxRun::new(
            "./examples/assert",
            "_gal/work.gxl",
            "default",
            vec!["assert_main"],
            true,
        );
        res.async_exec(context, def).await.unwrap();
    }
}
