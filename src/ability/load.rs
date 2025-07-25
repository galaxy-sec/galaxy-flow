use std::path::PathBuf;

use orion_error::ToStructError;
use orion_variate::{addr::HttpAddr, update::UpdateOptions};

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
    async fn async_exec(&self, _ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        let ex = EnvExpress::from_env_mix(vars_dict.global().clone());
        let mut addr = HttpAddr::from(ex.eval(self.svc_url())?);
        if let (Some(username), Some(password)) = (self.username(), self.password()) {
            let username = ex.eval(username)?;
            let password = ex.eval(password)?;
            addr = addr.with_credentials(username, password);
        }
        let local_file = ex.eval(self.local_file())?;
        let mut action = Action::from("gx.upload").with_target(&local_file);
        let local_file_path = PathBuf::from(&local_file);
        let method = ex.eval(self.method())?.to_uppercase();
        if local_file_path.exists() {
            addr.upload(&local_file_path, &method).await.owe_res()?;
            action.finish();
            Ok(TaskValue::from((vars_dict, ExecOut::Action(action))))
        } else {
            return ExecReason::Miss("local_file".into())
                .err_result()
                .want("gx.upload")
                .with(&local_file_path);
        }
    }
}

impl ComponentMeta for GxUpLoad {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.upload")
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxDownLoad {
    async fn async_exec(&self, _ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        let ex = EnvExpress::from_env_mix(vars_dict.global().clone());
        let mut addr = HttpAddr::from(ex.eval(self.svc_url())?);
        if let (Some(username), Some(password)) = (self.username(), self.password()) {
            let username = ex.eval(username)?;
            let password = ex.eval(password)?;
            addr = addr.with_credentials(username, password);
        }
        let local_file = ex.eval(self.local_file())?;
        let mut action = Action::from("gx.download").with_target(&local_file);
        let local_file_path = PathBuf::from(&local_file);
        if let Some(true) = local_file_path.parent().map(|x| x.exists()) {
            addr.download(&local_file_path, &UpdateOptions::default())
                .await
                .owe_res()?;
            action.finish();
            Ok(TaskValue::from((vars_dict, ExecOut::Action(action))))
        } else {
            return ExecReason::Miss("parent path not exists".into())
                .err_result()
                .want("gx.download")
                .with(&local_file_path);
        }
    }
}

impl ComponentMeta for GxDownLoad {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.donwload")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gx_download_parent_exists() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("README").to_str().unwrap().to_string();
        let download = GxDownLoadBuilder::default()
            .local_file(file_path.clone())
            .svc_url("https://mirrors.aliyun.com/postgresql/".to_string())
            .build()
            .unwrap();

        let vars_dict = VarSpace::default();
        let ctx = ExecContext::default();
        let result = download.async_exec(ctx, vars_dict).await;
        assert!(result.is_ok());
    }
}
