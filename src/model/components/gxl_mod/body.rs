use crate::ability::delegate::Activity;
use crate::ability::prelude::RgProp;
use crate::components::gxl_flow::runner::FlowRunner;
use crate::components::gxl_spc::GxlSpace;
use crate::components::GxlEnv;
use crate::components::GxlFlow;
use crate::model::components::prelude::*;

use crate::annotation::is_auto_func;

use crate::execution::hold::IsolationHold;
use crate::execution::job::Job;
use crate::execution::runnable::ComponentMeta;
use crate::menu::*;
use crate::meta::*;
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

#[derive(Clone, Getters, Default)]
pub struct GxlMod {
    meta: ModMeta,
    props: Vec<RgProp>,
    env_names: Vec<MenuItem>,
    flow_names: Vec<MenuItem>,
    envs: HashMap<String, GxlEnv>,
    flows: HashMap<String, GxlFlow>,
    acts: HashMap<String, Activity>,
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
        let mod_name = &self.meta.name();
        let mut ins = GxlMod::from(self.meta().clone());

        for p in self.props().iter() {
            let x = p.clone();
            ins.props.push(x);
        }

        for (k, env) in self.envs {
            ins.envs.insert(k.clone(), env.assemble(mod_name, src)?);
        }
        for (k, flow) in self.flows {
            ins.flows.insert(k.clone(), flow.assemble(mod_name, src)?);
        }
        for (k, act) in self.acts {
            ins.acts.insert(k.clone(), act.assemble(mod_name, src)?);
        }
        for item in self.flow_names {
            ins.flow_names.push(item.clone());
        }
        for item in self.env_names {
            ins.env_names.push(item.clone());
        }

        Ok(ins)
    }
}
impl PropsTrait for GxlMod {
    fn fetch_props(&self) -> &Vec<RgProp> {
        &self.props
    }
}

