use super::{code_spc::CodeSpace, prelude::*};
use crate::{
    ability::prelude::TaskValue, execution::sequence::Sequence, menu::*,
    util::task_report::task_local_report,
};
use colored::Colorize;
use contracts::requires;
use orion_error::ErrorConv;
use std::{collections::HashMap, fmt::Display};

use super::GxlMod;

const MAIN_MOD: &str = "main";
const ENV_MOD: &str = "env";
const ENVS_MOD: &str = "envs";

#[derive(Clone, Default)]
pub struct GxlSpace {
    mods_name: Vec<String>,
    mods_store: HashMap<String, GxlMod>,
    assembled: bool,
}

impl GxlSpace {
    pub fn get(&self, key: &str) -> Option<&GxlMod> {
        self.mods_store.get(key)
    }

    pub fn len(&self) -> usize {
        self.mods_store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.mods_store.is_empty()
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

    pub fn assemble(&mut self) -> AResult<Self> {
        let mut spc = Self::default();

        for mod_name in &self.mods_name {
            if let Some(module) = self.get(mod_name) {
                spc.append(module.clone());
                let updated = module.clone().assemble(mod_name, &spc)?;
                debug_assert!(updated.assembled());
                spc.replace(updated);
            }
        }
        spc.assembled = true;
        Ok(spc)
    }

    fn replace(&mut self, updated: GxlMod) {
        self.mods_store.insert(updated.of_name(), updated);
    }
}

impl AppendAble<GxlMod> for GxlSpace {
    fn append(&mut self, module: GxlMod) {
        let name = module.of_name();
        self.mods_name.push(name.clone());
        self.mods_store.insert(name, module);
    }
}

impl TryFrom<CodeSpace> for GxlSpace {
    type Error = AssembleError;

    fn try_from(value: CodeSpace) -> AResult<Self> {
        value.assemble()
    }
}

impl ExecLoadTrait for GxlSpace {
    #[requires(self.assembled)]
    fn load_env(&self, ctx: ExecContext, sequ: &mut Sequence, obj_path: &str) -> ExecResult<()> {
        let (mod_name, item_name) = parse_obj_path(obj_path)?;

        self.mods_store
            .get(mod_name)
            .ok_or(ExecReason::Miss(mod_name.to_string()))?
            .load_env(ctx, sequ, item_name)
    }

    #[requires(self.assembled)]
    fn load_flow(&self, ctx: ExecContext, sequ: &mut Sequence, obj_path: &str) -> ExecResult<()> {
        let (mod_name, item_name) = parse_obj_path(obj_path)?;

        self.mods_store
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

#[derive(Default, Debug, Clone)]
pub struct ExecOptions {
    dryrun: bool,
    out: bool,
}
impl ExecOptions {
    pub fn with_dryrun(mut self, dryrun: bool) -> Self {
        self.dryrun = dryrun;
        self
    }
    pub fn is_dryrun(&self) -> bool {
        self.dryrun
    }
    pub fn is_out(&self) -> bool {
        self.out
    }
}

impl GxlSpace {
    #[requires(self.assembled)]
    pub async fn exec<VS: Into<Vec<String>>>(
        &self,
        envs_name: VS,
        flows_name: VS,
        out: Option<bool>,
        dryrun: bool,
        var_space: VarSpace,
    ) -> RunResult<()> {
        info!(
            target: "execution",
            "Starting execution stack with output: {:?}", out
        );

        let envs: Vec<String> = envs_name.into();
        let flow_names: Vec<String> = flows_name.into();

        warn!(target : "exec","Executing with envs: {:?}, flows: {:?}", envs, flow_names);
        warn!(target : "exec","inherted vars :\n{}", var_space.inherited());
        info!(target : "exec","inherted vars :\n{}", var_space.global());

        let main_ctx = ExecContext::new(out, dryrun);
        for flow_name in flow_names {
            self.execute_flow(&main_ctx, &var_space, &envs, &flow_name)
                .await?;
        }

        Ok(())
    }

    #[requires(self.assembled)]
    async fn execute_flow(
        &self,
        main_ctx: &ExecContext,
        var_space: &VarSpace,
        envs: &[String],
        flow_name: &str,
    ) -> RunResult<()> {
        let flow_name = self.normalize_flow_name(flow_name);
        println!("execute flow: {}", flow_name);

        let mut exec_sequ = Sequence::from("flow");
        let mut ctx = main_ctx.clone();

        self.load_envs(&mut ctx, envs, &mut exec_sequ)?;

        let flow_ctx = main_ctx.clone();
        self.load_flow(flow_ctx, &mut exec_sequ, &flow_name)
            .err_conv()?;

        let exec_ctx = main_ctx.clone().with_subcontext("exec");

        match exec_sequ
            .execute(exec_ctx, var_space.clone())
            .await
            .err_conv()
        {
            Ok(TaskValue { rec, .. }) => {
                task_local_report(rec);
                Ok(())
            }
            Err(do_err) => {
                //todo report;
                Err(do_err)
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ability::prelude::GxlVar,
        components::{gxl_mod::meta::ModMeta, GxlEnv, GxlFlow, GxlMod, GxlProps},
        execution::exec_init_env,
        types::AnyResult,
    };
    use orion_common::friendly::New2;
    use orion_error::TestAssert;

    #[tokio::test]
    async fn execute_forward() -> AnyResult<()> {
        let (ctx, def) = exec_init_env();

        // Create main module
        let mut main_mod = GxlMod::from(ModMeta::build_mod(MAIN_MOD));
        main_mod.append(GxlVar::new("key1", "val1"));

        let flow = GxlFlow::load_ins("flow1");
        main_mod.append(flow);

        // Create environment module
        let mut env_mod = GxlMod::from(ModMeta::build_mod(ENV_MOD));

        let mut env = GxlEnv::from("env1");
        env.append(GxlVar::new("key1", "val1"));

        let mut rg_vars = GxlProps::default();
        rg_vars.append(GxlVar::new("key1", "val1"));
        env.append(rg_vars);

        env_mod.append(env);

        // Build code space
        let mut code_space = CodeSpace::default();
        code_space.append(env_mod);
        code_space.append(main_mod);

        // Execute
        let mut flow = Sequence::from("test");
        let work_space = code_space.assemble().assert();

        work_space.load_env(ctx.clone(), &mut flow, "env.env1")?;
        work_space.load_flow(ctx.clone(), &mut flow, "main.flow1")?;

        let task_v = flow.test_execute(ctx, def).await.unwrap();
        debug!("Job result: {:#?}", task_v);

        work_space.show().unwrap();
        Ok(())
    }
}
