use orion_error::WithContext;

use crate::ability::prelude::*;

use crate::components::gxl_spc::GxlSpace;
use crate::expect::{LogicScope, ShellOption};
use crate::model::components::gxl_utls::take_mod_obj;
use crate::rg_sh;

use crate::traits::DependTrait;
use crate::traits::Setter;
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
        let (t_mod, act_name) = take_mod_obj(mod_name, self.name.as_str());
        if let Some(act) = src.mods().get(&t_mod).and_then(|m| m.acts().get(&act_name)) {
            Ok(self.clone_from(act, t_mod))
        } else {
            error!("activity not found: {}.{}", t_mod, act_name);
            AssembleError::err_from_domain(AssembleReason::Miss(format!(
                "activity: {}.{}",
                t_mod, act_name
            )))
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

impl RunnableTrait for ActCall {
    fn exec(&self, mut ctx: ExecContext, def: &mut VarsDict) -> EOResult {
        ctx.append("@");
        match &self.act {
            Some(act) => act.exec(ctx, def),
            None => ExecError::err_from_domain(ExecReason::Depend(format!(
                "act call less{}",
                self.name
            ))),
        }
    }
}

#[derive(Debug, Default, Builder, PartialEq, Clone)]
pub struct Activity {
    host: String,
    dto: ActivityDTO,
}
#[derive(Clone, Debug, Builder, PartialEq, Default)]
pub struct ActivityDTO {
    pub name: String,
    pub executer: String,
    pub expect: ShellOption,
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

impl RunnableTrait for Activity {
    fn exec(&self, ctx: ExecContext, def: &mut VarsDict) -> EOResult {
        self.exec_cmd(ctx, def, &self.dto)
        // Ok(ExecOut::Ignore)
    }
}
impl ComponentRunnable for Activity {
    fn meta(&self) -> RgoMeta {
        RgoMeta::build_activity(self.dto.name.as_str())
    }
}

impl Activity {
    pub fn dto_new(dto: ActivityDTO) -> Self {
        Activity {
            host: String::new(),
            dto,
        }
    }
    pub fn set_host(&mut self, host: String) {
        self.host = host;
    }
    pub fn merge_prop(&mut self, props: Vec<Property>) {
        let default_props = self.dto.props.clone();
        self.dto.props = props;
        for prop in default_props {
            if !self.dto.props.iter().any(|x| x.key == prop.key) {
                self.dto.props.push(prop);
            }
        }
    }
    fn execute_impl(
        &self,
        mut ctx: ExecContext,
        def: &mut VarsDict,
        dto: &ActivityDTO,
    ) -> EOResult {
        ctx.append(format!("{}.{}", self.host, dto.name));
        debug!(target: ctx.path(),"actcall");
        let mut task = Task::from(dto.name.as_str());
        //let mut map = def.export();
        let mut dict = def.clone();
        for prop in &dto.props {
            let mut key = prop.key.clone();
            key.make_ascii_uppercase();
            dict.set(&key, prop.val.clone());
        }
        let mut r_with = WithContext::want("run shell");
        r_with.with("exec", dto.executer.clone());
        r_with.with("dto", format!("{:?}", dto.props));
        let exp = EnvExpress::from_env_mix(dict);
        let cmd = exp.eval(&dto.executer).with(&r_with)?;
        let mut opt = self.dto.expect.clone();
        opt.outer_print = *ctx.cmd_print();
        debug!(target: ctx.path(),"cmd: {}, opt:{:?}", cmd,opt);
        rg_sh!(LogicScope::Outer, ctx.path(), &cmd, &opt, &exp).with(&r_with)?;
        task.finish();
        Ok(ExecOut::Task(task))
    }
    pub fn exec_cmd(&self, ctx: ExecContext, def: &mut VarsDict, dto: &ActivityDTO) -> EOResult {
        self.execute_impl(ctx, def, dto)
    }
}

impl DependTrait<&GxlSpace> for Activity {
    fn assemble(self, _mod_name: &str, _src: &GxlSpace) -> AResult<Self> {
        Ok(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use orion_error::TestAssert;

    use crate::ability::ability_env_init;

    use super::*;

    #[ignore]
    #[test]
    fn act_test() {
        let (context, mut def) = ability_env_init();
        let expect = ShellOption::default();
        let mut dto = ActivityDTOBuilder::default()
            .name("os.copy".into())
            .executer("./extern/gxl-lab-0.2.8/mods/os/copy_act.sh ${_FUN} ${SRC} ${DST} ".into())
            .expect(expect)
            .props(Vec::new())
            .build()
            .unwrap();
        dto.append_prop(Property {
            key: "src".into(),
            val: "./example/conf/options/copy.txt".into(),
        });
        dto.append_prop(Property {
            key: "dst".into(),
            val: "./example/conf/used/copy_3.txt".into(),
        });

        let act = Activity::dto_new(dto.clone());
        let result = act.exec(context.clone(), &mut def).assert();
        match result {
            ExecOut::Task(task) => {
                assert_eq!(task.name(), "os.copy");
            }
            _ => unreachable!(),
        }
        //let mut dto = ActCall::default();
        //dto.name = "deletage".to_string();
    }
}
