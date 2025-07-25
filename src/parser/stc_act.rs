use super::{inner::sentence_body, prelude::*};
use winnow::{combinator::fail, Parser, Result};

use crate::{
    components::gxl_act::activity::{Activity, ActivityDTO},
    primitive::GxlObject,
    sec::SecValueType,
    types::*,
};

use super::{domain::parse_log, stc_base::gal_act_head};

pub fn gal_activity(input: &mut &str) -> Result<Activity> {
    let name = gal_act_head("activity", input)?;

    let props = sentence_body
        .context(wn_desc("<activity-end>"))
        .parse_next(input)?;
    let mut dto = ActivityDTO {
        name,
        ..Default::default()
    };
    for one in props {
        let key = one.0.to_lowercase();
        if let GxlObject::Value(SecValueType::String(s_v)) = one.1 {
            let p_v = s_v.value().clone();
            if one.0 == "log" {
                dto.expect.log_lev = Some(parse_log((one.0.as_str(), p_v.as_str())));
                continue;
            } else if key == "executer" {
                dto.executer = p_v;
                continue;
            } else if key == "default_param" {
                dto.default_param = Some(p_v);
                continue;
            } else if key == "sudo" && p_v.to_lowercase() == "true" {
                dto.expect.sudo = true;
                continue;
            } else if key == "silence" && p_v.to_lowercase() == "true" {
                dto.expect.secrecy = true;
                continue;
            } else if key == "out" && p_v.to_lowercase() == "true" {
                dto.expect.quiet = true;
                continue;
            } else if key == "suc" {
                dto.expect.suc = Some(p_v);
                continue;
            } else if key == "err" {
                dto.expect.err = Some(p_v);
                continue;
            } else {
                dto.append_prop(Property {
                    key: one.0,
                    val: p_v,
                });
            }
        } else {
            return fail
                .context(wn_desc("<activity>(Prop Not String Value)"))
                .parse_next(input);
        }
    }
    if !dto.check() {
        return fail.context(wn_desc("<activity-check>")).parse_next(input);
    }
    let obj = Activity::dto_new(dto);
    Ok(obj)
}
#[cfg(test)]
mod tests {

    use orion_error::TestAssert;

    use crate::parser::inner::run_gxl;

    use super::*;
    #[test]
    fn activity_test() {
        let mut data = r#"
        activity copy {
            src = "" ;
            dst = "" ;
            log = "1";
            default_param = "src";
            executer = "copy_act.sh" ;
        }
  "#;
        let obj = run_gxl(gal_activity, &mut data).assert();
        let mut dto = ActivityDTO {
            name: "copy".to_string(),
            executer: "copy_act.sh".to_string(),
            default_param: Some("src".into()),
            ..Default::default()
        };
        dto.expect.log_lev = Some(log::Level::Info);
        dto.append_prop(Property {
            key: "src".into(),
            val: "".into(),
        });
        dto.append_prop(Property {
            key: "dst".into(),
            val: "".into(),
        });
        assert_eq!(data.trim(), "");
        assert_eq!(obj, Activity::dto_new(dto));
    }
}
