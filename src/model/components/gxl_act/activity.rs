use crate::{
    ability::prelude::{Action, TaskValue},
    evaluator::Parser,
    model::components::prelude::*,
    traits::Setter,
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

#[derive(Debug, Default, Builder, Clone, Getters)]
pub struct Activity {
    meta: ActivityMeta,
    host: String,
    dto: ActivityDTO,
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
impl AsyncRunnableTrait for Activity {
    async fn async_exec(&self, ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        self.exec_cmd(ctx, vars_dict, &self.dto)
        // Ok(ExecOut::Ignore)
    }
}
impl ComponentMeta for Activity {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::from(self.dto.name.as_str())
    }
}

impl Activity {
    pub fn dto_new(dto: ActivityDTO) -> Self {
        Activity {
            host: String::new(),
            dto,
            ..Default::default()
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
        vars_dict: VarSpace,
        dto: &ActivityDTO,
    ) -> TaskResult {
        ctx.append(format!("{}.{}", self.host, dto.name));
        debug!(target: ctx.path(),"actcall");
        let mut action = Action::from(dto.name.as_str());
        //let mut map = def.export();
        let mut dict = vars_dict.clone();

        let mut default_key = if let Some(param) = &self.dto.default_param {
            param.clone()
        } else {
            "".into()
        };
        default_key.make_ascii_uppercase();
        let mut use_default_key = false;
        for prop in &dto.props {
            let mut key = prop.key.clone();
            key.make_ascii_uppercase();
            if key == "DEFAULT" {
                use_default_key = true;
                dict.global_mut().set(&default_key, prop.val.clone());
                debug!(target: ctx.path(),"set default to dict {}:{}", default_key,prop.val);
            } else {
                if key == default_key && use_default_key {
                    debug!(target: ctx.path(),"use default not {}:{} ", key,prop.val);
                    continue;
                }
                debug!(target: ctx.path(),"set to dict {}:{}", key,prop.val);
                dict.global_mut().set(&key, prop.val.clone());
            }
        }
        let mut r_with = WithContext::want("run shell");
        r_with.with("exec", dto.executer.clone());
        r_with.with("dto", format!("{:?}", dto.props));
        let exp = EnvExpress::from_env_mix(dict.global().clone());
        let cmd = exp.eval(&dto.executer).with(&r_with)?;
        let mut opt = self.dto.expect.clone();
        // 若未设置全局的输出模式，则使用局部模式
        if let Some(quiet) = ctx.quiet() {
            opt.quiet = quiet;
        }

        gxl_sh!(LogicScope::Outer, ctx.path(), &cmd, &opt, &exp).with(&r_with)?;
        action.finish();
        Ok(TaskValue::from((vars_dict, ExecOut::Action(action))))
    }
    pub fn exec_cmd(&self, ctx: ExecContext, vars_dict: VarSpace, dto: &ActivityDTO) -> TaskResult {
        self.execute_impl(ctx, vars_dict, dto)
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

    use crate::ability::ability_env_init;

    use super::*;

    #[ignore]
    #[tokio::test]
    async fn act_test() {
        let (context, def) = ability_env_init();
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
        let task_value = act.async_exec(context.clone(), def).await.assert();
        match task_value.rec() {
            ExecOut::Action(action) => {
                assert_eq!(action.name(), "os.copy");
            }
            _ => unreachable!(),
        }
        //let mut dto = ActCall::default();
        //dto.name = "deletage".to_string();
    }
}
