use crate::ability::prelude::*;
use crate::execution::task::Task;
use handlebars::{to_json, Handlebars};
use orion_error::WithContext;
use serde::Serialize;
use std::fmt::Display;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Default, Debug, PartialEq, Getters, Clone)]
pub struct GxTpl {
    dto: TplDTO,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum TPlEngineType {
    #[default]
    Handlebars,
    Helm,
}
impl FromStr for TPlEngineType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "handlebars" => Ok(Self::Handlebars),
            "helm" => Ok(Self::Helm),
            _ => Err("unknow engine".to_string()),
        }
    }
}
impl Display for TPlEngineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            TPlEngineType::Handlebars => "handlebars",
            TPlEngineType::Helm => "helm",
        };
        write!(f, "{}", msg)
    }
}

#[derive(Clone, Debug, PartialEq, Builder, Default)]
pub struct TplDTO {
    pub tpl: String,
    pub dst: String,
    #[builder(default = "None")]
    pub data: Option<String>,
    #[builder(default = "None")]
    pub file: Option<String>,
    #[builder(default = "TPlEngineType::Handlebars")]
    pub engine: TPlEngineType,
}
impl From<TplDTO> for GxTpl {
    fn from(value: TplDTO) -> Self {
        Self { dto: value }
    }
}
impl Display for TplDTO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[tpl] {},", self.tpl)?;
        write!(f, "[dst] {},", self.dst)?;
        write!(f, "[engine] {},", self.engine)?;
        write!(
            f,
            "[data|file|env]: {},",
            self.data
                .clone()
                .or(self.file.clone())
                .unwrap_or("env".to_string())
        )?;
        Ok(())
    }
}

impl GxTpl {
    pub fn render_path(&self, ctx: ExecContext, dto: &TplDTO, dict: VarSpace) -> ExecResult<()> {
        info!(target: ctx.path(), "gx.tpl : {}", dto);
        let exp = EnvExpress::from_env_mix(dict.globle().clone());
        let tpl = PathBuf::from(exp.eval(dto.tpl.as_str())?);
        let dst = PathBuf::from(exp.eval(dto.dst.as_str())?);

        let mut err_ctx = WithContext::want("render tpl path");
        // 处理目录模板
        if dto.engine != TPlEngineType::Handlebars {
            return Err(ExecReason::Args(format!(
                "Only support Handlebars Engine {:?}",
                dto.engine
            ))
            .into());
        }
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        // 准备数据

        let data = if let Some(json_file) = &dto.file {
            let json_file = exp.eval(json_file.as_str())?;
            err_ctx.with("file", json_file.as_str());
            let content = std::fs::read_to_string(json_file.as_str())
                .owe_data()
                .with(&err_ctx)?;
            err_ctx.with("need-fmt", "json");
            serde_json::from_str(content.as_str())
                .owe_data()
                .with(&err_ctx)?
        } else if let Some(data_str) = &dto.data {
            serde_json::from_str(data_str.as_str())
                .owe_data()
                .with(&err_ctx)?
        } else {
            to_json(dict.globle().export())
        };
        if tpl.is_dir() {
            self.render_dir_impl(ctx, &handlebars, &tpl, &dst, &data)
        } else {
            self.render_file_impl(ctx, &handlebars, &tpl, &dst, &data)
        }
    }
    fn render_dir_impl<T: Serialize>(
        &self,
        ctx: ExecContext,
        handlebars: &Handlebars,
        tpl_dir: &PathBuf,
        dst: &PathBuf,
        data: &T,
    ) -> ExecResult<()> {
        debug!(target: ctx.path(), "tpl dir: {}", tpl_dir.display());
        for entry in walkdir::WalkDir::new(tpl_dir) {
            let entry = entry.owe_data()?;
            let entry_path = entry.path();
            let relative_path = entry_path.strip_prefix(tpl_dir).owe_data()?;
            let dst_path = Path::new(dst).join(relative_path);

            if entry_path.is_dir() {
                // 如果是目录，确保在目标位置创建对应的目录
                std::fs::create_dir_all(&dst_path).owe_sys()?;
                debug!(target: ctx.path(), "created dir: {}", dst_path.display());
            } else if entry_path.is_file() {
                // 如果是文件，则渲染模板
                self.render_file_impl(
                    ctx.clone(),
                    handlebars,
                    &entry_path.to_path_buf(),
                    &dst_path,
                    &data,
                )?;
            }
            // 忽略其他类型（如符号链接等）
        }
        Ok(())
    }

