use std::str::FromStr;

use super::domain::gal_keyword_alt;
use super::prelude::*;
use orion_parse::symbol::symbol_comma;
use winnow::combinator::alt;
use winnow::combinator::fail;

use winnow::combinator::separated;
use winnow::ModalResult;
use winnow::Parser;

use crate::ability::assert::*;
use crate::ability::cmd::GxCmdDtoBuilder;
use crate::ability::delegate::ActCall;
use crate::ability::download::GxDownLoad;
use crate::ability::download::GxDownLoadBuilder;
use crate::ability::echo::*;
use crate::ability::read::ReadMode;
use crate::ability::read::RgReadDtoBuilder;
use crate::ability::tpl::TPlEngineType;
use crate::ability::version::*;
use crate::ability::GxCmd;
use crate::ability::GxRead;
use crate::ability::GxTpl;
use crate::ability::TplDTOBuilder;
use crate::components::gxl_var::*;
use crate::expect::ShellOption;
use crate::types::Property;

use super::domain::gal_keyword;
use super::domain::gal_sentence_beg;
use super::domain::gal_sentence_end;
use super::domain::gal_var_assign;
use super::domain::parse_log;

pub fn gal_vars(input: &mut &str) -> ModalResult<RgVars> {
    let mut vars = RgVars::default();
    gal_keyword("gx.vars", input)?;
    let founds = sentence_body.parse_next(input)?;
    for one in founds {
        vars.insert(one.0, one.1);
    }
    Ok(vars)
}

pub fn gal_echo(input: &mut &str) -> ModalResult<GxEcho> {
    let mut watcher = GxEcho::default();
    gal_keyword_alt("gx.echo", "rg.echo", input)?;
    let props = sentence_body.parse_next(input)?;
    for (k, v) in props {
        if k == "value" {
            watcher.set(v.as_str());
        }
    }
    Ok(watcher)
}

pub fn gal_downlaod(input: &mut &str) -> ModalResult<GxDownLoad> {
    let mut down = GxDownLoadBuilder::default();
    gal_keyword_alt("gx.down", "gx.download", input)?;
    let props = sentence_body.parse_next(input)?;
    for (k, v) in &props {
        if k == "file" {
            down.task_file(v.clone());
        }
        if k == "dst_path" {
            down.dst_path(v);
        }
        if k == "dst_name" {
            down.dst_name(v.clone());
        }
    }
    match down.build() {
        Ok(o) => Ok(o),
        Err(e) => {
            error!("{}", e);
            fail.context(wn_desc("gx.down")).parse_next(input)
        }
    }
}

pub fn gal_version(input: &mut &str) -> ModalResult<RgVersion> {
    let mut builder = RgVersionBuilder::default();
    builder.verinc(VerInc::Build);
    builder.export("VERSION".into());
    gal_keyword_alt("gx.ver", "rg.ver", input)?;
    let props = sentence_body.parse_next(input)?;
    for (key, val) in props {
        if key == "file" {
            builder.file(val);
            continue;
        }
        if key == "export" {
            builder.file(val);
            continue;
        }
        if key == "inc" {
            debug!("version inc :{}", val);
            if val == "build" {
                builder.verinc(VerInc::Build);
            }
            if val == "bugfix" {
                builder.verinc(VerInc::Bugfix);
            }
            if val == "feature" {
                builder.verinc(VerInc::Feature);
            }
            if val == "main" {
                builder.verinc(VerInc::Main);
            }
            if val == "null" {
                builder.verinc(VerInc::Null);
            }
        }
    }
    if let Ok(ver) = builder.build() {
        Ok(ver)
    } else {
        fail.parse_next(input)
    }
}

