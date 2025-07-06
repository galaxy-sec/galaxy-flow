use crate::ability::prelude::GxlVar;
use crate::ability::prelude::TaskValue;
use crate::components::gxl_act::activity::Activity;
use crate::components::gxl_flow::meta::FlowMeta;
use crate::components::gxl_spc::GxlSpace;
use crate::components::GxlEnv;
use crate::components::GxlFlow;
use crate::components::GxlProps;
use crate::model::components::prelude::*;

use crate::execution::runnable::ComponentMeta;
use crate::menu::*;
use crate::meta::*;
use contracts::requires;
use derive_getters::Getters;

use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

use super::meta::ModMeta;
pub type ModHold = Arc<GxlMod>;

pub enum ModItem {
    Env(GxlEnv),
    Flow(GxlFlow),
    Actv(Activity),
}
impl ModItem {
    pub(crate) fn bind(&mut self, mod_meta: ModMeta) {
        match self {
            ModItem::Env(o) => o.bind(mod_meta),
            ModItem::Flow(o) => o.bind(mod_meta),
            ModItem::Actv(o) => o.bind(mod_meta),
        }
    }
}

#[derive(Clone, Getters, Default)]
pub struct GxlMod {
    meta: ModMeta,
    props: GxlProps,
    env_names: Vec<MenuItem>,
    flow_names: Vec<MenuItem>,
    envs: HashMap<String, GxlEnv>,
    flows: HashMap<String, GxlFlow>,
    entrys: Vec<FlowMeta>,
    exits: Vec<FlowMeta>,
    acts: HashMap<String, Activity>,
    assembled: bool,
}

pub fn merge_mod(mut mixs: Vec<GxlMod>) -> Option<GxlMod> {
    let mut buffer = Vec::new();
    if let Some(mut target) = mixs.pop() {
        let _ = write!(&mut buffer, "{}", target.of_name());
        for x in mixs.iter().rev() {
            target.merge(x);
            let _ = write!(&mut buffer, " > {}", x.of_name());
            target.up_meta(x.meta().clone())
        }
        info!(target: "assemble","mod merge :{}", String::from_utf8(buffer).unwrap());
        return Some(target);
    }
    None
}
impl DependTrait<&GxlSpace> for GxlMod {
    fn assemble(self, _mod_name: &str, src: &GxlSpace) -> AResult<Self> {
        debug!(target : "assemble", "will assemble mod {}" , self.meta().name() );
        let mod_name = &self.meta.name();
        let mut ins = GxlMod::from(self.meta().clone());

        ins.props = self.props().clone();

        for (k, env) in self.envs {
            let ass_env = env.assemble(mod_name, src)?;
            debug_assert!(ass_env.assembled());
            ins.envs.insert(k.clone(), ass_env);
        }
        for (k, flow) in self.flows {
            let ass_flow = flow.assemble(mod_name, src)?;
            debug_assert!(ass_flow.assembled());
            if ass_flow.is_auto_entry() {
                for pre in ass_flow.meta().pre_metas() {
                    ins.entrys.push(pre.clone());
                }
                ins.entrys.push(ass_flow.meta().clone());
                for pos in ass_flow.meta().pos_metas() {
                    ins.entrys.push(pos.clone());
                }
            }
            if ass_flow.is_auto_exit() {
                for pre in ass_flow.meta().pre_metas() {
                    ins.exits.push(pre.clone());
                }
                ins.exits.push(ass_flow.meta().clone());
                for pos in ass_flow.meta().pos_metas() {
                    ins.exits.push(pos.clone());
                }
            }
            ins.flows.insert(k.clone(), ass_flow);
        }
        for (k, act) in self.acts {
            let ass_act = act.assemble(mod_name, src)?;
            debug_assert!(ass_act.assembled());
            ins.acts.insert(k.clone(), ass_act);
        }
        for item in self.flow_names {
            ins.flow_names.push(item.clone());
        }
        for item in self.env_names {
            ins.env_names.push(item.clone());
        }
        ins.assembled = true;
        debug!(target : "assemble", "assemble mod {} end!" , ins.meta().name() );

        Ok(ins)
    }
}
impl PropsTrait for GxlMod {
    fn fetch_props(&self) -> &Vec<GxlVar> {
        self.props.items()
    }
}

impl From<ModMeta> for GxlMod {
    fn from(meta: ModMeta) -> Self {
        Self {
            props: GxlProps::mod_new(meta.name()),
            meta,
            ..Default::default()
        }
    }
}

