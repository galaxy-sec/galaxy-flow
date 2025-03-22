use super::prelude::*;
use crate::execution::sequence::Sequence;
use crate::menu::*;
use crate::traits::Setter;
use crate::util::traits::*;
use crate::var::VarsDict;
use colored::*;
use orion_error::ErrorConv;
use std::collections::HashMap;

use super::code_spc::CodeSpace;
use super::GxlMod;

#[derive(Clone, Getters, Default)]
pub struct GxlSpace {
    mods: HashMap<String, GxlMod>,
}

impl GxlSpace {
    pub fn get(&self, key: &str) -> Option<&GxlMod> {
        self.mods.get(key)
    }
    pub fn main(&self) -> ExecResult<GxlMod> {
        self.get("main")
            .cloned()
            .ok_or(ExecReason::Args("not get main mod".to_string()).into())
    }
    pub fn env(&self) -> ExecResult<GxlMod> {
        //兼容("mod env");
        if let Some(found) = self.get("env") {
            return Ok(found.clone());
        }
        if let Some(found) = self.get("envs") {
            return Ok(found.clone());
        }
        self.get("main")
            .cloned()
            .ok_or(ExecReason::Args("not get main mod".to_string()).into())
    }
    pub fn show(&self) -> ExecResult<()> {
        let menu = self.menu()?;
        println!("---------------prj work menu-------------");
        println!("envs: ");
        for chose in menu.envs() {
            show_item(chose);
        }
        println!("\n flow: ");
        for chose in menu.flows() {
            show_item(chose);
        }
        Ok(())
    }

    pub(crate) fn assemble_depend(&mut self) -> AResult<Self> {
        let mut spc = Self::default();
        for (k, m) in self.mods.iter() {
            let x = m.clone().assemble(k.as_str(), self)?;
            spc.append(x);
        }
        Ok(spc)
    }
}

impl AppendAble<GxlMod> for GxlSpace {
    fn append(&mut self, now: GxlMod) {
        self.mods.insert(now.of_name(), now);
    }
}
fn get_os_sys() -> String {
    let info = os_info::get();
    let os_string = match info.os_type() {
        os_info::Type::Macos => "macos".to_string(),
        _ => info.os_type().to_string().to_lowercase(),
    };

    let arch = info.architecture().unwrap_or("unknown");
    let ver_major = match info.version() {
        os_info::Version::Semantic(major, _, _) => major,
        _ => &0,
    };
    let os_sys = format!("{}_{}_{}", arch, os_string, ver_major);
    os_sys
}

impl TryFrom<CodeSpace> for GxlSpace {
    type Error = AssembleError;
    fn try_from(value: CodeSpace) -> AResult<Self> {
        value.assemble_mix()?.assemble_depend()
    }
}

impl ExecLoadTrait for GxlSpace {
    fn load_env(&self, ctx: ExecContext, sequ: &mut Sequence, obj_path: &str) -> ExecResult<()> {
        //info!(target:ctx.path(),"load env:{}", obj_path);
        let v: Vec<&str> = obj_path.split('.').collect();
        if v.len() >= 2 {
            let mod_name = v[0];
            let item_name = v[1];
            if let Some(target_mod) = self.mods.get(mod_name) {
                return target_mod.load_env(ctx, sequ, item_name);
            }
        }
        ExecError::err_from_domain(ExecReason::Miss(obj_path.into()))
    }
    fn load_flow(&self, ctx: ExecContext, sequ: &mut Sequence, obj_path: &str) -> ExecResult<()> {
        let v: Vec<&str> = obj_path.split('.').collect();
        if v.len() >= 2 {
            let mod_name = v[0];
            let item_name = v[1];
            if let Some(target_mod) = self.mods.get(mod_name) {
                return target_mod.load_flow(ctx, sequ, item_name);
            }
        }
        ExecError::err_from_domain(ExecReason::Miss(obj_path.into()))
    }
    fn of_name(&self) -> String {
        "space".to_string()
    }
    fn menu(&self) -> ExecResult<GxMenu> {
        let mut m1 = self.main()?.menu()?;
        let mut m2 = self.env()?.menu()?;
        m1.envs.truncate(0);
        m2.flows.truncate(0);
        m1.merge(&mut m2);
        Ok(m1)
    }
}

