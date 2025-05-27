use std::path::PathBuf;

use orion_error::ErrorConv;
use orion_syspec::{
    artifact::ArtifactPackage,
    error::ToErr,
    types::{AsyncUpdateable, Configable},
};

use crate::ability::prelude::*;

#[derive(Clone, Default, Debug, PartialEq, Builder, Getters)]
#[builder(setter(into))]
pub struct GxDownLoad {
    task_file: Option<String>,
    dst_path: String,
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
            let dst_file = ex.eval(self.dst_path())?;
            info!(target: ctx.path(), "task_file :{} ", task_file);
            let artifact_pkg = if task_file.ends_with("yml") {
                ArtifactPackage::from_conf(&PathBuf::from(task_file)).err_conv()?
            } else {
                return ExecReason::Bug("only toml format support".into()).err_result();
            };
            let dst_path = PathBuf::from(dst_file);
            for artifact in artifact_pkg.iter() {
                artifact
                    .addr()
                    .update_rename(&dst_path, artifact.local())
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
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::build_ability("gx.echo")
    }
}

#[cfg(test)]
mod tests {}