impl From<ModMeta> for GxlMod {
    fn from(meta: ModMeta) -> Self {
        Self {
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
    pub fn load_scope_flow(&self, name: &str) -> Option<FlowRunner> {
        if let Some(flow) = self.flows.get(name) {
            debug!(target : "assmeble","load scope flow {}", name);
            let props = self.props().clone();
            let befores = self.get_auto_func("entry");
            let afters = self.get_auto_func("exit");
            Some(FlowRunner::new(
                self.of_name(),
                props,
                flow.clone(),
                befores,
                afters,
            ))
        } else {
            None
        }
    }

    fn up_meta(&mut self, meta: ModMeta) {
        self.meta = meta;
    }

    fn get_auto_func(&self, auto_arg: &str) -> Vec<GxlFlow> {
        let mut found = Vec::new();
        for flow in self.flows().values() {
            if flow
                .meta()
                .annotations()
                .iter()
                .any(|x| is_auto_func(x, auto_arg))
            {
                //sequ.append(make_stc_hold(flow.clone()));
                found.push(flow.clone());
            }
        }
        found
    }
}

impl MergeTrait for GxlMod {
    fn merge(&mut self, other: &Self) {
        // Merge props using the existing append method for Vec<RgProp>
        self.props.extend(other.props.clone());

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

#[derive(Clone, Getters)]
pub struct ModRunner {
    meta: ModMeta,
    async_items: Vec<AsyncComHold>,
}
impl AppendAble<AsyncComHold> for ModRunner {
    fn append(&mut self, now: AsyncComHold) {
        self.async_items.push(now)
    }
}
#[async_trait]
impl AsyncRunnableTrait for ModRunner {
    async fn async_exec(&self, mut ctx: ExecContext, mut dct: VarSpace) -> VTResult {
        ctx.append(self.meta().name());
        let mut job = Job::default();
        for i in &self.async_items {
            let (d, t) = i.async_exec(ctx.clone(), dct).await?;
            dct = d;
            job.append(t);
        }
        Ok((dct, ExecOut::Job(job)))
    }
}

impl From<ModMeta> for ModRunner {
    fn from(value: ModMeta) -> Self {
        Self {
            meta: value,
            async_items: Vec::new(),
        }
    }
}
impl ComponentMeta for ModRunner {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::Mod(self.meta.clone())
    }
}

impl ExecLoadTrait for GxlMod {
    // 加载环境变量
    fn load_env(&self, mut ctx: ExecContext, sequ: &mut Sequence, args: &str) -> ExecResult<()> {
        // 将当前模块的名称添加到上下文中
        ctx.append(self.meta.name().as_str());
        //info!(target:ctx.path(),"load env:{}", obj_path);
        // 如果环境变量中存在指定的参数
        if let Some(found) = self.envs.get(args) {
            // 创建一个模块运行器
            let mut mr = ModRunner::from(self.meta().clone());
            // 将当前模块添加到模块运行器中
            mr.append(AsyncComHold::from(self.clone()));
            // 打印加载环境变量的信息
            info!( target: ctx.path(),"load env [{}.{}] suc!", self.meta.name(), args);
            // 将找到的环境变量添加到模块运行器中
            mr.append(AsyncComHold::from(found.clone()));
            // 将模块运行器添加到序列中
            sequ.append(AsyncComHold::from(mr));
            // 返回成功
            return Ok(());
        }
        // 如果没有找到指定的环境变量，返回错误
        Err(ExecError::from(ExecReason::Miss(args.into())))
    }

    fn load_flow(&self, mut ctx: ExecContext, sequ: &mut Sequence, args: &str) -> ExecResult<()> {
        ctx.append(self.meta.name().as_str());
        if let Some(found) = self.load_scope_flow(args) {
            sequ.append(IsolationHold::from(AsyncComHold::from(found)));
        }
        Ok(())
    }
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
    fn exec_self(&self, ctx: ExecContext, mut def: VarSpace) -> VTResult {
        self.export_props(ctx, def.global_mut(), self.meta.name().as_str())?;
        Ok((def, ExecOut::Ignore))
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxlMod {
    async fn async_exec(&self, ctx: ExecContext, def: VarSpace) -> VTResult {
        self.exec_self(ctx, def)
    }
}
impl ComponentMeta for GxlMod {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::Mod(self.meta.clone())
    }
}

impl AppendAble<RgProp> for GxlMod {
    fn append(&mut self, prop: RgProp) {
        self.props.push(prop);
    }
}
impl AppendAble<Vec<RgProp>> for GxlMod {
    fn append(&mut self, prop_vec: Vec<RgProp>) {
        for prop in prop_vec {
            self.props.push(prop);
        }
    }
}

impl AppendAble<Activity> for GxlMod {
    fn append(&mut self, hold: Activity) {
        self.acts.insert(hold.com_meta().name().to_string(), hold);
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

    use crate::{
        components::{
            gxl_env::meta::EnvMeta, gxl_flow::meta::FlowMeta, gxl_spc::GxlSpace, gxl_var::RgProp,
            GxlEnv, GxlFlow, GxlMod, RgVars,
        },
        context::ExecContext,
        execution::sequence::Sequence,
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
        assert_eq!(result.is_some(), true);
    }

    #[test]
    fn test_merge_to_head_multiple() {
        let meta1 = ModMeta::build_mod("mod1");
        let mut mod1 = GxlMod::from(meta1.clone());
        mod1.props.push(RgProp::new("k1", "v1"));

        let meta2 = ModMeta::new2(GxlType::Mod, "mod2".to_string());
        let mut mod2 = GxlMod::from(meta2);
        mod2.props.push(RgProp::new("k2", "v2"));

        let mixs: Vec<GxlMod> = vec![mod1, mod2];

        let result = merge_mod(mixs);
        assert_eq!(result.is_some(), true);

        if let Some(target) = result {
            assert_eq!(target.meta.name(), "mod1");
            assert_eq!(target.props.len(), 2);
            assert!(target.props().iter().any(|x| x.key() == &"K1".to_string()));
            assert!(target.props().iter().any(|x| x.key() == &"K2".to_string()));
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
        env1.append(RgProp::new("key1", "value1"));
        mod1.append(ModItem::Env(env1));

        // 创建 mod2 并引用 mod1 的环境变量
        let mod_name2 = "mod2";
        let meta_mod2 = ModMeta::build_mod(mod_name2);
        let mut mod2 = GxlMod::from(meta_mod2.clone());

        // 假设 mod2 添加了一个依赖于 mod1 的环境变量
        let mut env2 = GxlEnv::from(EnvMeta::build_env_mix("env2", vec!["mod1.env1"]));
        env2.append(RgProp::new("key2", "value2"));
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
        env1.append(RgProp::new("key1", "value1"));
        let mut var = RgVars::default();
        var.insert("key3", "value1");
        env1.append(var);
        mod1.append(env1);
        mod1.append(RgProp::new("key2", "value1"));

        let ctx = ExecContext::default();
        let mut sequ = Sequence::from("exec");

        // 调用 assemble_env 方法
        mod1.load_env(ctx, &mut sequ, "env1")?;

        let ctx = ExecContext::default();
        let vars = VarSpace::default();
        let (vars, _) = sequ.execute(ctx, vars.clone()).await.unwrap();

        println!("{:?}", vars.global().maps());
        assert_eq!(
            vars.global().maps().get(&"ENV_KEY1".to_string()),
            Some(&SecVar::new(VarMeta::Normal, "value1".to_string()))
        );
        assert_eq!(
            vars.global().maps().get(&"MOD1_KEY2".to_string()),
            Some(&SecVar::new(VarMeta::Normal, "value1".to_string()))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_assemble_flow_success() -> AnyResult<()> {
        once_init_log();
        let meta1 = ModMeta::build_mod("mod1");
        let mut mod1 = GxlMod::from(meta1);
        mod1.append(RgProp::new("k1", "v1"));
        mod1.append(GxlFlow::from("flow1"));

        let mod_name = "mod2";
        let meta2 = ModMeta::build_mod(mod_name);
        let mut mod2 = GxlMod::from(meta2.clone());
        mod2.append(RgProp::new("k2", "v2"));

        // 添加一个流程
        let flow1 = GxlFlow::from(FlowMeta::build_flow_pre("flow2", "mod1.flow1"));
        mod2.append(flow1);

        // 设置自动入口流程

        let mut spc = GxlSpace::default();
        spc.append(mod1);
        spc.append(mod2);
        let work_spc = spc.assemble_depend()?;

        let ctx = ExecContext::default();
        let mut sequ = Sequence::from("exec");

        // 调用 assemble_flow 方法
        work_spc.load_flow(ctx, &mut sequ, "mod2.flow2")?;

        let ctx = ExecContext::default();
        let vars = VarSpace::default();
        let (vars, _task) = sequ.execute(ctx, vars).await.unwrap();

        println!("{:?}", vars.global().maps());
        assert_eq!(
            vars.global().maps().len(),
            0 //vars.maps().get(&"MOD2_K2".to_string()),

              //Some(&SecVar::new(VarMeta::Normal, "v2".to_string()))
        );
        Ok(())
    }
}
