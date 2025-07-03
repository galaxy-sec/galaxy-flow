use std::collections::HashMap;

use orion_common::friendly::AppendAble;

use crate::error::AResult;
use crate::traits::ExecLoadTrait;

use super::gxl_mod::merge_mod;
use super::gxl_spc::GxlSpace;
use super::GxlMod;

#[derive(Clone, Getters, Default)]
pub struct CodeSpace {
    mods: Vec<String>,
    store: HashMap<String, Vec<GxlMod>>, //mods: BTreeMap<String, RgMod>,
}

impl CodeSpace {
    #[allow(clippy::result_large_err)]
    pub fn assemble(&self) -> AResult<GxlSpace> {
        let mut target_spc = GxlSpace::default();
        for m_name in self.mods.iter() {
            if let Some(m) = self.store.get(m_name) {
                if let Some(target_mod) = merge_mod(m.clone()) {
                    target_spc.append(target_mod);
                }
            }
        }
        target_spc.assemble()
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
    fn append(&mut self, gxl_mod: GxlMod) {
        let key = gxl_mod.of_name();
        let mix = gxl_mod.meta().mix().clone();
        if let Some(_vec) = self.store.get(&key) {
            warn!(target: "stc","重复 mod  {}", key);
        } else {
            let mut mod_vec = vec![gxl_mod];
            for item in &mix {
                if let Some(i_vec) = self.store.get(item) {
                    mod_vec.append(&mut i_vec.clone());
                }
            }
            self.mods.push(key.clone());
            self.store.insert(key, mod_vec);
        }
    }
}

#[cfg(test)]
mod tests {
    use orion_common::friendly::New2;
    use orion_error::TestAssert;

    use crate::components::gxl_mod::meta::ModMeta;
    use crate::components::gxl_var::GxlVar;
    use crate::components::{GxlEnv, GxlFlow, GxlMod, GxlProps};
    use crate::execution::exec_init_env;
    use crate::execution::sequence::Sequence;
    use crate::types::AnyResult;

    use super::*;

    #[tokio::test]
    async fn execute_forword() -> AnyResult<()> {
        let (ctx, def) = exec_init_env();

        let meta = ModMeta::build_mod("main");
        let mut gxl_mod = GxlMod::from(meta);
        gxl_mod.append(GxlVar::new("key1", "val1"));

        let gxl_flow = GxlFlow::load_ins("flow1".to_string());

        let mut gxl_vars = GxlProps::new("forword.props");
        gxl_vars.append(GxlVar::new("key1", "val1"));

        let meta = ModMeta::build_mod("env");
        let mut gxl_mod_env = GxlMod::from(meta);
        gxl_mod.append(GxlVar::new("key1", "val1"));

        let mut gxl_env = GxlEnv::from("env1");
        gxl_env.append(GxlVar::new("key1", "val1"));
        gxl_env.append(gxl_vars);
        gxl_mod_env.append(gxl_env);

        gxl_mod.append(gxl_flow);
        let mut gxl_space = CodeSpace::default();
        gxl_space.append(gxl_mod_env);
        gxl_space.append(gxl_mod);

        let mut flow = Sequence::from("test");
        let work_spc = gxl_space.assemble().assert();
        work_spc.load_env(ctx.clone(), &mut flow, "env.env1")?;
        work_spc.load_flow(ctx.clone(), &mut flow, "main.flow1")?;
        let job = flow.test_execute(ctx, def).await;
        debug!("job {:#?}", job);
        work_spc.show().unwrap();
        Ok(())
    }
}
