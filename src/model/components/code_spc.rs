use std::collections::HashMap;

use orion_common::friendly::AppendAble;

use crate::error::AResult;
use crate::traits::ExecLoadTrait;

use super::gxl_mod::merge_to_head;
use super::gxl_spc::GxlSpace;
use super::GxlMod;

#[derive(Clone, Getters, Default)]
pub struct CodeSpace {
    mods: HashMap<String, Vec<GxlMod>>, //mods: BTreeMap<String, RgMod>,
}

impl CodeSpace {
    #[allow(clippy::result_large_err)]
    pub fn assemble_mix(&self) -> AResult<GxlSpace> {
        let mut target_spc = GxlSpace::default();
        for (_, m) in self.mods.iter() {
            if let Some(target_mod) = merge_to_head(m.clone()) {
                target_spc.append(target_mod);
            }
        }
        target_spc.assemble_depend()?;
        Ok(target_spc)
    }
}

impl AppendAble<Vec<GxlMod>> for CodeSpace {
    fn append(&mut self, mod_vec: Vec<GxlMod>) {
        for item in mod_vec {
            self.append(item)
        }
    }
}
impl AppendAble<GxlMod> for CodeSpace {
    fn append(&mut self, rg_mod: GxlMod) {
        let key = rg_mod.of_name();
        let mix = rg_mod.meta().mix().clone();
        if let Some(_vec) = self.mods.get(&key) {
            warn!(target: "stc","重复 mod  {}", key);
        } else {
            let mut mod_vec = vec![rg_mod];
            for item in &mix {
                if let Some(i_vec) = self.mods.get(item) {
                    mod_vec.append(&mut i_vec.clone());
                }
            }
            self.mods.insert(key, mod_vec);
        }
    }
}

#[cfg(test)]
mod tests {
    use orion_common::friendly::New2;
    use orion_error::TestAssert;

    use crate::components::gxl_var::RgProp;
    use crate::components::{GxlEnv, GxlFlow, GxlMod, RgVars};
    use crate::execution::exec_init_env;
    use crate::execution::sequence::Sequence;
    use crate::meta::RgoMeta;
    use crate::types::AnyResult;

    use super::*;

    #[tokio::test]
    async fn execute_forword() -> AnyResult<()> {
        let (ctx, def) = exec_init_env();

        let meta = RgoMeta::build_mod("main");
        let mut rg_mod = GxlMod::from(meta);
        rg_mod.append(RgProp::new("key1", "val1"));

        let rg_flow = GxlFlow::load_ins("flow1".to_string());

        let mut rg_vars = RgVars::default();
        rg_vars.append(RgProp::new("key1", "val1"));

        let meta = RgoMeta::build_mod("env");
        let mut rg_mod_env = GxlMod::from(meta);
        rg_mod.append(RgProp::new("key1", "val1"));

        let mut rg_env = GxlEnv::from("env1");
        rg_env.append(RgProp::new("key1", "val1"));
        rg_env.append(rg_vars);
        rg_mod_env.append(rg_env);

        rg_mod.append(rg_flow);
        let mut rg_space = CodeSpace::default();
        rg_space.append(rg_mod_env);
        rg_space.append(rg_mod);

        let mut flow = Sequence::from("test");
        let work_spc = rg_space.assemble_mix().assert();
        work_spc.load_env(ctx.clone(), &mut flow, "env.env1")?;
        work_spc.load_flow(ctx.clone(), &mut flow, "main.flow1")?;
        let job = flow.test_execute(ctx, def).await;
        debug!("job {:#?}", job);
        work_spc.show().unwrap();
        Ok(())
    }
}