impl From<&str> for GxlMod {
    fn from(name: &str) -> Self {
        let meta = ModMeta::build_mod(name);
        Self::from(meta)
    }
}

impl GxlMod {
    pub fn load_scope_flow(&self, name: &str) -> Option<GxlFlow> {
        if let Some(flow) = self.flows.get(name) {
            return Some(flow.clone());
        }
        None
    }

    fn up_meta(&mut self, meta: ModMeta) {
        self.meta = meta;
    }
}

impl MergeTrait for GxlMod {
    fn merge(&mut self, other: &Self) {
        // Merge props using the existing append method for Vec<RgProp>
        //self.props.extend(other.props.clone());
        self.props.merge(&other.props);

        // Merge envs, overriding existing entries with the other's content
        for (name, env) in &other.envs {
            if !self.envs.contains_key(name) {
                self.envs.insert(name.clone(), env.clone());
            }
        }

        // Merge flows, overriding existing entries with the other's content
        for (name, flow) in &other.flows {
            if !self.flows.contains_key(name) {
                self.flows.insert(name.clone(), flow.clone());
            }
        }
        for (name, flow) in &other.acts {
            if !self.acts.contains_key(name) {
                self.acts.insert(name.clone(), flow.clone());
            }
        }

        self.env_names.extend(other.env_names.clone());
        self.flow_names.extend(other.flow_names.clone());
    }
}

impl ExecLoadTrait for GxlMod {
    #[requires(!self.props().meta().name().is_empty())]
    fn load_env(
        &self,
        mut ctx: ExecContext,
        sequ: &mut ExecSequence,
        args: &str,
    ) -> ExecResult<()> {
        ctx.append(self.meta.name().as_str());
        debug!(target:ctx.path(),"will load env:{}", args);
        if let Some(found) = self.envs.get(args) {
            sequ.append(AsyncComHold::from(found.clone()));
            return Ok(());
        }
        // 如果没有找到指定的环境变量，返回错误
        Err(ExecError::from(ExecReason::Miss(args.into())))
    }
    fn load_flow(
        &self,
        mut _ctx: ExecContext,
        _sequ: &mut ExecSequence,
        _name: &str,
    ) -> ExecResult<()> {
        todo!();
    }

