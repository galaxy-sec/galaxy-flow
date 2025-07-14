use super::super::prelude::*;
use super::common::action_call_args;

use crate::{
    calculate::defined::{FnDefined, FnDefinedBuilder},
    parser::domain::gal_keyword,
};

fn gal_defined(input: &mut &str) -> Result<FnDefined> {
    let mut builder = FnDefinedBuilder::default();
    gal_keyword("defined", input)?;
    let props = action_call_args.parse_next(input)?;
    for one in props {
        let key = one.0.to_lowercase();
        if key == "default" || key == "var" {
            builder.name(one.1);
        }
    }

    match builder.build() {
        Ok(obj) => Ok(obj),
        Err(e) => {
            error!("{e}");
            fail.context(wn_desc("defined")).parse_next(input)
        }
    }
}

#[cfg(test)]
mod tests {

    use orion_error::TestAssert;

    use crate::infra::once_init_log;

    use super::*;

    #[test]
    fn cmd_test() {
        once_init_log();
        let mut data = r#"
             defined(${HOME}) ;"#;
        let obj = gal_defined(&mut data).assert();
        assert_eq!(obj.name(), "${HOME}");
    }
}