pub fn gal_read(input: &mut &str) -> ModalResult<GxRead> {
    gal_keyword_alt("gx.read", "rg.read", input)?;
    let props = sentence_body.parse_next(input)?;
    let mut builder = RgReadDtoBuilder::default();
    let mut expect = ShellOption::default();
    for one in props {
        let key = one.0.to_lowercase();
        if key == "cmd" {
            builder.cmd(Some(one.1));
            builder.mode(ReadMode::CMD);
        } else if key == "name" {
            builder.name(Some(one.1));
        } else if key == "stdin" {
            builder.stdin(Some(one.1));
            builder.mode(ReadMode::STDIN);
        } else if key == "ini" {
            builder.ini(Some(one.1));
            builder.mode(ReadMode::INI);
        } else if key == "oxc_toml" {
            builder.oxc_toml(Some(one.1));
            builder.mode(ReadMode::OxcToml);
        } else {
            shell_opt_setting(key, one.1, &mut expect);
        }
    }
    builder.expect(expect);
    match builder.build() {
        Ok(dto) => Ok(GxRead::dto_new(dto)),
        Err(e) => {
            error!(target: "parse", "{}",e);
            //println!("{}", e);
            fail.context(wn_desc("read")).parse_next(input)
        }
    }
}

pub fn gal_tpl(input: &mut &str) -> ModalResult<GxTpl> {
    gal_keyword_alt("gx.tpl", "rg.tpl", input)?;
    let props = sentence_body.parse_next(input)?;
    let mut builder = TplDTOBuilder::default();
    for one in props {
        let key = one.0.to_lowercase();
        let val: String = one.1;
        if key == "tpl" {
            builder.tpl(val);
        } else if key == "dst" {
            builder.dst(val);
        } else if key == "data" {
            builder.data(Some(val));
        } else if key == "engine" {
            if let Ok(engine) = TPlEngineType::from_str(val.as_str()) {
                builder.engine(engine);
            } else {
                error!(target: "parse", "unknow engine :{}",val);
                return fail.context(wn_desc("gx.tpl build")).parse_next(input);
            }
        } else if key == "file" {
            builder.file(Some(val));
        }
    }
    match builder.build() {
        Ok(dto) => Ok(GxTpl::from(dto)),
        Err(e) => {
            error!(target: "parse", "{}",e);
            //println!("{}", e);
            fail.context(wn_desc("gx.tpl build")).parse_next(input)
        }
    }
}
/*
pub fn gal_vault(input: &mut &str) -> ModalResult<GxVault> {
    gal_keyword("gx.vault", input)?;
    gal_sentence_beg.parse_next(input)?;
    let props: Vec<(String, String)> = repeat(0.., gal_var_assign).parse_next(input)?;
    gal_sentence_end.parse_next(input)?;
    let mut builder = GxVaultDtoBuilder::default();
    for one in props {
        let key = one.0.to_lowercase();
        let val: String = one.1;
        if key == "vault" {
            builder.vault(val);
        } else if key == "sec_file" {
            builder.sec_file(val);
        } else if key == "sec_env" {
            builder.sec_env(Some(val));
        } else if key == "id_env" {
            builder.id_env(Some(val));
        } else if key == "cache_file" {
            builder.cache_file(Some(val));
        } else if key == "cache_min" {
            builder.cache_min(val.parse().unwrap_or(10));
        }
    }
    match builder.build() {
        Ok(dto) => Ok(GxVault::from(dto)),
        Err(e) => {
            error!(target: "parse", "{}",e);
            //println!("{}", e);
            fail.context(wn_desc("gx.vault build")).parse_next(input)
        }
    }
}
*/
pub fn sentence_body(input: &mut &str) -> ModalResult<Vec<(String, String)>> {
    gal_sentence_beg.parse_next(input)?;
    let props: Vec<(String, String)> =
        separated(0.., gal_var_assign, alt((symbol_comma, symbol_semicolon))).parse_next(input)?;
    opt(alt((symbol_comma, symbol_semicolon))).parse_next(input)?;
    gal_sentence_end.parse_next(input)?;
    Ok(props)
}

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

fn shell_opt_setting(key: String, value: String, expect: &mut ShellOption) {
    if key == "suc" {
        expect.suc = Some(value);
    } else if key == "out" && value.to_lowercase() == "true" {
        expect.outer_print = true;
    } else if key == "err" {
        expect.err = Some(value);
    } else if key == "sudo" && value.to_lowercase() == "true" {
        expect.sudo = true;
    } else if key == "log" {
        expect.log_lev = Some(parse_log((key.as_str(), value.as_str())));
    } else if key == "silence" && value.to_lowercase() == "true" {
        expect.secrecy = true;
    }
}

