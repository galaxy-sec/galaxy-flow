use std::path::PathBuf;

use orion_common::serde::Configable;
use orion_error::ToStructError;
use orion_variate::{ext::ArtifactPackage, types::LocalUpdate, update::UpdateOptions};

use crate::ability::prelude::*;

#[derive(Clone, Default, Debug, PartialEq, Builder, Getters)]
#[builder(setter(into))]
pub struct GxArtifact {
    pkg_file: String,
    dst_path: String,
    #[builder(default = "true")]
    download: bool,
}

// impl DefaultDTO for RgEcho {}

#[async_trait]
impl AsyncRunnableTrait for GxArtifact {
    async fn async_exec(&self, ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        let exp = EnvExpress::from_env_mix(vars_dict.global().clone());
        let pkg_file = exp.eval(self.pkg_file())?;
        let dst_file = exp.eval(self.dst_path())?;
        info!(target: ctx.path(), "task_file :{} ", pkg_file);
        let artifact_pkg = if pkg_file.ends_with("yml") {
            ArtifactPackage::from_conf(&PathBuf::from(pkg_file)).owe_data()?
        } else {
            return ExecReason::Bug("only yml format support".into()).err_result();
        };
        let dst_path = PathBuf::from(dst_file);
        for artifact in artifact_pkg.iter() {
            artifact
                .origin_addr()
                .update_local_rename(&dst_path, artifact.local(), &UpdateOptions::default())
                .await
                .owe_res()?;
        }

        Ok(TaskValue::from((vars_dict, ExecOut::Ignore)))
    }
}

impl ComponentMeta for GxArtifact {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.artifact")
    }
}

#[cfg(test)]
mod tests {}
