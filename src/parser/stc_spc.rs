use super::prelude::*;
use winnow::{
    ascii::multispace0,
    combinator::{alt, fail, opt},
    ModalResult, Parser,
};

use crate::components::code_spc::CodeSpace;

use super::stc_mod::gal_stc_mod;

pub fn gal_stc_spc(input: &mut &str) -> ModalResult<CodeSpace> {
    skip_spaces_block(input)?;
    let mut spc = CodeSpace::default();
    let mut items = Vec::new();
    loop {
        skip_spaces_block.parse_next(input)?;
        if starts_with(alt(((multispace0, "mod"), (multispace0, "#["))), input) {
            let item = gal_stc_mod.context(wn_desc("<mod>")).parse_next(input)?;
            items.push(item);
        } else {
            break;
        }
    }
    let _ = opt(symbol_semicolon).parse_next(input)?;
    info!(target: "parse","mod count: {}", items.len());
    skip_spaces_block(input)?;
    if !input.is_empty() {
        return fail.context(wn_desc("<space-end>")).parse_next(input);
    }

    spc.append(items);
    Ok(spc)
}
#[cfg(test)]
mod tests {

    use orion_error::TestAssert;

    use crate::parser::inner::run_gxl;

    use super::*;

    #[test]
    fn test_spc1() {
        let mut data = r#"
mod main{
  root = "xxx" ;
  name = "xxx" ;
  flow  api {
    gx.echo { value = "${PRJ_ROOT}"; } ;
  } ;
  flow admin {
    gx.echo { value = "${PRJ_ROOT}"; } ;
  } ;
  flow test : admin,api  ;
};
"#;
        let spc = run_gxl(gal_stc_spc, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(spc.mods().len(), 1);
    }

    #[test]
    fn test_spc2() {
        let mut data = r#"
mod envs {
    env dev {
        root = "HOME";
    }
}
mod main{
  root = "xxx" ;
  name = "xxx" ;
  flow  api {
    gx.echo { value = "${PRJ_ROOT}"; } ;
  } ;
  flow admin {
    gx.echo { value = "${PRJ_ROOT}"; } ;
  } ;
  flow test : admin,api  ;
};
"#;
        let spc = run_gxl(gal_stc_spc, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(spc.mods().len(), 2);
    }
}