    /*
    #[requires(self.assembled)]
    fn load_flow(&self, mut ctx: ExecContext, sequ: &mut Sequence, name: &str) -> ExecResult<()> {
        ctx.append(self.meta.name().as_str());

        match self.flows.get(name) {
            Some(found) => {
                debug!(target:ctx.path(),"will load flow:{}", name);
                sequ.append(AsyncComHold::from(found.clone()));
                debug_assert!(found.assembled());
                debug_assert!(self.assembled());
                sequ.append_mod_head(self.props.clone());
                for x in self.entrys.iter() {
                    debug_assert!(x.assembled());
                    sequ.append_mod_entry(TransableHold::from(x.clone()));
                }
                let pre_flows = found.clone_pre_flows();
                let post_flows = found.clone_post_flows();
                for flow in pre_flows.into_iter() {
                    sequ.append(AsyncComHold::from(flow));
                }
                sequ.append(AsyncComHold::from(found.clone()));
                for flow in post_flows.into_iter() {
                    sequ.append(AsyncComHold::from(flow));
                }
                for x in self.exits().iter() {
                    debug_assert!(x.assembled());
                    sequ.append_mod_exit(TransableHold::from(x.clone()));
                }
                Ok(())
            }
            None => Err(ExecError::from(ExecReason::Miss(name.into()))),
        }
    }
    */
    fn menu(&self) -> ExecResult<GxMenu> {
        let mut menu = GxMenu::default();
        let mut cur = GxMenuBuilder::default()
            .envs(self.env_names.clone())
            .flows(self.flow_names.clone())
            .build()
            .unwrap();
        menu.merge(&mut cur);
        Ok(menu)
    }
    fn of_name(&self) -> String {
        self.meta.name().clone()
    }
}
impl GxlMod {
    #[requires(self.assembled)]
    fn exec_self(&self, mut ctx: ExecContext, mut def: VarSpace) -> TaskResult {
        ctx.append(self.of_name());
        self.export_props(ctx, def.global_mut(), self.meta.name().as_str())?;
        Ok(TaskValue::from((def, ExecOut::Ignore)))
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxlMod {
    async fn async_exec(&self, ctx: ExecContext, def: VarSpace) -> TaskResult {
        self.exec_self(ctx, def)
    }
}
impl ComponentMeta for GxlMod {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::Mod(self.meta.clone())
    }
}

impl AppendAble<GxlVar> for GxlMod {
    fn append(&mut self, prop: GxlVar) {
        self.props.append(prop);
    }
}
impl AppendAble<Vec<GxlVar>> for GxlMod {
    fn append(&mut self, prop_vec: Vec<GxlVar>) {
        self.props
            .merge(&GxlProps::new(format!("{}.props", self.of_name())).with_vars(prop_vec));
    }
}

impl AppendAble<Activity> for GxlMod {
    fn append(&mut self, hold: Activity) {
        self.acts.insert(hold.gxl_meta().name().to_string(), hold);
    }
}

impl AppendAble<GxlEnv> for GxlMod {
    fn append(&mut self, hold: GxlEnv) {
        let meta = hold.meta();
        debug!(target:format!("stc/mod({})",self.meta.name()).as_str(),
            "append {:#?} {}, ",meta.class(), meta.name());
        self.env_names.push(MenuItem::new(
            meta.name().clone(),
            meta.desp(),
            meta.color(),
        ));
        self.envs.insert(meta.name().clone(), hold);
    }
}

impl AppendAble<GxlFlow> for GxlMod {
    fn append(&mut self, hold: GxlFlow) {
        let hold = hold.with_mod(self.meta.clone());
        let meta = hold.meta();
        debug!(target:format!("stc/mod({})",self.meta.name()).as_str(), "append {:#?} {} ",meta.class(), meta.name());
        let desp = meta.desp();
        self.flow_names
            .push(MenuItem::new(meta.name().clone(), desp, meta.color()));
        self.flows.insert(meta.name().clone(), hold);
    }
}

impl AppendAble<ModItem> for GxlMod {
    fn append(&mut self, now: ModItem) {
        match now {
            ModItem::Env(v) => self.append(v),
            ModItem::Flow(v) => self.append(v),
            ModItem::Actv(v) => self.append(v),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use orion_common::friendly::{MultiNew2, New2};
    use orion_error::TestAssertWithMsg;

    use crate::{
        components::{
            gxl_env::meta::EnvMeta, gxl_flow::meta::FlowMeta, gxl_spc::GxlSpace, gxl_var::GxlVar,
            GxlEnv, GxlFlow, GxlMod, GxlProps,
        },
        context::ExecContext,
        execution::sequence::ExecSequence,
        infra::{init_env, once_init_log},
        traits::{DependTrait, ExecLoadTrait},
        types::AnyResult,
        var::{SecVar, VarMeta},
    };

    #[test]
    fn test_merge_to_head_empty() {
        let mixs: Vec<GxlMod> = vec![];
        let result = merge_mod(mixs);
        assert!(result.is_none());
    }

    #[test]
    fn test_merge_to_head_single() {
        let mod1 = GxlMod::from(ModMeta::build_mod("mod1"));
        let mixs: Vec<GxlMod> = vec![mod1];
        let result = merge_mod(mixs);
        assert!(result.is_some());
    }

    #[test]
    fn test_merge_to_head_multiple() {
        let meta1 = ModMeta::build_mod("mod1");
        let mut mod1 = GxlMod::from(meta1.clone());
        mod1.append(GxlVar::new("k1", "v1"));

        let meta2 = ModMeta::new2(GxlType::Mod, "mod2".to_string());
        let mut mod2 = GxlMod::from(meta2);
        mod2.append(GxlVar::new("k2", "v2"));

        let mixs: Vec<GxlMod> = vec![mod1, mod2];

        let result = merge_mod(mixs);
        assert!(result.is_some());
        if let Some(target) = result {
            assert_eq!(target.meta.name(), "mod1");
            assert_eq!(target.props().items().len(), 2);
            assert!(target
                .props()
                .items()
                .iter()
                .any(|x| x.key() == &"K1".to_string()));
            assert!(target
                .props()
                .items()
                .iter()
                .any(|x| x.key() == &"K2".to_string()));
            //assert_eq!(target.props.get("k1"), Some(&"v1".to_string()));
        }
    }

    #[test]
    fn test_assemble_depend_boundary() -> AnyResult<()> {
        let mod_name = "mod4";
        let mod4 = GxlMod::from(ModMeta::build_mod(mod_name));

        // 不添加任何依赖项

        let assembled_mod4 = mod4.assemble(mod_name, &GxlSpace::default())?;

        // 断言检查，确保 envs、flows、acts 空
        assert!(assembled_mod4.envs.is_empty());
        Ok(())
    }

    #[test]
    fn test_assemble_depend_basic() -> AnyResult<()> {
        // 创建 mod1 并添加属性和环境变量
        let mod_name = "mod1";
        let meta_mod1 = ModMeta::build_mod(mod_name);
        let mut mod1 = GxlMod::from(meta_mod1.clone());
        let mut env1 = GxlEnv::from("env1");
        env1.append(GxlVar::new("key1", "value1"));
        mod1.append(ModItem::Env(env1));

        // 创建 mod2 并引用 mod1 的环境变量
        let mod_name2 = "mod2";
        let meta_mod2 = ModMeta::build_mod(mod_name2);
        let mut mod2 = GxlMod::from(meta_mod2.clone());

        // 假设 mod2 添加了一个依赖于 mod1 的环境变量
        let mut env2 = GxlEnv::from(EnvMeta::build_env_mix("env2", vec!["mod1.env1"]));
        env2.append(GxlVar::new("key2", "value2"));
        mod2.append(ModItem::Env(env2));

        let mut spc = GxlSpace::default();
        spc.append(mod1);
        //spc.append(mod2);

        // 调用 assemble_depend 方法
        let assembled_mod2 = mod2.assemble(mod_name2, &spc)?;

        // 断言检查：验证 mod2 是否包含了 mod1 的环境变量
        assert!(assembled_mod2.envs.contains_key("env2"));
        if let Some(env) = assembled_mod2.envs.get("env2") {
            assert!(env.props().iter().any(|x| x.key() == "KEY1"));
            assert!(env.props().iter().any(|x| x.key() == "KEY2"));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_assemble_env_success() -> AnyResult<()> {
        init_env();
        let mod_name = "mod1";
        let meta_mod1 = ModMeta::build_mod(mod_name);
        let mut mod1 = GxlMod::from(meta_mod1.clone());

        // 添加一个环境变量
        let mut env1 = GxlEnv::from("env1");
        env1.append(GxlVar::new("key1", "value1"));
        let mut var = GxlProps::new("env");
        var.insert("key3", "value1");
        env1.append(var);
        mod1.append(env1);
        mod1.append(GxlVar::new("key2", "value1"));
        let mut spc = GxlSpace::default();
        spc.append(mod1);
        spc = spc.assemble().assert("assemble");

        let ctx = ExecContext::default();
        let mut sequ = ExecSequence::from("exec");
        //mod1.assemble(mod_name, src)

        // 调用 assemble_env 方法
        spc.load_env(ctx, &mut sequ, "mod1.env1")?;

        let ctx = ExecContext::default();
        let vars = VarSpace::default();
        let TaskValue { vars, .. } = sequ.execute(ctx, vars.clone(), &spc).await.unwrap();

        println!("{:?}", vars.global().maps());
        assert_eq!(
            vars.global().maps().get(&"ENV_KEY1".to_string()),
            Some(&SecVar::new(VarMeta::Normal, "value1".to_string()))
        );
        assert_eq!(
            vars.global().maps().get(&"KEY3".to_string()),
            Some(&SecVar::new(VarMeta::Normal, "value1".to_string()))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_assemble_flow_success() -> AnyResult<()> {
        once_init_log();
        let meta1 = ModMeta::build_mod("mod1");
        let mut mod1 = GxlMod::from(meta1);
        mod1.append(GxlVar::new("k1", "v1"));
        mod1.append(GxlFlow::from("flow1"));

        let meta2 = ModMeta::build_mod("mod2");
        let mut mod2 = GxlMod::from(meta2.clone());
        mod2.append(GxlVar::new("k2", "v2"));

        // 添加一个流程
        let flow1 = GxlFlow::from(FlowMeta::build_flow_pre("flow2", "mod1.flow1"));
        mod2.append(flow1);

        // 设置自动入口流程

        let mut spc = GxlSpace::default();
        spc.append(mod1);
        spc.append(mod2);
        let work_spc = spc.assemble()?;

        let ctx = ExecContext::default();
        let mut sequ = ExecSequence::from("exec");

        // 调用 assemble_flow 方法
        work_spc.load_flow(ctx, &mut sequ, "mod2.flow2")?;

        let ctx = ExecContext::default();
        let vars = VarSpace::default();
        let TaskValue { vars, .. } = sequ.execute(ctx, vars, &work_spc).await.unwrap();

        println!("{:?}", vars.global().maps());
        assert_eq!(
            vars.global().maps().len(),
            2 //vars.maps().get(&"MOD2_K2".to_string()),

              //Some(&SecVar::new(VarMeta::Normal, "v2".to_string()))
        );
        Ok(())
    }
}
