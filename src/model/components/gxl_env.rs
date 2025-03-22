use orion_error::DomainFrom;

use super::prelude::*;

use crate::ability::prelude::ComponentRunnable;
use crate::ability::GxRead;
use crate::evaluator::Parser;
use crate::parser::stc_base::AnnDto;

use crate::model::annotation::EnvAnnotation;
use std::collections::VecDeque;
use std::sync::Arc;

use super::gxl_spc::GxlSpace;
use super::gxl_utls::take_mod_obj;
use super::gxl_var::RgProp;
use super::RgVars;
use std::io::Write;

#[derive(Clone, Getters, Debug, Default)]
pub struct GxlEnv {
    meta: RgoMeta,
    props: Vec<RgProp>,
    items: Vec<EnvItem>,
}
#[derive(Clone, Debug)]
pub enum EnvItem {
    Var(RgVars),
    Read(GxRead),
    //Vault(GxVault),
}
impl GxlEnv {
    fn get_env(mod_name: &str, mix: &str, src: &GxlSpace) -> AResult<Self> {
        let cur_mix = EnvExpress::from_env().eval(mix).unwrap_or(mix.to_string());
        let (t_mod, env_name) = take_mod_obj(mod_name, cur_mix.as_str());
        if let Some(env) = src.mods().get(&t_mod).and_then(|m| m.envs().get(&env_name)) {
            let link_env = env.clone().assemble(mod_name, src)?;
            return Ok(link_env);
        }
        AssembleError::err_from_domain(AssembleReason::Miss(format!("env {}", cur_mix)))
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
            meta: RgoMeta::build_env(name.to_string()),
            props: Vec::new(),
            items: Vec::new(),
        }
    }
}
impl From<RgoMeta> for GxlEnv {
    fn from(meta: RgoMeta) -> Self {
        Self {
            meta,
            ..Default::default()
        }
    }
}

impl From<String> for GxlEnv {
    fn from(name: String) -> Self {
        Self {
            meta: RgoMeta::build_env(name),
            props: Vec::new(),
            items: Vec::new(),
        }
    }
}

impl From<(String, Vec<String>)> for GxlEnv {
    fn from(value: (String, Vec<String>)) -> Self {
        let mut meta = RgoMeta::build_env(value.0);
        meta.set_mix(value.1);
        Self {
            meta,
            props: Vec::new(),
            items: Vec::new(),
        }
    }
}

impl GxlEnv {
    pub fn set_anns(&mut self, dto: Option<AnnDto>) {
        let ann_vec = if let Some(have) = dto {
            have.convert::<EnvAnnotation>()
        } else {
            Vec::new()
        };
        self.meta.set_anns(ann_vec);
    }
    pub fn meta_mut(&mut self) -> &mut RgoMeta {
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

impl RunnableTrait for GxlEnv {
    fn exec(&self, mut ctx: ExecContext, def: &mut VarsDict) -> EOResult {
        let env_name = self.meta.name();
        ctx.append(env_name);

        debug!(target: ctx.path(),"env {} setting", env_name );
        self.export_props(ctx.clone(), def, "ENV")?;
        for item in &self.items {
            item.exec(ctx.clone(), def)?;
        }
        Ok(ExecOut::Ignore)
    }
}
impl RunnableTrait for EnvItem {
    fn exec(&self, ctx: ExecContext, dict: &mut VarsDict) -> EOResult {
        match self {
            EnvItem::Var(o) => o.exec(ctx, dict),
            EnvItem::Read(o) => o.exec(ctx, dict),
            //EnvItem::Vault(o) => o.exec(ctx, dict),
        }
    }
}
impl ComponentRunnable for GxlEnv {
    fn meta(&self) -> RgoMeta {
        self.meta.clone()
    }
}

impl PropsTrait for GxlEnv {
    fn fetch_props(&self) -> &Vec<RgProp> {
        &self.props
    }
}

pub type GxlEnvHold = Arc<GxlEnv>;
impl AppendAble<RgProp> for GxlEnv {
    fn append(&mut self, prop: RgProp) {
        self.props.push(prop);
    }
}
impl AppendAble<Vec<RgProp>> for GxlEnv {
    fn append(&mut self, mut props: Vec<RgProp>) {
        self.props.append(&mut props)
    }
}

impl AppendAble<RgVars> for GxlEnv {
    fn append(&mut self, vars: RgVars) {
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
        components::{code_spc::CodeSpace, gxl_spc::GxlSpace, gxl_var::RgProp, GxlEnv},
        infra::once_init_log,
        model::components::GxlMod,
        traits::{DependTrait, PropsTrait},
        types::AnyResult,
    };

    #[test]
    fn test_assemble_com() -> AnyResult<()> {
        // Create a base RgEnv instance
        let mut base_env = GxlEnv::from("base_env");
        base_env.append(RgProp::new("base_prop1", "p1"));
        base_env.append(RgProp::new("base_prop2", "p2"));

        // Create a source RgMod with an RgEnv to be merged
        let mut src_mod = GxlMod::from("src_mod");
        let mut src_env = GxlEnv::from("src_env");
        src_env.append(RgProp::new("src_prop1", "s1"));
        src_env.append(RgProp::new("src_prop2", "s2"));
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
        base_env.append(RgProp::new("base_prop1", "p1"));

        // Create a source RgMod with multiple RgEnv instances to be merged
        let mut src_mod = GxlMod::from("src_mod");

        // Add first source environment
        let mut src_env1 = GxlEnv::from("src_env1");
        src_env1.append(RgProp::new("src_prop1", "s1"));
        src_mod.append(src_env1);

        // Add second source environment
        let mut src_env2 = GxlEnv::from("src_env2");
        src_env2.append(RgProp::new("src_prop2", "s2"));
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
        base_env.append(RgProp::new("base_prop1", "p1"));

        // Create a source RgMod with an RgEnv, but do not add it to the mix
        let mut src_mod = GxlMod::from("src_mod");
        let mut src_env = GxlEnv::from("src_env");
        src_env.append(RgProp::new("src_prop1", "s1"));
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
