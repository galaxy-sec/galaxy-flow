use std::path::PathBuf;

use orion_error::ErrorConv;
use orion_syspec::{addr::HttpAddr, error::ToErr};

use crate::ability::prelude::*;

#[derive(Clone, Default, Debug, PartialEq, Builder, Getters)]
#[builder(setter(into))]
pub struct GxUpLoad {
    local_file: String,
    svc_url: String,
    #[builder(default)]
    username: Option<String>,
    #[builder(default)]
    password: Option<String>,
    method: String,
}

#[derive(Clone, Default, Debug, PartialEq, Builder, Getters)]
#[builder(setter(into))]
pub struct GxDownLoad {
    local_file: String,
    svc_url: String,
    #[builder(default)]
    username: Option<String>,
    #[builder(default)]
    password: Option<String>,
}

#[async_trait]
impl AsyncRunnableTrait for GxUpLoad {
    async fn async_exec(&self, _ctx: ExecContext, def: VarSpace) -> VTResult {
        let ex = EnvExpress::from_env_mix(def.globle().clone());
        let mut addr = HttpAddr::from(ex.eval(self.svc_url())?);
        if let (Some(username), Some(password)) = (self.username(), self.password()) {
            let username = ex.eval(username)?;
            let password = ex.eval(password)?;
            addr = addr.with_credentials(username, password);
        }
        let dst_file = ex.eval(self.local_file())?;
        let mut task = Task::from("gx.download").with_target(&dst_file);
        let dst_path = PathBuf::from(&dst_file);
        let method = ex.eval(self.method())?.to_uppercase();
        if dst_path.exists() {
            addr.upload(&dst_path, &method).await.err_conv()?;
            task.finish();
            Ok((def, ExecOut::Task(task)))
        } else {
            return ExecReason::Miss("dst_file".into()).err_result();
        }
    }
}

impl ComponentMeta for GxUpLoad {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::build_ability("gx.upload")
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxDownLoad {
    async fn async_exec(&self, _ctx: ExecContext, def: VarSpace) -> VTResult {
        let ex = EnvExpress::from_env_mix(def.globle().clone());
        let mut addr = HttpAddr::from(ex.eval(self.svc_url())?);
        if let (Some(username), Some(password)) = (self.username(), self.password()) {
            let username = ex.eval(username)?;
            let password = ex.eval(password)?;
            addr = addr.with_credentials(username, password);
        }
        let dst_file = ex.eval(self.local_file())?;
        let mut task = Task::from("gx.download").with_target(&dst_file);
        let dst_path = PathBuf::from(&dst_file);
        if let Some(true) = dst_path.parent().map(|x| x.exists()) {
            addr.download(&dst_path).await.err_conv()?;
            task.finish();
            Ok((def, ExecOut::Task(task)))
        } else {
            return ExecReason::Miss("dst_file".into()).err_result();
        }
    }
}

impl ComponentMeta for GxDownLoad {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::build_ability("gx.donwload")
    }
}

#[cfg(test)]
mod tests {}
