use crate::ability::prelude::*;
use crate::execution::task::Task;
use handlebars::{to_json, Handlebars};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Default, Debug, PartialEq, Getters, Clone)]
pub struct GxTpl {
    dto: RgTplDto,
}

#[derive(Clone, Debug, PartialEq, Builder, Default)]
pub struct RgTplDto {
    pub tpl: String,
    pub dst: String,
    //pub subs: Vec<RgProp>,
    //#[builder(default = "String::new")]
    #[builder(default = "\"\".to_string()")]
    pub data: String,
    //pub log_lev: log::Level,
}
impl From<RgTplDto> for GxTpl {
    fn from(value: RgTplDto) -> Self {
        Self { dto: value }
    }
}

impl GxTpl {
    pub fn render(&self, ctx: ExecContext, dto: &RgTplDto, dict: VarsDict) -> ExecResult<()> {
        let exp = EnvExpress::from_env_mix(dict.clone());
        let tpl = exp.eval(dto.tpl.as_str())?;
        let dst = exp.eval(dto.dst.as_str())?;
        let data_str = exp.eval(dto.data.as_str())?;

        debug!(target: ctx.path(), "tpl:{}", tpl);
        debug!(target: ctx.path(),  "dst:{}", dst);
        let dst_path = Path::new(dst.as_str());
        if dst_path.exists() {
            std::fs::remove_file(dst.as_str()).owe_sys()?;
        }

        debug!(target: ctx.path(), "read tpl src file: {}", tpl);
        let mut tpl_file = File::open(tpl.as_str())
            .map_err(|_| ExecReason::Args(format!("read tpl src fail!{}", tpl)))?;
        debug!(target: ctx.path(), "crate tpl dst file: {}", dst);
        let mut dst_file = File::create(dst.as_str())
            .map_err(|_| ExecReason::Args(format!("create tpl dst fail! {}", dst)))?;

        let handlebars = Handlebars::new();

        let data = if data_str.is_empty() {
            to_json(dict.export())
        } else {
            serde_json::from_str(data_str.as_str()).owe_data()?
        };
        let mut template = String::new();
        tpl_file.read_to_string(&mut template).owe_data()?;
        handlebars
            .render_template_to_write(&template, &data, &mut dst_file)
            .owe_data()?;
        debug!(target:ctx.path(),"tpl({}) have generate  file({})", tpl,dst);
        Ok(())
    }
}

impl RunnableTrait for GxTpl {
    fn exec(&self, ctx: ExecContext, def: &mut VarsDict) -> EOResult {
        let mut task = Task::from("build tpl file");
        self.render(ctx, &self.dto, def.clone())?;
        task.finish();
        Ok(ExecOut::Task(task))
    }
}

impl ComponentRunnable for GxTpl {
    fn meta(&self) -> RgoMeta {
        RgoMeta::build_ability("gx.tpl")
    }
}

impl GxTpl {
    pub fn new(tpl: String, dst: String) -> Self {
        let obj = RgTplDto {
            tpl,
            dst,
            ..Default::default()
        };
        GxTpl { dto: obj }
    }
    pub fn new_by_dto(dto: RgTplDto) -> Self {
        GxTpl { dto }
    }
}

#[cfg(test)]
mod tests {

    use crate::{ability::ability_env_init, traits::Setter};

    use super::*;

    #[test]
    fn tpl_test() {
        let (mut context, mut def) = ability_env_init();
        let tpl = format!("{}/example/conf/tpls/nginx.conf", context.cur_path());
        let dst = format!("{}/example/conf/options/nginx.conf", context.cur_path());
        let xpt = format!("{}/example/conf/expect/nginx.conf", context.cur_path());
        let conf_tpl = GxTpl::new(tpl.clone(), dst.clone());
        context.append("RG");
        def.set("RG_PRJ_ROOT", "/home/galaxy");
        def.set("DOMAIN", "www.galaxy-sec.org");
        def.set("SOCK_FILE", "galaxy.socket");
        conf_tpl.exec(context.clone(), &mut def).unwrap();

        let xpt_ct = std::fs::read_to_string(xpt);
        let dst_ct = std::fs::read_to_string(dst);
        assert_eq!(dst_ct.unwrap(), xpt_ct.unwrap());
    }
    #[test]
    fn handlebar_test() {
        let mut handlebars = Handlebars::new();
        let source = "hello {{world}} {{PRJ_HOME}}!";
        assert!(handlebars.register_template_string("t1", source).is_ok());
        let mut def = VarsDict::default();
        def.set("world", "世界!".to_string());
        def.set("PRJ_HOME", "home".to_string());
        let data = def.export();
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
