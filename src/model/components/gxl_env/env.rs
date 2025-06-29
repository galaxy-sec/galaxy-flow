use crate::ability::prelude::{GxlProp, TaskValue};
use crate::components::gxl_spc::GxlSpace;
use crate::components::gxl_utls::mod_obj_name;
use crate::components::GxlVars;
use crate::model::components::prelude::*;

use crate::ability::GxRead;
use crate::evaluator::Parser;
use crate::execution::runnable::ComponentMeta;
use crate::parser::stc_base::{AnnDto, FunDto};

use std::collections::VecDeque;
use std::sync::Arc;

use std::io::Write;

use super::meta::EnvMeta;

#[derive(Clone, Getters, Debug, Default)]
pub struct GxlEnv {
    meta: EnvMeta,
    props: Vec<GxlProp>,
    items: Vec<EnvItem>,
}
#[derive(Clone, Debug)]
pub enum EnvItem {
    Var(GxlVars),
    Read(GxRead),
    //Vault(GxVault),
}
impl GxlEnv {
    fn get_env(mod_name: &str, mix: &str, src: &GxlSpace) -> AResult<Self> {
        let cur_mix = EnvExpress::from_env().eval(mix).unwrap_or(mix.to_string());
        let (t_mod, env_name) = mod_obj_name(mod_name, cur_mix.as_str());
        if let Some(env) = src.get(&t_mod).and_then(|m| m.envs().get(&env_name)) {
            let link_env = env.clone().assemble(mod_name, src)?;
            return Ok(link_env);
        }
        Err(AssembleError::from(AssembleReason::Miss(format!(
            "mod : {} env {} : by {} , {} ",
            mod_name, cur_mix, t_mod, env_name
        ))))
    }
    fn assemble_impl(&self, mod_name: &str, src: &GxlSpace) -> AResult<Self> {
        let mut buffer = Vec::new();
        let mut mix_list = VecDeque::from(self.meta.mix().clone());

        let mut linked = false;
        let target = if let Some(top) = mix_list.pop_front() {
            let mut base = Self::get_env(mod_name, top.as_str(), src)?;
            for mix in mix_list {
                let link_env = Self::get_env(mod_name, mix.as_str(), src)?;
                base.merge(&link_env);
                let _ = write!(&mut buffer, "{} | ", mix);
                linked = true;
            }
            base.merge(self);
            base
        } else {
            self.clone()
        };
        let _ = write!(&mut buffer, "{} ", self.meta().name());
        if linked {
            info!(
                target: "assemble",
                "assemble env {:>8}.{:<8} : {} ",
                mod_name,
                self.meta().name(),
                String::from_utf8(buffer).unwrap()
            );
        }
        Ok(target)
    }
}

impl DependTrait<&GxlSpace> for GxlEnv {
    fn assemble(self, mod_name: &str, src: &GxlSpace) -> AResult<Self> {
        self.assemble_impl(mod_name, src)
    }
}

impl From<&str> for GxlEnv {
    fn from(name: &str) -> Self {
        Self {
            meta: EnvMeta::build_env(name.to_string()),
            props: Vec::new(),
            items: Vec::new(),
        }
    }
}
impl From<EnvMeta> for GxlEnv {
    fn from(meta: EnvMeta) -> Self {
        Self {
            meta,
            ..Default::default()
        }
    }
}

impl From<String> for GxlEnv {
    fn from(name: String) -> Self {
        Self {
            meta: EnvMeta::build_env(name),
            props: Vec::new(),
            items: Vec::new(),
        }
    }
}

impl From<(String, Vec<String>)> for GxlEnv {
    fn from(value: (String, Vec<String>)) -> Self {
        let mut meta = EnvMeta::build_env(value.0);
        meta.set_mix(value.1);
        Self {
            meta,
            props: Vec::new(),
            items: Vec::new(),
        }
    }
}

