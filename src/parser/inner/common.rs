use super::super::prelude::*;
use orion_parse::symbol::symbol_comma;
use winnow::combinator::separated;

use crate::ability::delegate::ActCall;
use crate::components::{gxl_var::*, GxlProps};
use crate::expect::ShellOption;
use crate::parser::domain::{
    gal_call_beg, gal_call_end, gal_keyword, gal_sentence_beg, gal_sentence_end, gal_var_assign,
    gal_var_input, parse_log,
};
use crate::types::Property;

pub fn gal_vars(input: &mut &str) -> ModalResult<GxlProps> {
    let mut vars = GxlProps::default();
    gal_keyword("gx.vars", input)?;
    let founds = sentence_body.parse_next(input)?;
    for one in founds {
        vars.insert(one.0, one.1);
    }
    Ok(vars)
}
pub fn sentence_call_args(input: &mut &str) -> ModalResult<Vec<(String, String)>> {
    gal_call_beg.parse_next(input)?;
    let props: Vec<(String, String)> =
        separated(0.., gal_var_input, symbol_comma).parse_next(input)?;
    opt(symbol_comma).parse_next(input)?;
    gal_call_end.parse_next(input)?;
    Ok(props)
}

pub fn sentence_body(input: &mut &str) -> ModalResult<Vec<(String, String)>> {
    gal_sentence_beg.parse_next(input)?;
    let props: Vec<(String, String)> =
        separated(0.., gal_var_assign, alt((symbol_comma, symbol_semicolon))).parse_next(input)?;
    opt(alt((symbol_comma, symbol_semicolon))).parse_next(input)?;
    gal_sentence_end.parse_next(input)?;
    Ok(props)
}

pub fn shell_opt_setting(key: String, value: String, expect: &mut ShellOption) {
    if key == "suc" {
        expect.suc = Some(value);
    } else if key == "quiet" {
        if value.to_lowercase() == "true" {
            expect.quiet = true;
        } else if value.to_lowercase() == "true" {
            expect.quiet = false;
        }
    } else if key == "out" {
        if value.to_lowercase() == "true" {
            expect.quiet = false;
        } else if value.to_lowercase() == "true" {
            expect.quiet = true;
        }
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
    let var_props = sentence_call_args.parse_next(input)?;
    let mut props = Vec::new();
    for v_prop in var_props {
        props.push(Property::from(v_prop))
    }
    let dto = ActCall::from((name, props));
    Ok(dto)
}

pub fn gal_prop(input: &mut &str) -> ModalResult<GxlVar> {
    skip_spaces_block.parse_next(input)?;
    let prop = gal_var_assign.parse_next(input)?;
    alt((symbol_comma, symbol_semicolon)).parse_next(input)?;
    let vars = GxlVar::ext_new(prop.0, "str".into(), prop.1);
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

    use super::*;

    #[test]
    fn vars_test() -> ModalResult<()> {
        let mut data = r#"
             gx.vars {
             x = "${PRJ_ROOT}/test/main.py" ;
             y = "${PRJ_ROOT}/test/main.py" ;
             } ;"#;
        let var = gal_vars(&mut data)?;
        let mut expect = GxlProps::default();
        expect.append(GxlVar::new("X", "${PRJ_ROOT}/test/main.py"));
        expect.append(GxlVar::new("Y", "${PRJ_ROOT}/test/main.py"));
        assert_eq!(var, expect);
        assert_eq!(data, "");
        Ok(())
    }
    #[test]
    fn call_test() {
        let mut data = r#"
             os.path ( dist: "./tests/" , keep: "ture" ) ;"#;
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
}
