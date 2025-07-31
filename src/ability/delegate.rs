use derive_more::From;
use getset::Getters;
use orion_error::ToStructError;

use crate::ability::prelude::*;

use crate::components::gxl_act::activity::Activity;
use crate::components::gxl_fun::fun::GxlFun;
use crate::components::gxl_spc::GxlSpace;
use crate::execution::runnable::AsyncRunnableArgsTrait;
use crate::model::components::gxl_utls::mod_obj_name;

use crate::primitive::{GxlAParam, GxlAParams};
use crate::traits::DependTrait;

#[derive(Clone, From)]
pub enum ActTypes {
    Fun(GxlFun),
    Act(Activity),
}
#[derive(Clone, Default, Builder, Getters)]
#[getset(get = "pub")]
pub struct ActCall {
    pub name: String,
    pub sudo: bool,
    pub actual_params: GxlAParams,
    act: Option<ActTypes>,
}

impl From<String> for ActCall {
    fn from(name: String) -> Self {
        Self {
            name,
            sudo: false,
            actual_params: GxlAParams::new(),
            act: None,
        }
    }
}
impl From<(String, Vec<GxlAParam>)> for ActCall {
    fn from(value: (String, Vec<GxlAParam>)) -> Self {
        let mut args = GxlAParams::new();
        value.1.into_iter().for_each(|x| {
            args.insert(x.name().clone(), x);
        });
        Self {
            name: value.0,
            sudo: false,
            actual_params: args,
            act: None,
        }
    }
}

impl DependTrait<&GxlSpace> for ActCall {
    fn assemble(self, mod_name: &str, src: &GxlSpace) -> AResult<Self> {
        let (find_mod, act_name) = mod_obj_name(mod_name, self.name.as_str());
        if let Some(found_mod) = src.get(&find_mod) {
            if let Some(act) = found_mod.acts().get(&act_name) {
                return Ok(ActCall::from((self.clone(), act, find_mod)));
            } else if let Some(fun) = found_mod.funs().get(&act_name) {
                for param in fun.meta().params() {
                    let found = if param.is_default() {
                        //use default actura name
                        self.actual_params
                            .get(param.name())
                            .or(self.actual_params.get("default"))
                    } else {
                        self.actual_params.get(param.name())
                    };

                    if found.is_none() && param.default_value().is_none() {
                        return AssembleReason::Miss(format!(
                            "{} for call {find_mod}.{act_name}",
                            param.name()
                        ))
                        .err_result();
                    }
                }
                return Ok(ActCall::from((self.clone(), fun, find_mod)));
            }
        }
        error!("activity not found: {find_mod}.{act_name}");
        Err(AssembleError::from(AssembleReason::Miss(format!(
            "activity: {find_mod}.{act_name}"
        ))))
    }
}

impl ActCall {
    /*
    pub fn clone_from(&self, act: &Activity, host: String) -> Self {
        let mut ins = self.clone();
        let mut cur_act = act.clone();
        cur_act.set_host(host);
        cur_act.merge_prop(self.props.clone());
        ins.act = Some(ActTypes::from(cur_act));
        ins
    }
    */
}
impl From<(Self, &Activity, String)> for ActCall {
    fn from(value: (Self, &Activity, String)) -> Self {
        let mut ins = value.0;
        let cur_act = value.1.clone();
        //cur_act.set_host(value.2);
        //cur_act.merge_prop(ins.args.clone());
        ins.act = Some(ActTypes::from(cur_act));
        ins
    }
}

impl From<(Self, &GxlFun, String)> for ActCall {
    fn from(value: (Self, &GxlFun, String)) -> Self {
        let mut ins = value.0;
        let cur_fun = value.1.clone();
        //cur_fun.set_host(host);
        //cur_fun.merge_prop(self.props.clone());
        ins.act = Some(ActTypes::from(cur_fun));
        ins
    }
}
#[async_trait]
impl AsyncRunnableTrait for ActCall {
    async fn async_exec(&self, mut ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        ctx.append("@");
        match &self.act {
            Some(act) => act.async_exec(ctx, vars_dict, &self.actual_params).await,
            None => Err(ExecError::from(ExecReason::Depend(format!(
                "act call not support :{}",
                self.name
            )))),
        }
    }
}

#[async_trait]
impl AsyncRunnableArgsTrait for ActTypes {
    async fn async_exec(&self, ctx: ExecContext, vars: VarSpace, args: &GxlAParams) -> TaskResult {
        match self {
            ActTypes::Fun(o) => o.async_exec(ctx, vars, args).await,
            ActTypes::Act(o) => o.async_exec(ctx, vars, args).await,
        }
    }
}
