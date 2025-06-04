use super::super::prelude::*;
use super::common::{sentence_call_args, shell_opt_setting};

use crate::ability::cmd::GxCmdDtoBuilder;
use crate::ability::GxCmd;
use crate::expect::ShellOption;
use crate::parser::domain::{gal_keyword, gal_keyword_alt};

pub fn gal_cmd(input: &mut &str) -> ModalResult<GxCmd> {
    let mut builder = GxCmdDtoBuilder::default();
    gal_keyword_alt("gx.cmd", "rg.cmd", input)?;
    let props = sentence_call_args.parse_next(input)?;
    let mut expect = ShellOption::default();
    builder.expect(ShellOption::default());
    for one in props {
        let key = one.0.to_lowercase();
        if key == "default" || key == "cmd" {
            builder.cmd(one.1);
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

/// read ```cmd  ... ``` to GxCmd;
pub fn gal_cmd_block(input: &mut &str) -> ModalResult<GxCmd> {
    let mut builder = GxCmdDtoBuilder::default();
    builder.expect(ShellOption::default());
    // 1. 匹配开始的 ```cmd
    gal_keyword("```cmd", input)?;

    // 2. 跳过可能的空白和换行
    //*input = input.trim_start();
    let cmd_content = take_until(0.., "```")
        .context(wn_desc("cmd block"))
        .parse_next(input)?;
    "```".context(wn_desc("block-end")).parse_next(input)?;
    builder.cmd(cmd_content.trim().to_string());
    multispace0.parse_next(input)?;
    // 6. 构建并返回 GxCmd
    if let Ok(dto) = builder.build() {
        Ok(GxCmd::dto_new(dto))
    } else {
        fail.context(wn_desc("cmd-block build fail"))
            .parse_next(input)
    }
}

#[cfg(test)]
mod tests {

    use orion_error::TestAssert;

    use crate::parser::{inner::common::run_gxl, stc_blk::gal_block};

    use super::*;

    #[test]
    fn cmd_test_simple() {
        let expect = ShellOption::default();
        let mut data = r#"
             gx.cmd(
             cmd : "${PRJ_ROOT}/do.sh",
             ) ;"#;
        let obj = gal_cmd(&mut data).assert();
        //let (input, obj) = show_err(data, RgCmdParser::default().parse(ctx, data)).unwrap();
        let xpt = GxCmdDtoBuilder::default()
            .cmd("${PRJ_ROOT}/do.sh".into())
            .expect(expect)
            .build()
            .unwrap();
        assert_eq!(data, "");
        assert_eq!(obj, GxCmd::dto_new(xpt));
    }
    #[test]
    fn cmd_test_default() {
        let expect = ShellOption::default();
        let mut data = r#"
             gx.cmd( "${PRJ_ROOT}/do.sh" ) ;"#;
        let obj = gal_cmd(&mut data).assert();
        //let (input, obj) = show_err(data, RgCmdParser::default().parse(ctx, data)).unwrap();
        let xpt = GxCmdDtoBuilder::default()
            .cmd("${PRJ_ROOT}/do.sh".into())
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
             gx.cmd(
             cmd : "${PRJ_ROOT}/do.sh",
             err : "you err",
             log : "1",
             ) ;"#;
        let obj = gal_cmd(&mut data).assert();
        expect.err = Some(String::from("you err"));
        let xpt = GxCmdDtoBuilder::default()
            .cmd("${PRJ_ROOT}/do.sh".into())
            .expect(expect)
            .build()
            .unwrap();
        assert_eq!(data, "");
        assert_eq!(obj, GxCmd::dto_new(xpt));
    }

    #[test]
    fn cmd_test3() {
        let mut expect = ShellOption::default();
        expect.log_lev = Some(log::Level::Info);
        let mut data = r#"
             gx.cmd(
             "${PRJ_ROOT}/do.sh",
             err : "you err",
             log : "1",
             ) ;"#;
        let obj = gal_cmd(&mut data).assert();
        expect.err = Some(String::from("you err"));
        let xpt = GxCmdDtoBuilder::default()
            .cmd("${PRJ_ROOT}/do.sh".into())
            .expect(expect)
            .build()
            .unwrap();
        assert_eq!(data, "");
        assert_eq!(obj, GxCmd::dto_new(xpt));
    }

    #[test]
    fn cmd_block_1() {
        let mut data = r#"```cmd echo ${HOME};```"#;
        let obj = run_gxl(gal_cmd_block, &mut data).assert();
        assert_eq!(obj.dto().cmd, "echo ${HOME};");

        assert_eq!(data, "");
    }
    #[test]
    fn cmd_block_2() {
        let mut data = r#"```cmd
echo ${HOME};
echo ${HOME};
            ```"#;
        let obj = run_gxl(gal_cmd_block, &mut data).assert();
        assert_eq!(obj.dto().cmd, "echo ${HOME};\necho ${HOME};");

        assert_eq!(data, "");
    }
    #[test]
    fn cmd_block_3() {
        let mut data = r#"
            {
            gx.cmd ( cmd : "echo ${HOME}" )
            ```cmd
                cp /a /b;
                echo ${HOME};
                echo ${HOME};
            ```
            }
            "#;
        let obj = run_gxl(gal_block, &mut data).assert();
        assert_eq!(obj.items().len(), 2);
        assert_eq!(data, "");
    }
}
