use orion_common::friendly::MultiNew2;
use orion_parse::atom::peek_line;

use super::prelude::*;

use crate::{
    components::{
        gxl_env::env::anns_from_option_dto,
        gxl_mod::{meta::ModMeta, ModItem},
        gxl_var::GxlProp,
        GxlMod,
    },
    meta::GxlType,
    parser::stc_flow::body::gal_stc_flow_body,
};

use super::{
    domain::{gal_block_beg, gal_block_end},
    inner::gal_prop,
    stc_act::gal_activity,
    stc_base::{gal_ann, gal_mod_head},
    stc_env::gal_stc_env_body,
};
pub fn gal_stc_mod_item(input: &mut &str) -> ModalResult<ModItem> {
    skip_spaces_block.parse_next(input)?;
    let ann = opt(gal_ann).parse_next(input)?;
    skip_spaces_block.parse_next(input)?;
    if starts_with("env", input) {
        let mut env = gal_stc_env_body
            .context(wn_desc("<env>"))
            .parse_next(input)?;
        env.set_anns(ann);
        return Ok(ModItem::Env(env));
    }
    if starts_with("flow", input) {
        let mut flow = gal_stc_flow_body
            .context(wn_desc("<flow>"))
            .parse_next(input)?;
        flow.set_anns(ann);
        return Ok(ModItem::Flow(flow));
    }
    if starts_with("activity", input) {
        return gal_activity
            .context(wn_desc("<activity>"))
            .map(ModItem::Actv)
            .parse_next(input);
    }
    error!(target:"parse", "mod not support: {}", peek_line(input));
    fail.context(wn_desc("mod not support")).parse_next(input)
}

pub fn gal_stc_mod(input: &mut &str) -> ModalResult<GxlMod> {
    skip_spaces_block(input)?;
    let ann_dto = opt(gal_ann).parse_next(input)?;
    let anns = anns_from_option_dto(ann_dto);
    let head = gal_mod_head
        .context(wn_desc("<flow-head>"))
        .parse_next(input)?;
    let mut meta = ModMeta::new2(GxlType::Mod, head.name().clone()).with_annotates(anns);
    meta.set_mix(head.mix().clone());
    let mut obj = GxlMod::from(meta);
    gal_block_beg.parse_next(input)?;
    let props: Vec<GxlProp> = repeat(0.., gal_prop).parse_next(input)?;
    obj.append(props);
    loop {
        skip_spaces_block.parse_next(input)?;
        if starts_with((multispace0, alt(("activity", "env", "flow", "#["))), input) {
            let item = gal_stc_mod_item.parse_next(input)?;
            obj.append(item);
        } else {
            break;
        }
    }
    gal_block_end
        .context(wn_desc("<mod-end>"))
        .parse_next(input)?;
    let _ = opt(symbol_semicolon).parse_next(input)?;
    Ok(obj)
}
#[cfg(test)]
mod tests {

    use orion_error::TestAssert;

    use crate::parser::inner::run_gxl;

    use super::*;

    #[test]
    fn test_mod1() {
        let mut data = r#"
mod main{
  root = "xxx" ;
  name = "xxx" ;
  flow  api {
    gx.echo ( value : "${PRJ_ROOT}", ) ;
  } ;
  flow admin {
    gx.echo ( value : "${PRJ_ROOT}", ) ;
  } ;
  flow test : admin,api  ;
};
"#;
        let rgmod = run_gxl(gal_stc_mod, &mut data).assert();
        assert_eq!(data, "\n");
        assert_eq!(rgmod.props().len(), 2);
    }

    #[test]
    fn test_mod2() {
        let mut data = r#"
mod main : mod_a {
  root = "xxx" ;
  name = "xxx" ;
  flow  api {
    gx.echo ( value : "${PRJ_ROOT}", ) ;
  } ;
  flow admin {
    gx.echo ( value : "${PRJ_ROOT}", ) ;
  } ;
  flow test : admin,api  ;
};
"#;
        let rgmod = run_gxl(gal_stc_mod, &mut data).assert();
        assert_eq!(data, "\n");
        assert_eq!(rgmod.props().len(), 2);
    }
}
