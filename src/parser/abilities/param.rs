use super::{
    define::{gal_full_value, gal_gxl_object},
    prelude::*,
};
use orion_parse::symbol::{symbol_assign, symbol_colon};

use crate::primitive::{GxlAParam, GxlFParam};

pub fn gal_formal_param(input: &mut &str) -> Result<GxlFParam> {
    let _ = multispace0.parse_next(input)?;
    let default_name = opt("*").parse_next(input)?;
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_']))
        .context(wn_desc("<var-name>"))
        .parse_next(input)?;
    let _ = multispace0.parse_next(input)?;
    let val = if starts_with("=", input) {
        symbol_assign.parse_next(input)?;
        let _ = multispace0.parse_next(input)?;
        let val = Some(
            gal_full_value
                .context(wn_desc("<var-val>"))
                .parse_next(input)?,
        );
        let _ = multispace0.parse_next(input)?;
        val
    } else {
        None
    };
    Ok(GxlFParam::new(key.to_string())
        .with_default_value(val)
        .with_default_name(default_name.is_some()))
}
/*
pub fn gal_actual_param(input: &mut &str) -> Result<GxlAParam> {
    let _ = multispace0.parse_next(input)?;

    let key_opt = opt(
        take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.']))
            .context(wn_desc("<var-name>")),
    )
    .parse_next(input)?;
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_']))
        .context(wn_desc("<var-name>"))
        .parse_next(input)?;
    symbol_assign.parse_next(input)?;
    let _ = multispace0.parse_next(input)?;

    let val = gal_gxl_object
        .context(wn_desc("<var-obj>"))
        .parse_next(input)?;
    multispace0(input)?;
    //(multispace0, alt((symbol_comma, symbol_semicolon))).parse_next(input)?;
    Ok(GxlAParam::new(key.to_string(), val))
}
*/

pub fn gal_actual_param(input: &mut &str) -> Result<GxlAParam> {
    let _ = multispace0.parse_next(input)?;
    let key_opt = opt(
        take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.']))
            .context(wn_desc("<var-name>")),
    )
    .parse_next(input)?;
    if key_opt.is_some() {
        symbol_colon.parse_next(input)?;
    }
    let _ = multispace0.parse_next(input)?;
    let val = gal_gxl_object
        .context(wn_desc("<var-obj>"))
        .parse_next(input)?;
    multispace0(input)?;
    Ok(GxlAParam::new(key_opt.unwrap_or("default"), val))
}

#[cfg(test)]
mod tests {

    use crate::{
        parser::inner::run_gxl,
        sec::{SecFrom, SecValueType},
    };

    use super::*;

    #[test]
    fn test_formal_param_without_default() -> Result<()> {
        // 测试无默认值的简单参数
        let mut input = "paramName";
        let result = run_gxl(gal_formal_param, &mut input)?;

        assert_eq!(result.name(), "paramName");
        assert_eq!(result.default_value(), &None);
        assert_eq!(*result.default_name(), false);
        Ok(())
    }

    #[test]
    fn test_formal_param_with_string_default() -> Result<()> {
        // 测试带字符串默认值的参数
        let mut input = "path = \"/usr/local/bin\"";
        let result = run_gxl(gal_formal_param, &mut input)?;

        assert_eq!(result.name(), "path");
        assert_eq!(
            result.default_value().as_ref().unwrap(),
            &SecValueType::nor_from("/usr/local/bin".to_string())
        );
        assert_eq!(*result.default_name(), false);
        Ok(())
    }

    #[test]
    fn test_formal_param_with_number_default() -> Result<()> {
        // 测试带数字默认值的参数
        let mut input = "count = 42";
        let result = run_gxl(gal_formal_param, &mut input)?;

        assert_eq!(result.name(), "count");
        assert_eq!(
            result.default_value().as_ref().unwrap(),
            &SecValueType::nor_from(42)
        );
        assert_eq!(*result.default_name(), false);
        Ok(())
    }

    #[test]
    fn test_formal_param_with_bool_default() -> Result<()> {
        // 测试带布尔值默认值的参数
        let mut input = "*enabled = true";
        let result = run_gxl(gal_formal_param, &mut input)?;

        assert_eq!(result.name(), "enabled");
        assert_eq!(
            result.default_value().as_ref().unwrap(),
            &SecValueType::nor_from(true)
        );
        assert!(*result.default_name());
        Ok(())
    }
}