    fn render_file_impl<T: Serialize>(
        &self,
        ctx: ExecContext,
        handlebars: &Handlebars,
        tpl: &PathBuf,
        dst: &PathBuf,
        data: &T,
    ) -> ExecResult<()> {
        debug!(target: ctx.path(), "tpl:{}", tpl.display());
        debug!(target: ctx.path(),  "dst:{}", dst.display());

        let mut err_ctx = WithContext::want("render tpl");
        err_ctx.with("tpl", tpl.to_string_lossy());
        // 2. 验证模板文件
        let tpl_path = Path::new(&tpl);
        if !tpl_path.exists() {
            return Err(
                ExecReason::Args(format!("Template file not found: {}", tpl.display())).into(),
            );
        }
        if !tpl_path.is_file() {
            return Err(ExecReason::Args(format!(
                "Template path is not a file: {}",
                tpl.display()
            ))
            .into());
        }
        err_ctx.with("dst", dst.to_string_lossy());
        // 3. 准备目标文件
        let dst_path = Path::new(&dst);
        if let Some(parent) = dst_path.parent() {
            std::fs::create_dir_all(parent).owe_sys()?;
        }
        if dst_path.exists() {
            std::fs::remove_file(dst).owe_sys()?;
        }

        // 4. 日志记录
        debug!(target: ctx.path(), "Processing template: {} → {}", tpl.display(), dst.display());

        // 5. 读取模板内容
        let template = std::fs::read_to_string(tpl).owe_data().with(&err_ctx)?;

        let mut dst_file = File::create(dst).map_err(|e| {
            ExecReason::Args(format!(
                "Failed to create output file {}: {}",
                dst.display(),
                e
            ))
        })?;

        handlebars
            .render_template_to_write(&template, data, &mut dst_file)
            .owe_biz()
            .with(&err_ctx)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o644); // rw-r--r--
            std::fs::set_permissions(dst, perms)
                .owe_sys()
                .with(&err_ctx)?;
        }
        println!("render {:30} ---> {}", tpl.display(), dst.display());

        debug!(target: ctx.path(), "Successfully generated: {}", dst.display());
        Ok(())
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxTpl {
    async fn async_exec(&self, ctx: ExecContext, vars_dict: VarSpace) -> VTResult {
        let mut task = Task::from("build tpl file");
        self.render_path(ctx, &self.dto, vars_dict.clone())?;
        task.finish();
        Ok((vars_dict, ExecOut::Task(task)))
    }
}

impl ComponentMeta for GxTpl {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::build_ability("gx.tpl")
    }
}

impl GxTpl {
    pub fn new(tpl: String, dst: String) -> Self {
        let obj = TplDTO {
            tpl,
            dst,
            ..Default::default()
        };
        GxTpl { dto: obj }
    }
    pub fn new_by_dto(dto: TplDTO) -> Self {
        GxTpl { dto }
    }
}

#[cfg(test)]
mod tests {

    use crate::{ability::ability_env_init, traits::Setter};

    use super::*;
    fn files_identical(path1: &str, path2: &str) -> std::io::Result<bool> {
        let content1 = std::fs::read(path1)?;
        let content2 = std::fs::read(path2)?;
        Ok(content1 == content2)
    }

    #[tokio::test]
    async fn tpl_test_by_envars() {
        let (mut context, mut def) = ability_env_init();
        let tpl = format!(
            "{}/examples/template/conf/tpls/nginx.conf",
            context.cur_path()
        );
        let dst = format!(
            "{}/examples/template/conf/used/nginx_env.conf",
            context.cur_path()
        );
        let conf_tpl = GxTpl::new(tpl.clone(), dst.clone());
        context.append("RG");
        def.global_mut().set("GXL_PRJ_ROOT", "/home/galaxy");
        def.global_mut().set("DOMAIN", "www.galaxy-sec.org");
        def.global_mut().set("SOCK_FILE", "galaxy.socket");
        conf_tpl.async_exec(context.clone(), def).await.unwrap();

        let ngx_dst = format!(
            "{}/examples/template/conf/used/nginx_env.conf",
            context.cur_path()
        );
        let ngx_xpt = format!(
            "{}/examples/template/conf/expect/nginx.conf",
            context.cur_path()
        );
        assert!(files_identical(ngx_dst.as_str(), ngx_xpt.as_str()).unwrap());
    }
    #[tokio::test]
    async fn tpl_test_by_json() {
        //use ValueDict
        let (context, def) = ability_env_init();
        let root = format!("{}/examples/template/conf", context.cur_path());
        let tpl = format!("{}/tpls", root);
        let dst = format!("{}/used", root);
        let file = format!("{}/value.json", root);

        let mut dto = TplDTO::default();
        dto.tpl = tpl.clone();
        dto.dst = dst.clone();
        dto.file = Some(file.clone());

        let conf_tpl = GxTpl::new_by_dto(dto);
        conf_tpl.async_exec(context.clone(), def).await.unwrap();

        let ngx_dst = format!("{}/used/nginx.conf", root);
        let ngx_xpt = format!("{}/expect/nginx.conf", root);
        assert!(files_identical(ngx_dst.as_str(), ngx_xpt.as_str()).unwrap());

        let sys_dst = format!("{}/used/sys.toml", root);
        let sys_tpl = format!("{}/tpls/sys.toml", root);
        assert!(files_identical(sys_dst.as_str(), sys_tpl.as_str()).unwrap());
    }

    #[test]
    fn handlebar_test() {
        let mut handlebars = Handlebars::new();
        let source = "hello {{world}} {{PRJ_HOME}}!";
        assert!(handlebars.register_template_string("t1", source).is_ok());
        let mut def = VarSpace::default();
        def.global_mut().set("world", "世界!".to_string());
        def.global_mut().set("PRJ_HOME", "home".to_string());
        let data = def.globle().export();
        assert_eq!(handlebars.render("t1", &data).unwrap(), "hello 世界! home!");

        let data = r#"
        {
            "world": "John",
            "PRJ_HOME": "home/bj",
            "age": 43
        }"#;
        let v: serde_json::Value = serde_json::from_str(data).unwrap();
        assert_eq!(handlebars.render("t1", &v).unwrap(), "hello John home/bj!");
    }
    #[test]
    fn handlebar_test1() {
        let mut handlebars = Handlebars::new();
        let source = "hello {{PRJ.HOME}}!";
        assert!(handlebars.register_template_string("t1", source).is_ok());
        let data = r#"
        {
            "PRJ" : { "HOME": "home" }
        }"#;
        let v: serde_json::Value = serde_json::from_str(data).unwrap();
        assert_eq!(handlebars.render("t1", &v).unwrap(), "hello home!");
    }
}
