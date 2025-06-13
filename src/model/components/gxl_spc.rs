use super::{code_spc::CodeSpace, prelude::*};
use crate::{
    execution::sequence::Sequence, menu::*, traits::Setter, util::task_report::task_local_report,
};
use colored::Colorize;
use orion_error::{ErrorConv, ErrorOwe, ErrorWith};
use std::{collections::HashMap, env::current_dir, fmt::Display};

use super::GxlMod;

const MAIN_MOD: &str = "main";
const ENV_MOD: &str = "env";
const ENVS_MOD: &str = "envs";

#[derive(Clone, Default)]
pub struct GxlSpace {
    mods: Vec<String>,
    store: HashMap<String, GxlMod>,
}

impl GxlSpace {
    pub fn get(&self, key: &str) -> Option<&GxlMod> {
        self.store.get(key)
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    pub fn main(&self) -> ExecResult<&GxlMod> {
        self.get(MAIN_MOD)
            .ok_or_else(|| ExecReason::Args(format!("'{}' mod not found", MAIN_MOD)).into())
    }

    pub fn env(&self) -> ExecResult<&GxlMod> {
        self.get(ENV_MOD)
            .or_else(|| self.get(ENVS_MOD))
            .or_else(|| self.get(MAIN_MOD))
            .ok_or_else(|| {
                ExecReason::Args("Neither 'envs' 'env' nor 'main' mod found".to_string()).into()
            })
    }

    pub fn show(&self) -> ExecResult<()> {
        let menu = self.menu()?;
        println!(
            "{}",
            "---------------prj work menu-------------".cyan().bold()
        );

        println!("{}", "envs:".yellow());
        for choice in menu.envs() {
            show_item(choice);
        }

        println!("\n{}", "flow:".yellow());
        for choice in menu.flows() {
            show_item(choice);
        }

        Ok(())
    }

    pub(crate) fn assemble_depend(&mut self) -> AResult<Self> {
        let mut spc = Self::default();

        for mod_name in &self.mods {
            if let Some(module) = self.get(mod_name) {
                let updated = module.clone().assemble(mod_name, self)?;
                spc.append(updated);
            }
        }

        Ok(spc)
    }
}

impl AppendAble<GxlMod> for GxlSpace {
    fn append(&mut self, module: GxlMod) {
        let name = module.of_name();
        self.mods.push(name.clone());
        self.store.insert(name, module);
    }
}

fn get_os_info() -> (String, String, u64) {
    let info = os_info::get();
    let os_type = match info.os_type() {
        os_info::Type::Macos => "macos".to_string(),
        _ => info.os_type().to_string().to_lowercase(),
    };

    let arch = info.architecture().unwrap_or("unknown").to_string();
    let ver_major = match info.version() {
        os_info::Version::Semantic(major, _, _) => *major,
        _ => 0,
    };

    (arch, os_type, ver_major)
}

fn format_os_sys() -> String {
    let (arch, os_type, ver_major) = get_os_info();
    format!("{}_{}_{}", arch, os_type, ver_major)
}

impl TryFrom<CodeSpace> for GxlSpace {
    type Error = AssembleError;

    fn try_from(value: CodeSpace) -> AResult<Self> {
        value.assemble_mix()?.assemble_depend()
    }
}

impl ExecLoadTrait for GxlSpace {
    fn load_env(&self, ctx: ExecContext, sequ: &mut Sequence, obj_path: &str) -> ExecResult<()> {
        let (mod_name, item_name) = parse_obj_path(obj_path)?;

        self.store
            .get(mod_name)
            .ok_or(ExecReason::Miss(mod_name.to_string()))?
            .load_env(ctx, sequ, item_name)
    }

    fn load_flow(&self, ctx: ExecContext, sequ: &mut Sequence, obj_path: &str) -> ExecResult<()> {
        let (mod_name, item_name) = parse_obj_path(obj_path)?;

        self.store
            .get(mod_name)
            .ok_or(ExecReason::Miss(mod_name.to_string()))?
            .load_flow(ctx, sequ, item_name)
    }

    fn of_name(&self) -> String {
        "space".to_string()
    }

    fn menu(&self) -> ExecResult<GxMenu> {
        let mut main_menu = self.main()?.menu()?;
        let mut env_menu = self.env()?.menu()?;

        // Merge menus
        env_menu.flows.clear();
        main_menu.envs.clear();
        main_menu.envs.extend(env_menu.envs);

        Ok(main_menu)
    }
}

// Helper function to parse object paths
fn parse_obj_path(obj_path: &str) -> ExecResult<(&str, &str)> {
    let mut parts = obj_path.splitn(2, '.');

    match (parts.next(), parts.next()) {
        (Some(mod_name), Some(item_name)) => Ok((mod_name, item_name)),
        _ => Err(ExecReason::InvalidPath(obj_path.to_string()).into()),
    }
}

impl GxlSpace {
    pub async fn exec<VS: Into<Vec<String>>>(
        &self,
        envs: VS,
        flow_names: VS,
        out: bool,
    ) -> RunResult<()> {
        info!(
            target: "execution",
            "Starting execution stack with output: {}", out
        );

        let envs: Vec<String> = envs.into();
        let flow_names: Vec<String> = flow_names.into();

        info!("Executing with envs: {:?}, flows: {:?}", envs, flow_names);

        let main_ctx = ExecContext::new(out);
        let mut var_space = VarSpace::default();
        var_space.load_secfile()?;

        // Set global variables
        self.setup_global_vars(&mut var_space)?;

        for flow_name in flow_names {
            self.execute_flow(&main_ctx, &mut var_space, &envs, &flow_name)
                .await?;
        }

        Ok(())
    }

