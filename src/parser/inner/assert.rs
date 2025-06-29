use super::super::prelude::*;
use super::common::sentence_call_args;
use winnow::combinator::fail;

use crate::ability::assert::*;
use crate::parser::domain::gal_keyword_alt;

pub fn gal_assert(input: &mut &str) -> Result<GxAssert> {
    let mut builder = GxAssertBuilder::default();
    gal_keyword_alt("gx.assert", "rg.assert", input)?;
    let props = sentence_call_args.parse_next(input)?;
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn assert_test() {
        let mut data = r#"
             gx.assert ( value : "hello" , expect : "hello" , err : "errinfo",) ;"#;
        let found = gal_assert(&mut data).unwrap();
        let mut expect = GxAssert::from_diy_error("errinfo");
        expect.expect_eq("hello", "hello");
        //expect.err() = Some(format!("errinfo"));
        assert_eq!(found, expect);
        assert_eq!(data, "");
    }
}
