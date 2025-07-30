use orion_parse::atom::take_var_name;
use orion_parse::symbol::symbol_comma;
use orion_parse::symbol::wn_desc;
use winnow::ascii::multispace0;
use winnow::combinator::separated;
use winnow::token::take_while;
use winnow::Parser;
use winnow::Result;

use crate::components::gxl_fun::meta::FunMeta;
use crate::parser::atom::spaced_desc;
use crate::parser::domain::gal_call_beg;
use crate::parser::domain::gal_call_end;

pub fn gal_fun_head(input: &mut &str) -> Result<FunMeta> {
    spaced_desc("fun", "<keyword:fun>").parse_next(input)?;
    let fun_name = take_var_name.parse_next(input)?;
    multispace0.parse_next(input)?;
    let args = fun_define_args.parse_next(input)?;
    multispace0.parse_next(input)?;
    Ok(FunMeta::build_fun(fun_name).with_args(args))
}
pub fn fun_define_args(input: &mut &str) -> Result<Vec<String>> {
    gal_call_beg.parse_next(input)?;
    let props: Vec<String> = separated(0.., arg_def, symbol_comma).parse_next(input)?;
    gal_call_end.parse_next(input)?;
    Ok(props)
}

pub fn arg_def(input: &mut &str) -> Result<String> {
    let _ = multispace0.parse_next(input)?;
    let arg_name = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_']))
        .context(wn_desc("<arg-def>"))
        .parse_next(input)?;
    let _ = multispace0.parse_next(input)?;
    multispace0(input)?;
    Ok(arg_name.to_string())
}