    fn setup_global_vars(&self, var_space: &mut VarSpace) -> RunResult<()> {
        var_space.global_mut().set("__ENVS", "UNDEF");
        var_space
            .global_mut()
            .set("GXL_OS_SYS", format_os_sys().as_str());

        let start_root = current_dir().owe_sys().want("get current dir")?;
        var_space
            .global_mut()
            .set("GXL_START_ROOT", start_root.display().to_string());

        Ok(())
    }

    async fn execute_flow(
        &self,
        main_ctx: &ExecContext,
        var_space: &VarSpace,
        envs: &[String],
        flow_name: &str,
    ) -> RunResult<()> {
        let flow_name = self.normalize_flow_name(flow_name);
        println!("Initializing flow: {}", flow_name);

        let mut exec_sequ = Sequence::from("flow");
        let mut ctx = main_ctx.clone();

        self.load_envs(&mut ctx, envs, &mut exec_sequ)?;

        let flow_ctx = main_ctx.clone();
        self.load_flow(flow_ctx, &mut exec_sequ, &flow_name)
            .err_conv()?;

        let exec_ctx = main_ctx.clone().with_subcontext("exec");
        let (_, output) = exec_sequ
            .execute(exec_ctx, var_space.clone())
            .await
            .err_conv()?;

        task_local_report(output);
        Ok(())
    }

    fn normalize_flow_name(&self, name: &str) -> String {
        if name.contains('.') {
            name.to_string()
        } else {
            format!("{}.{}", MAIN_MOD, name)
        }
    }

    fn load_envs(
        &self,
        ctx: &mut ExecContext,
        envs: &[String],
        exec_sequ: &mut Sequence,
    ) -> RunResult<()> {
        ctx.append("env");

        for env in envs {
            let env_paths = [
                env.as_str(),
                &format!("{}.{}", MAIN_MOD, env),
                &format!("{}.{}", ENVS_MOD, env),
                &format!("{}.{}", ENV_MOD, env),
            ];

            if let Some(found) = env_paths
                .iter()
                .find(|path| self.load_env(ctx.clone(), exec_sequ, path).is_ok())
            {
                info!("Loaded environment: {}", found);
                continue;
            }

            return Err(RunReason::Args(format!("Environment '{}' not found", env)).into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ability::prelude::RgProp,
        components::{GxlEnv, GxlFlow, GxlMod, RgVars},
        execution::exec_init_env,
        meta::GxlMeta,
        types::AnyResult,
    };
    use orion_common::friendly::New2;
    use orion_error::TestAssert;

    #[tokio::test]
    async fn execute_forward() -> AnyResult<()> {
        let (ctx, def) = exec_init_env();

        // Create main module
        let mut main_mod = GxlMod::from(GxlMeta::build_mod(MAIN_MOD));
        main_mod.append(RgProp::new("key1", "val1"));

        let flow = GxlFlow::load_ins("flow1");
        main_mod.append(flow);

        // Create environment module
        let mut env_mod = GxlMod::from(GxlMeta::build_mod(ENV_MOD));

        let mut env = GxlEnv::from("env1");
        env.append(RgProp::new("key1", "val1"));

        let mut rg_vars = RgVars::default();
        rg_vars.append(RgProp::new("key1", "val1"));
        env.append(rg_vars);

        env_mod.append(env);

        // Build code space
        let mut code_space = CodeSpace::default();
        code_space.append(env_mod);
        code_space.append(main_mod);

        // Execute
        let mut flow = Sequence::from("test");
        let work_space = code_space.assemble_mix().assert();

        work_space.load_env(ctx.clone(), &mut flow, "env.env1")?;
        work_space.load_flow(ctx.clone(), &mut flow, "main.flow1")?;

        let (_, job) = flow.test_execute(ctx, def).await.unwrap();
        debug!("Job result: {:#?}", job);

        work_space.show().unwrap();
        Ok(())
    }
}

// UI helper functions
pub fn show_item(item: &MenuItem) {
    if item.key.starts_with('_') {
        return;
    }

    let display_text = match &item.desp {
        Some(desp) => format!("    * {:<20} -- {}", item.key, desp),
        None => format!("    * {:<20}", item.key),
    };

    color_show(display_text, item.color.as_deref());
}

pub fn color_show<S: AsRef<str> + Display>(text: S, color: Option<&str>) {
    let colored_text = match color {
        Some("red") => text.as_ref().red(),
        Some("green") => text.as_ref().green(),
        Some("blue") => text.as_ref().blue(),
        Some("yellow") => text.as_ref().yellow(),
        Some("cyan") => text.as_ref().cyan(),
        Some("magenta") => text.as_ref().magenta(),
        Some("black") => text.as_ref().black(),
        Some("white") => text.as_ref().white(),
        Some("purple") => text.as_ref().purple(),
        _ => return println!("{}", text),
    };

    println!("{}", colored_text);
}
