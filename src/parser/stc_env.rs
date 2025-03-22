use super::prelude::*;
use orion_parse::{
    atom::{skip_spaces_block, starts_with},
    symbol::wn_desc,
};

use crate::components::{gxl_env::EnvItem, gxl_var::RgProp, GxlEnv};

use super::{
    domain::{gal_sentence_beg, gal_sentence_end},
    inner::{gal_prop, gal_read, gal_vars},
    stc_base::{gal_ann, gal_env_head},
};

pub fn gal_env_item(input: &mut &str) -> ModalResult<EnvItem> {
    multispace0(input)?;
    if starts_with("gx.vars", input) {
        return gal_vars.map(EnvItem::Var).parse_next(input);
    }
    if starts_with("gx.read", input) {
        return gal_read.map(EnvItem::Read).parse_next(input);
    }
    //if starts_with("gx.vault", input) {
    //    return gal_vault.map(EnvItem::Vault).parse_next(input);
    //}
    fail.context(wn_desc("not support env item"))
        .parse_next(input)
}

pub fn gal_stc_env(input: &mut &str) -> ModalResult<GxlEnv> {
    //let mut builder = RgAssertBuilder::default();
    skip_spaces_block(input)?;
    let ann = opt(gal_ann).parse_next(input)?;
    let mut env = gal_stc_env_body.parse_next(input)?;
    env.set_anns(ann);
    Ok(env)
}
pub fn gal_stc_env_body(input: &mut &str) -> ModalResult<GxlEnv> {
    let head = gal_env_head
        .context(wn_desc("<env-head>"))
        .parse_next(input)?;
    let mut obj = GxlEnv::from((head.name().clone(), head.mix().clone()));
    if starts_with((multispace0, ";"), input) {
        (multispace0, ";").parse_next(input)?;
        return Ok(obj);
    }
    gal_sentence_beg
        .context(wn_desc("<env-beg>"))
        .parse_next(input)?;
    let props: Vec<RgProp> = repeat(0.., gal_prop).parse_next(input)?;
    let env_items: Vec<EnvItem> = repeat(0.., gal_env_item).parse_next(input)?;
    gal_sentence_end
        .context(wn_desc("<env-end>"))
        .parse_next(input)?;

    for i in props {
        obj.append(i);
    }
    for i in env_items {
        obj.append(i);
    }
    Ok(obj)
}

#[cfg(test)]
mod tests {

    use orion_error::TestAssert;

    use crate::parser::inner::run_gxl;

    use super::*;
    #[test]
    fn env_test_vars() {
        let mut data = r#"
        env  base {
             gx.vars {
               x = "${PRJ_ROOT}/test/main.py" ;
               y = "${PRJ_ROOT}/test/main.py" ;
             };
        };"#;
        let _env = run_gxl(gal_stc_env, &mut data).assert();
        assert_eq!(data, "");
    }
    #[test]
    fn env_only_props() {
        let mut data = r#"
        env  base {
               x = "${PRJ_ROOT}/test/main.py" ;
               y = "${PRJ_ROOT}/test/main.py" ;
        };"#;
        let env = run_gxl(gal_stc_env, &mut data).assert();
        assert_eq!(env.props().len(), 2);
        assert_eq!(data, "");
    }
    #[test]
    fn env_empty() {
        let mut data = r#" env  base { };"#;
        let env = run_gxl(gal_stc_env, &mut data).assert();
        assert_eq!(env.props().len(), 0);
        assert_eq!(data, "");
    }
    #[test]
    fn env_mix() {
        let mut data = r#" env  dev : base ;"#;
        let env = run_gxl(gal_stc_env, &mut data).assert();
        assert_eq!(env.props().len(), 0);
        assert_eq!(data, "");
    }
    #[test]
    fn env_vars() {
        let mut data = r#" env  base : test_a,test_b {
             gx.vars {
               x = "${PRJ_ROOT}/test/main.py" ;
               y = "${PRJ_ROOT}/test/main.py" ;
             };
    };"#;
        let _env = run_gxl(gal_stc_env, &mut data).assert();
        assert_eq!(data, "");
    }
    #[test]
    fn env_read() {
        let mut data = r#"
            env branch_auto {
                gx.read {
                  name = "BRANCH_ENV";
                  cmd  = ^"git branch --show-current |  sed -E "s/(feature|develop|ver-dev|release|master|issue)(\/.*)?/_branch_\1/g" "^ ;
                  log  = "debug";
                  }
            }
            "#;
        let _env = run_gxl(gal_stc_env, &mut data).assert();
        assert_eq!(data, "");
    }
}
