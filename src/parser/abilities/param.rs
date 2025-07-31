use super::{
    define::{gal_full_value, gal_gxl_object},
    prelude::*,
};
use orion_parse::{
    define::{gal_raw_str, take_bool, take_float, take_number, take_string, take_var_ref_name},
    symbol::{symbol_assign, symbol_colon, wn_desc},
    utils::peek_one,
};
use winnow::{
    combinator::{peek, separated},
    token::literal,
};

use crate::{
    primitive::{GxlAParam, GxlFParam, GxlObject},
    sec::{SecFrom, SecValueObj, SecValueType, SecValueVec},
    var::UniString,
};

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
pub fn gal_actual_param(input: &mut &str) -> Result<GxlAParam> {
    let _ = multispace0.parse_next(input)?;
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

#[cfg(test)]
mod tests {

    use orion_error::TestAssert;

    use crate::{parser::inner::run_gxl, sec::ToUniCase};

    use super::*;
}
