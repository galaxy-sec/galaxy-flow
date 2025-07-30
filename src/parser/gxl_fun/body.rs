use crate::components::gxl_flow::meta::FlowMeta;
use crate::components::gxl_fun::fun::GxlFun;
use crate::parser::prelude::*;

use crate::components::GxlFlow;
use crate::parser::stc_ann::gal_ann;
use crate::parser::stc_blk::gal_block;

use super::head::gal_fun_head;

pub fn gal_fun_body(input: &mut &str) -> Result<GxlFun> {
    let meta = gal_fun_head
        .context(wn_desc("<flow-head>"))
        .parse_next(input)?;
    let mut obj = GxlFun::from(meta);
    multispace0.parse_next(input)?;
    if !starts_with(";", input) {
        let block = gal_block.parse_next(input)?;
        obj.append(block);
    }
    let _ = opt(symbol_semicolon).parse_next(input)?;
    Ok(obj)
}
pub fn gal_stc_fun(input: &mut &str) -> Result<GxlFun> {
    skip_spaces_block(input)?;
    //let ann = opt(gal_ann).parse_next(input)?;
    let fun = gal_fun_body.parse_next(input)?;
    //flow.set_anns(ann);
    Ok(fun)
}

#[cfg(test)]
mod tests {
    use orion_error::TestAssert;

    use crate::parser::{gxl_fun::body::gal_stc_fun, inner::run_gxl};

    #[test]
    fn flow_test0() {
        let mut data = r#"
    fun start () {
         A = "this is A";
         B = ${A} ;
         gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
    };"#;
        let fun = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(fun.meta().name(), "start");
    }
    #[test]
    fn flow_test1() {
        let mut data = r#"
    fun start( first, second) {
        gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
        gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
    };"#;
        let flow = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "start");
    }
    #[test]
    fn flow_test2() {
        let mut data = r#"
        fun start () {
             key = "value";
             gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
        };"#;
        let flow = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "start");
    }

    #[test]
    fn flow_test4() {
        let mut data = r#"
    fun start(first) {
         if ${VAL} == "1" {
            gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
         }
         else {
            gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
         }
    };"#;
        let flow = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "start");
    }

    #[test]
    fn flow_test6() {
        let mut data = r#"
        fun x() {
            conf.tpl (
              tpl : "${MAIN_CONF}/tpls/test.sh"  ,
              dst : "${MAIN_CONF}/options/test.sh" ,
              data : "hello" ,
            );
            os.copy (
                src  : "${MAIN_CONF}/options/nginx.conf",
                dst  : "${MAIN_CONF}/used/nginx_ex.conf",
            );
        }
            "#;

        let flow = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "x");
    }
}
