use crate::ability::prelude::*;

use crate::components::gxl_act::activity::Activity;
use crate::components::gxl_spc::GxlSpace;
use crate::model::components::gxl_utls::mod_obj_name;

use crate::traits::DependTrait;
use crate::types::Property;

#[derive(Clone, Debug, Default, Builder, PartialEq)]
pub struct ActCall {
    pub name: String,
    pub sudo: bool,
    pub props: Vec<Property>,
    act: Option<Activity>,
}

impl From<String> for ActCall {
    fn from(name: String) -> Self {
        Self {
            name,
            sudo: false,
            props: Vec::new(),
            act: None,
        }
    }
}
impl From<(String, Vec<Property>)> for ActCall {
    fn from(value: (String, Vec<Property>)) -> Self {
        Self {
            name: value.0,
            sudo: false,
            props: value.1,
            act: None,
        }
    }
}

impl DependTrait<&GxlSpace> for ActCall {
    fn assemble(self, mod_name: &str, src: &GxlSpace) -> AResult<Self> {
        let (t_mod, act_name) = mod_obj_name(mod_name, self.name.as_str());
        if let Some(act) = src.get(&t_mod).and_then(|m| m.acts().get(&act_name)) {
            Ok(self.clone_from(act, t_mod))
        } else {
            error!("activity not found: {t_mod}.{act_name}");
            Err(AssembleError::from(AssembleReason::Miss(format!(
                "activity: {t_mod}.{act_name}"
            ))))
        }
    }
}

impl ActCall {
    pub fn append_prop(&mut self, prop: Property) {
        self.props.push(prop);
    }
    pub fn clone_from(&self, act: &Activity, host: String) -> Self {
        let mut ins = self.clone();
        let mut cur_act = act.clone();
        cur_act.set_host(host);
        cur_act.merge_prop(self.props.clone());
        ins.act = Some(cur_act);
        ins
    }
}
#[async_trait]
impl AsyncRunnableTrait for ActCall {
    async fn async_exec(&self, mut ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        ctx.append("@");
        match &self.act {
            Some(act) => act.async_exec(ctx, vars_dict).await,
            None => Err(ExecError::from(ExecReason::Depend(format!(
                "act call less{}",
                self.name
            )))),
        }
    }
}