pub fn anns_from_option_dto<T: From<FunDto>>(value: Option<AnnDto>) -> Vec<T> {
    value.map_or_else(Vec::new, |have| {
        have.funs.into_iter().map(T::from).collect()
    })
}
impl GxlEnv {
    pub fn set_anns(&mut self, dto: Option<AnnDto>) {
        self.meta.set_annotates(anns_from_option_dto(dto));
    }
    pub fn meta_mut(&mut self) -> &mut EnvMeta {
        &mut self.meta
    }
    pub fn merge(&mut self, other: &Self) {
        self.props.extend(other.props.clone());
        self.items.extend(other.items.clone());
    }
    pub fn pre_merge(&mut self, other: &Self) {
        let self_props_old = self.props.clone();
        self.props.clear();
        self.props.extend(other.props.clone());
        self.props.extend(self_props_old);

        let self_items_old = self.items.clone();
        self.items.clear();
        self.items.extend(other.items.clone());
        self.items.extend(self_items_old);
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxlEnv {
    async fn async_exec(&self, mut ctx: ExecContext, mut def: VarSpace) -> VTResult {
        let env_name = self.meta.name();
        ctx.append(env_name);

        debug!(target: ctx.path(),"env {} setting", env_name );
        self.export_props(ctx.clone(), def.global_mut(), "ENV")?;
        for item in &self.items {
            let TaskValue { vars, .. } = item.async_exec(ctx.clone(), def).await?;
            def = vars;
        }
        Ok(TaskValue::from((def, ExecOut::Ignore)))
    }
}
#[async_trait]
impl AsyncRunnableTrait for EnvItem {
    async fn async_exec(&self, ctx: ExecContext, dict: VarSpace) -> VTResult {
        match self {
            EnvItem::Var(o) => o.async_exec(ctx, dict).await,
            EnvItem::Read(o) => o.async_exec(ctx, dict).await,
            //EnvItem::Vault(o) => o.exec(ctx, dict),
        }
    }
}
impl ComponentMeta for GxlEnv {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::Env(self.meta.clone())
    }
}

impl PropsTrait for GxlEnv {
    fn fetch_props(&self) -> &Vec<GxlProp> {
        &self.props
    }
}

pub type GxlEnvHold = Arc<GxlEnv>;
impl AppendAble<GxlProp> for GxlEnv {
    fn append(&mut self, prop: GxlProp) {
        self.props.push(prop);
    }
}
impl AppendAble<Vec<GxlProp>> for GxlEnv {
    fn append(&mut self, mut props: Vec<GxlProp>) {
        self.props.append(&mut props)
    }
}

impl AppendAble<GxlVars> for GxlEnv {
    fn append(&mut self, vars: GxlVars) {
        self.items.push(EnvItem::Var(vars))
    }
}
impl AppendAble<EnvItem> for GxlEnv {
    fn append(&mut self, item: EnvItem) {
        self.items.push(item)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use orion_common::friendly::New2;
    use orion_error::TestAssert;

    use crate::{
        components::{code_spc::CodeSpace, gxl_spc::GxlSpace, gxl_var::GxlProp, GxlEnv},
        infra::once_init_log,
        model::components::GxlMod,
        traits::{DependTrait, PropsTrait},
        types::AnyResult,
    };

    #[test]
    fn test_assemble_com() -> AnyResult<()> {
        // Create a base RgEnv instance
        let mut base_env = GxlEnv::from("base_env");
        base_env.append(GxlProp::new("base_prop1", "p1"));
        base_env.append(GxlProp::new("base_prop2", "p2"));

        // Create a source RgMod with an RgEnv to be merged
        let mut src_mod = GxlMod::from("src_mod");
        let mut src_env = GxlEnv::from("src_env");
        src_env.append(GxlProp::new("src_prop1", "s1"));
        src_env.append(GxlProp::new("src_prop2", "s2"));
        src_mod.append(src_env);
        let mut raw_spc = CodeSpace::default();
        raw_spc.append(src_mod);
        let work_spc = raw_spc.assemble_mix().assert();

        // Add the source environment to the base environment's mix
        base_env.meta_mut().set_mix(vec!["src_env".to_string()]);

        // Assemble the base environment with the source module
        let assembled_env = base_env.assemble("src_mod", &work_spc)?;

        // Verify that the assembled environment contains both base and source properties
        let props = assembled_env.fetch_props();
        assert_eq!(props.len(), 4);
        assert!(props.iter().any(|p| p.key() == "BASE_PROP1"));
        assert!(props.iter().any(|p| p.key() == "BASE_PROP2"));
        assert!(props.iter().any(|p| p.key() == "SRC_PROP1"));
        assert!(props.iter().any(|p| p.key() == "SRC_PROP2"));
        Ok(())
    }

    #[test]
    fn test_assemble_com_with_multiple_mix() -> AnyResult<()> {
        once_init_log();
        // Create a base RgEnv instance
        let mut base_env = GxlEnv::from("base_env");
        base_env.append(GxlProp::new("base_prop1", "p1"));

        // Create a source RgMod with multiple RgEnv instances to be merged
        let mut src_mod = GxlMod::from("src_mod");

        // Add first source environment
        let mut src_env1 = GxlEnv::from("src_env1");
        src_env1.append(GxlProp::new("src_prop1", "s1"));
        src_mod.append(src_env1);

        // Add second source environment
        let mut src_env2 = GxlEnv::from("src_env2");
        src_env2.append(GxlProp::new("src_prop2", "s2"));
        src_mod.append(src_env2);

        // Add both source environments to the base environment's mix
        base_env
            .meta_mut()
            .set_mix(vec!["src_env1".to_string(), "src_env2".to_string()]);

        let mut spc = CodeSpace::default();
        spc.append(src_mod);
        let w_spc = spc.assemble_mix().assert();
        // Assemble the base environment with the source module
        let assembled_env = base_env.assemble("src_mod", &w_spc)?;

        // Verify that the assembled environment contains all properties
        let props = assembled_env.fetch_props();
        assert_eq!(props.len(), 3);
        assert!(props.iter().any(|p| p.key() == "BASE_PROP1"));
        assert!(props.iter().any(|p| p.key() == "SRC_PROP1"));
        assert!(props.iter().any(|p| p.key() == "SRC_PROP2"));
        Ok(())
    }

    #[test]
    fn test_assemble_com_with_no_mix() -> AnyResult<()> {
        // Create a base RgEnv instance
        let mut base_env = GxlEnv::from("base_env");
        base_env.append(GxlProp::new("base_prop1", "p1"));

        // Create a source RgMod with an RgEnv, but do not add it to the mix
        let mut src_mod = GxlMod::from("src_mod");
        let mut src_env = GxlEnv::from("src_env");
        src_env.append(GxlProp::new("src_prop1", "s1"));
        src_mod.append(src_env);

        let mut spc = GxlSpace::default();
        spc.append(src_mod);
        // Assemble the base environment with the source module
        let assembled_env = base_env.assemble("src_mod", &spc)?;

        // Verify that the assembled environment only contains the base property
        let props = assembled_env.fetch_props();
        assert_eq!(props.len(), 1);
        //println!("{:?}", props);
        assert!(props.iter().any(|p| p.key() == "BASE_PROP1"));
        Ok(())
    }
}
