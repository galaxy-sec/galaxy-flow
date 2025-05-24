use std::path::PathBuf;

use orion_error::ErrorConv;
use orion_syspec::{
    artifact::Artifact,
    error::ToErr,
    types::{AsyncUpdateable, TomlAble},
};

use crate::ability::prelude::*;

#[derive(Clone, Default, Debug, PartialEq, Builder)]
#[builder(setter(into))]
pub struct GxDownLoad {
    task_file: Option<String>,
    dst_path: String,
    #[builder(default)]
    dst_name: Option<String>,
}
impl GxDownLoad {
    pub fn with_file<S: Into<String>>(mut self, file: S) -> Self {
        self.task_file = Some(file.into());
        self
    }
}

// impl DefaultDTO for RgEcho {}

#[async_trait]
impl AsyncRunnableTrait for GxDownLoad {
    async fn async_exec(&self, ctx: ExecContext, def: VarSpace) -> VTResult {
        if let Some(file) = &self.task_file {
            let ex = EnvExpress::from_env_mix(def.globle().clone());
            let task_file = ex.eval(file.as_str())?;
            info!(target: ctx.path(), "task_file :{} ", task_file);
            let artifact = if task_file.ends_with("toml") {
                Artifact::from_toml(&PathBuf::from(task_file)).err_conv()?
            } else {
                return ExecReason::Bug("only toml format support".into()).err_result();
            };

            if let Some(dst_name) = &self.dst_name {
                artifact
                    .addr()
                    .update_rename(&PathBuf::from(self.dst_path.as_str()), dst_name)
                    .await
                    .err_conv()?;
            } else {
                artifact
                    .addr()
                    .update_local(&PathBuf::from(self.dst_path.as_str()))
                    .await
                    .err_conv()?;
            }
            Ok((def, ExecOut::Ignore))
        } else {
            return ExecReason::Miss("task_file".into()).err_result();
        }
    }
}

impl ComponentMeta for GxDownLoad {
    fn com_meta(&self) -> RgoMeta {
        RgoMeta::build_ability("gx.echo")
    }
}

#[cfg(test)]
mod tests {}