pub fn gal_call(input: &mut &str) -> ModalResult<ActCall> {
    let name = take_var_path
        .context(wn_desc("<call-name>"))
        .parse_next(input)?;
    let var_props = sentence_body.parse_next(input)?;
    let mut props = Vec::new();
    for v_prop in var_props {
        props.push(Property::from(v_prop))
    }
    let dto = ActCall::from((name, props));
    Ok(dto)
}

pub fn gal_assert(input: &mut &str) -> ModalResult<GxAssert> {
    let mut builder = GxAssertBuilder::default();
    gal_keyword_alt("gx.assert", "rg.assert", input)?;
    let props = sentence_body.parse_next(input)?;
    builder.result(true);
    builder.error(None);
    for (key, val) in props {
        if key == "err" {
            builder.error(Some(val));
        } else if key == "value" {
            builder.value(val);
        } else if key == "expect" {
            builder.expect(val);
        } else if key == "result" {
            if val == "false" {
                builder.result(false);
            }
            if val == "true" {
                builder.result(true);
            }
        }
    }
    if let Ok(ast) = builder.build() {
        Ok(ast)
    } else {
        fail.parse_next(input)
    }
}

pub fn gal_prop(input: &mut &str) -> ModalResult<RgProp> {
    skip_spaces_block.parse_next(input)?;
    let prop = gal_var_assign.parse_next(input)?;
    alt((symbol_comma, symbol_semicolon)).parse_next(input)?;
    let vars = RgProp::ext_new(prop.0, "str".into(), prop.1);
    Ok(vars)
}

pub fn run_gxl<T, F>(gal_fn: F, input: &mut &str) -> ModalResult<T>
where
    F: Fn(&mut &str) -> ModalResult<T>,
{
    match gal_fn(input) {
        Ok(v) => Ok(v),
        Err(e) => {
            println!("{}", e);
            println!("input@:> _{}", input);
            Err(e)
        }
    }
}
#[cfg(test)]
mod tests {

    use orion_common::friendly::New2;
    use orion_error::TestAssert;

    use crate::ability::{read::ReadMode, RgReadDto, TplDTO};

    use super::*;

    #[test]
    fn vars_test() -> ModalResult<()> {
        let mut data = r#"
             gx.vars {
             x = "${PRJ_ROOT}/test/main.py" ;
             y = "${PRJ_ROOT}/test/main.py" ;
             } ;"#;
        let var = gal_vars(&mut data)?;
        let mut expect = RgVars::default();
        expect.append(RgProp::new("X", "${PRJ_ROOT}/test/main.py"));
        expect.append(RgProp::new("Y", "${PRJ_ROOT}/test/main.py"));
        assert_eq!(var, expect);
        assert_eq!(data, "");
        Ok(())
    }

