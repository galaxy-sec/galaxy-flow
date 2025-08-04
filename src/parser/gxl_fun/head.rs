use orion_parse::atom::take_var_name;
use orion_parse::symbol::symbol_comma;
use winnow::ascii::multispace0;
use winnow::combinator::separated;
use winnow::Parser;
use winnow::Result;

use crate::components::gxl_fun::meta::FunMeta;
use crate::parser::abilities::param::gal_formal_param;
use crate::parser::atom::spaced_desc;
use crate::parser::domain::gal_call_beg;
use crate::parser::domain::gal_call_end;
use crate::primitive::GxlFParam;

pub fn gal_fun_head(input: &mut &str) -> Result<FunMeta> {
    spaced_desc("fn", "<keyword:fn>").parse_next(input)?;
    let fun_name = take_var_name.parse_next(input)?;
    multispace0.parse_next(input)?;
    let args = fun_define_params.parse_next(input)?;
    multispace0.parse_next(input)?;
    Ok(FunMeta::build_fun(fun_name).with_params(args))
}
pub fn fun_define_params(input: &mut &str) -> Result<Vec<GxlFParam>> {
    gal_call_beg.parse_next(input)?;
    let props: Vec<GxlFParam> = separated(0.., gal_formal_param, symbol_comma).parse_next(input)?;
    gal_call_end.parse_next(input)?;
    Ok(props)
}