impl GxlSpace {
    pub fn exec<VS: LocalInto<Vec<String>>>(
        &self,
        envs: VS,
        flow_names: VS,
        out: bool,
    ) -> RunResult<()> {
        info!(target: "-----------exec stack -------------", "--------------out info--------------");
        let main_ctx = ExecContext::new(out);
        let mut def = VarsDict::default();
        let l_envs: Vec<String> = envs.into();
        let l_flws: Vec<String> = flow_names.into();
        info!(target:main_ctx.path(),"galaxy flow execute envs: {:?},flow:  {:?}", l_envs,l_flws);
        let mut ctx = main_ctx.clone();

        ctx.append("load");
        for f_name in l_flws {
            let f_name = if !f_name.contains('.') {
                format!("main.{}", f_name)
            } else {
                f_name.to_string()
            };
            //let mut ctx = ctx.clone();
            //debug!(target:ctx.path(),"----load flow[{}] sequ begin ----", f_name);
            let mut exec_sequ = Sequence::from("flow");
            def.set("__ENVS", "UNDEF");

            let os_sys = get_os_sys();
            def.set("RG_OS_SYS", os_sys.as_str());

            if let Some(value) = self.load_envs(ctx.clone(), &l_envs, &mut exec_sequ) {
                return value;
            }
            let mut cur_ctx = ctx.clone();
            cur_ctx.append("flow");
            //ctx.append(&f_name);
            self.load_flow(cur_ctx, &mut exec_sequ, f_name.as_str())
                .err_conv()?;
            //.map_err(stc_err_conv)?;
            let mut exec_ctx = ExecContext::new(out);
            exec_ctx.append("exec");
            let _ = exec_sequ.execute(exec_ctx, &mut def).err_conv()?;
            //.map_err(stc_err_conv)?;
        }
        Ok(())
    }

    fn load_envs(
        &self,
        mut ctx: ExecContext,
        envs: &Vec<String>,
        exec_sequ: &mut Sequence,
    ) -> Option<RunResult<()>> {
        ctx.append("env");
        for env in envs {
            //info!(target:ctx.path(),"load env:{}", env);
            if self.load_env(ctx.clone(), exec_sequ, env).is_ok()
                || self
                    .load_env(ctx.clone(), exec_sequ, format!("main.{}", env).as_str())
                    .is_ok()
                || self
                    .load_env(ctx.clone(), exec_sequ, format!("envs.{}", env).as_str())
                    .is_ok()
                || self
                    .load_env(ctx.clone(), exec_sequ, format!("env.{}", env).as_str())
                    .is_ok()
            {
                continue;
            }
            return Some(RunError::err_from_domain(RunReason::Args(format!(
                "not fount env {}",
                env
            ))));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use orion_common::friendly::New2;
    use orion_error::TestAssert;

    use crate::components::code_spc::CodeSpace;
    use crate::components::gxl_var::RgProp;
    use crate::components::{GxlEnv, GxlFlow, GxlMod, RgVars};
    use crate::execution::exec_init_env;
    use crate::meta::RgoMeta;
    use crate::types::AnyResult;

    use super::*;

    #[test]
    fn execute_forword() -> AnyResult<()> {
        let (ctx, mut def) = exec_init_env();

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
        let job = flow.test_execute(ctx, &mut def);
        debug!("job {:#?}", job);
        work_spc.show().unwrap();
        Ok(())
    }
}
pub fn show_item(chose: &MenuItem) {
    if !chose.key.starts_with('_') {
        if let Some(str_desp) = &chose.desp {
            color_show(
                format!("    * {:<20} -- {}", chose.key, str_desp),
                chose.color.clone(),
            );
        } else {
            color_show(format!("    * {:<20}", chose.key), chose.color.clone());
        }
    }
}
pub fn color_show(item: String, color: Option<String>) {
    if let Some(c) = color {
        match c.as_str() {
            "red" => println!("{}", item.red()),
            "green" => println!("{}", item.green()),
            "blue" => println!("{}", item.blue()),
            "yellow" => println!("{}", item.yellow()),
            "cyan" => println!("{}", item.cyan()),
            "magenta" => println!("{}", item.magenta()),
            "black" => println!("{}", item.black()),
            "white" => println!("{}", item.white()),
            "purple" => println!("{}", item.purple()),
            _ => println!("{}", item),
        }
        return;
    }
    println!("{}", item)
}