    #[test]
    fn echo_test() -> ModalResult<()> {
        let mut data = r#"
             gx.echo { value = "${PRJ_ROOT}/test/main.py" ; } ;"#;
        let found = run_gxl(gal_echo, &mut data)?;
        let mut expect = GxEcho::default();
        expect.set(r#"${PRJ_ROOT}/test/main.py"#);
        assert_eq!(found, expect);
        assert_eq!(data, "");
        Ok(())
    }
    #[test]
    fn assert_test() {
        let mut data = r#"
             gx.assert { value = "hello" ; expect = "hello" ; err = "errinfo";} ;"#;
        let found = gal_assert(&mut data).unwrap();
        let mut expect = GxAssert::from_diy_error("errinfo");
        expect.expect_eq("hello", "hello");
        //expect.err() = Some(format!("errinfo"));
        assert_eq!(found, expect);
        assert_eq!(data, "");
    }
    #[test]
    fn ver_test() {
        let mut data = r#"
             gx.ver  { file = "./tests/version.txt";  inc = "build" ; } ;"#;
        let found = gal_version(&mut data).unwrap();
        let expect = RgVersion::new(format!("./tests/version.txt"));
        assert_eq!(found, expect);
        assert_eq!(data, "");
    }

    #[test]
    fn call_test() {
        let mut data = r#"
             os.path { dist= "./tests/";  keep= "ture" ; } ;"#;
        let found = run_gxl(gal_call, &mut data).assert();
        let expect = ActCall::from((
            "os.path".to_string(),
            vec![
                Property::from(("dist", "./tests/")),
                Property::from(("keep", "ture")),
            ],
        ));
        assert_eq!(found, expect);
        assert_eq!(data, "");
    }

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

    #[test]
    fn read_cmd_test1() {
        let mut dto = RgReadDto::default();
        dto.expect = ShellOption::default();
        dto.mode = ReadMode::CMD;
        dto.cmd = Some(format!("echo galaxy-1.0"));
        dto.name = Some(format!("RG"));
        let mut data = r#"
                 gx.read {
                 name = "RG";
                 cmd  = "echo galaxy-1.0";
                 } ;"#;
        let obj = run_gxl(gal_read, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(&dto, obj.dto());
    }
    #[test]
    fn read_cmd_test2() {
        let mut dto = RgReadDto::default();
        dto.expect = ShellOption::default();
        dto.expect.log_lev = Some(log::Level::Info);

        dto.mode = ReadMode::CMD;
        dto.cmd = Some(format!("echo galaxy-1.0"));
        dto.name = Some(format!("RG"));
        dto.expect.err = Some(format!("you err"));

        let mut data = r#"
                 gx.read{
                 name = "RG";
                 cmd  = "echo galaxy-1.0";
                 err = "you err";
                 log = "1";
                 } ;"#;

        let obj = run_gxl(gal_read, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(&dto, obj.dto());
    }
    #[test]
    fn read_ini_test() {
        let mut dto = RgReadDto::default();
        dto.expect = ShellOption::default();
        dto.ini = Some(format!("vars.ini"));
        dto.mode = ReadMode::INI;

        let mut data = r#"
                 gx.read {
                 ini = "vars.ini";
                 } ;"#;
        let obj = run_gxl(gal_read, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(&dto, obj.dto());
    }
    #[test]
    fn read_toml_test() {
        let mut dto = RgReadDto::default();
        dto.expect = ShellOption::default();
        dto.oxc_toml = Some(format!("vars.toml"));
        dto.mode = ReadMode::OxcToml;

        let mut data = r#"
                 gx.read {
                 oxc_toml = "vars.toml";
                 } ;"#;
        let obj = run_gxl(gal_read, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(&dto, obj.dto());
    }

    #[test]
    fn read_stdin_test() {
        let mut dto = RgReadDto::default();
        dto.expect = ShellOption::default();
        dto.mode = ReadMode::STDIN;
        dto.name = Some(format!("name"));
        dto.stdin = Some(format!("please input you name"));

        let mut data = r#"
                 gx.read {
                 stdin = "please input you name";
                 name  = "name";
                 } ;"#;

        let obj = run_gxl(gal_read, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(&dto, obj.dto());
    }

    #[test]
    fn tpl_sample() {
        let tpl = "${PRJ_ROOT}/conf_tpl.toml";
        let dst = "${PRJ_ROOT}/conf.toml";
        let mut data = r#"
                 gx.tpl {
                 tpl = "${PRJ_ROOT}/conf_tpl.toml" ;
                 dst = "${PRJ_ROOT}/conf.toml" ;
                 } ;"#;
        let mut dto = TplDTO::default();
        dto.tpl = tpl.to_string();
        dto.dst = dst.to_string();
        let obj = run_gxl(gal_tpl, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(&dto, obj.dto());
    }
    #[test]
    fn tpl_data() {
        let tpl = "${PRJ_ROOT}/conf_tpl.toml";
        let dst = "${PRJ_ROOT}/conf.toml";
        let mut data = r#"
                 gx.tpl {
                 tpl = "${PRJ_ROOT}/conf_tpl.toml" ;
                 dst = "${PRJ_ROOT}/conf.toml";
                 data = ^"{"branchs": ["develop","issue/11"]} "^;
                 } ;"#;
        let obj = run_gxl(gal_tpl, &mut data).assert();
        let mut dto = TplDTO::default();
        dto.tpl = tpl.to_string();
        dto.dst = dst.to_string();
        dto.data = Some(String::from("{\"branchs\": [\"develop\",\"issue/11\"]} "));
        assert_eq!(data, "");
        assert_eq!(&dto, obj.dto());
    }
}
