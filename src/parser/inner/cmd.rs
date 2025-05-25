use super::super::prelude::*;
use super::common::{sentence_body, shell_opt_setting};

use crate::ability::cmd::GxCmdDtoBuilder;
use crate::ability::GxCmd;
use crate::expect::ShellOption;
use crate::parser::domain::gal_keyword_alt;

pub fn gal_cmd(input: &mut &str) -> ModalResult<GxCmd> {
    let mut builder = GxCmdDtoBuilder::default();
    gal_keyword_alt("gx.cmd", "rg.cmd", input)?;
    let props = sentence_body.parse_next(input)?;
    let mut expect = ShellOption::default();
    builder.expect(ShellOption::default());
    for one in props {
        let key = one.0.to_lowercase();
        if key == "forword" || key == "cmd" {
            builder.forword(one.1);
        } else {
            shell_opt_setting(key, one.1, &mut expect);
        }
    }
    builder.expect(expect);
    if let Ok(dto) = builder.build() {
        Ok(GxCmd::dto_new(dto))
    } else {
        fail.parse_next(input)
    }
}
#[cfg(test)]
mod tests {

    use orion_error::TestAssert;

    use crate::parser::inner::common::{gal_call, run_gxl};

    use super::*;

    #[test]
    fn cmd_test() {
        let expect = ShellOption::default();
        let mut data = r#"
             gx.cmd{
             cmd = "${PRJ_ROOT}/do.sh";
             } ;"#;
        let obj = gal_cmd(&mut data).assert();
        //let (input, obj) = show_err(data, RgCmdParser::default().parse(ctx, data)).unwrap();
        let xpt = GxCmdDtoBuilder::default()
            .forword("${PRJ_ROOT}/do.sh".into())
            .expect(expect)
            .build()
            .unwrap();
        assert_eq!(data, "");
        assert_eq!(obj, GxCmd::dto_new(xpt));
    }
    #[test]
    fn cmd_test2() {
        let mut expect = ShellOption::default();
        expect.log_lev = Some(log::Level::Info);
        let mut data = r#"
             gx.cmd{
             cmd = "${PRJ_ROOT}/do.sh";
             err = "you err";
             log = "1";
             } ;"#;
        let obj = gal_cmd(&mut data).assert();
        expect.err = Some(String::from("you err"));
        let xpt = GxCmdDtoBuilder::default()
            .forword("${PRJ_ROOT}/do.sh".into())
            .expect(expect)
            .build()
            .unwrap();
        assert_eq!(data, "");
        assert_eq!(obj, GxCmd::dto_new(xpt));
    }

    #[test]
    fn cmd_test3() {
        let mut data = r#"
            conf.tpl {
              tpl = "${MAIN_CONF}/tpls/test.sh"  ;
              dst = "${MAIN_CONF}/options/test.sh" ;
              data = ^"hello"^;
            }
            "#;
        let _obj = run_gxl(gal_call, &mut data).assert();
        assert_eq!(data, "");
    }
}
