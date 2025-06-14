use orion_error::ErrorConv;

use crate::ability::prelude::*;

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
    async fn async_exec(&self, mut ctx: ExecContext, vars_dict: VarSpace) -> VTResult {
        ctx.append("gx.run");
        let mut task = Task::from("gx.run");
        let exp = EnvExpress::from_env_mix(vars_dict.global().clone());
        let cmd = GxlCmd {
            env: exp.eval(&self.env_conf)?,
            flow: self.flow_cmd.clone(),
            debug: 0,
            conf: Some(exp.eval(&self.gxl_path)?),
            log: None,
            cmd_print: true,
        };
        let run_path = exp.eval(&self.run_path)?;
        let _g = WorkDir::change(run_path)
            .owe_res()
            .with(self.run_path().clone())?;
        debug!(target:ctx.path(), "{:#?}", cmd);

        let sub_var_space = VarSpace::inherit_init(vars_dict.clone(), self.env_isolate)?;
        GxlRunner::run(cmd, sub_var_space).await.err_conv()?;
        task.finish();
        Ok((vars_dict, ExecOut::Task(task)))
    }
}
impl ComponentMeta for GxRun {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::build_ability("gx.gxl")
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
