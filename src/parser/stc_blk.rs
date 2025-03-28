use super::prelude::*;
use orion_parse::atom::take_env_var;
use orion_parse::symbol::symbol_cmp;
use winnow::combinator::repeat;

use crate::calculate::cond::IFExpress;
use crate::calculate::express::{BinExpress, EVarDef, ExpressEnum};
use crate::components::gxl_block::{BlockAction, BlockNode};
use crate::components::gxl_cond::RgCond;

use super::atom::spaced;
use super::domain::{gal_block_beg, gal_block_end, gal_keyword};
use super::inner::{gal_assert, gal_call, gal_cmd, gal_echo, gal_read, gal_tpl, gal_version};

pub fn gal_block(input: &mut &str) -> ModalResult<BlockNode> {
    let mut block = BlockNode::new();
    gal_block_beg
        .context(wn_desc("<block-beg>"))
        .parse_next(input)?;
    let sentens: Vec<BlockAction> = repeat(1.., gal_sentens_item)
        .context(wn_desc("<sentens>"))
        .parse_next(input)?;
    gal_block_end
        .context(wn_desc("<block-end>"))
        .parse_next(input)?;
    block.append(sentens);
    Ok(block)
}

pub fn gal_block_code(input: &mut &str) -> ModalResult<BlockNode> {
    let mut block = BlockNode::new();
    loop {
        skip_spaces_block.parse_next(input)?;
        if starts_with((multispace0, "}"), input) {
            return Ok(block);
        }

        let senten = gal_sentens_item
            .context(wn_desc("<senten>"))
            .parse_next(input)?;
        block.append(senten);
    }
    //Ok(block)
}

pub fn gal_sentens_item(input: &mut &str) -> ModalResult<BlockAction> {
    multispace0(input)?;
    if starts_with("if", input) {
        return gal_cond.map(BlockAction::Cond).parse_next(input);
    }
    if starts_with("gx.cmd", input) {
        return gal_cmd.map(BlockAction::Command).parse_next(input);
    }
    if starts_with("gx.echo", input) {
        return gal_echo.map(BlockAction::Echo).parse_next(input);
    }
    if starts_with("gx.assert", input) {
        return gal_assert.map(BlockAction::Assert).parse_next(input);
    }
    if starts_with("gx.ver", input) {
        return gal_version.map(BlockAction::Version).parse_next(input);
    }
    if starts_with("gx.read", input) {
        return gal_read.map(BlockAction::Read).parse_next(input);
    }
    if starts_with("gx.tpl", input) {
        return gal_tpl.map(BlockAction::Tpl).parse_next(input);
    }
    /*
    if starts_with("gx.vault", input) {
        return gal_vault.map(BlockAction::Vault).parse_next(input);
    }
    */

    gal_call
        .context(wn_desc("<flow-call>"))
        .map(BlockAction::Delegate)
        .parse_next(input)
    //gal_prop.map(make_run_hold).parse_next(input)?;
    //error!(target : "gxl", "not support gxl :{}" , peek_line(input));
    //fail.context(wn_desc("not support gxl sentens"))
    //    .parse_next(input)
}

pub fn gal_cond(input: &mut &str) -> ModalResult<RgCond> {
    //if val == 1
    skip_spaces_block(input)?;
    gal_keyword("if", input)?;

    let (name, cmp, value) = (
        spaced(take_env_var).context(wn_desc("<env-var>")),
        spaced(symbol_cmp).context(wn_desc("operator")),
        spaced(take_string).context(wn_desc("<value-str>")),
    )
        .parse_next(input)?;
    let true_block = gal_block.parse_next(input)?;
    skip_spaces_block(input)?;
    multispace0(input)?;
    let false_block = if starts_with("else", input) {
        gal_keyword("else", input)?;
        gal_block.parse_next(input)?
    } else {
        BlockNode::new()
    };
    let ctrl_express = IFExpress::new(
        ExpressEnum::EStr(BinExpress::from_op(cmp, EVarDef::new(name), value)),
        true_block,
        false_block,
    );
    Ok(RgCond::new(ctrl_express))
}

#[cfg(test)]
mod tests {

    use orion_error::TestAssert;

    use crate::parser::{
        inner::run_gxl,
        stc_blk::{gal_block, gal_cond},
    };

    #[test]
    fn test_block_1() {
        let mut data = r#"
        {
             gx.echo { value  = "${PRJ_ROOT}/test/main.py" ; };
             gx.echo { value  = "${PRJ_ROOT}/test/main.py" ; };

             gx.assert { value = "hello" ; expect = "hello" ; err = "errinfo";} ;
        }"#;
        let blk = run_gxl(gal_block, &mut data).assert();
        assert_eq!(blk.items().len(), 3);
        assert_eq!(data, "");
    }

    #[test]
    fn test_block_if() {
        let mut data = r#"
            if ${val} == "1" {
                gx.echo { value  = "${PRJ_ROOT}/test/main.py" ; };
             }"#;
        let _ = run_gxl(gal_cond, &mut data).assert();
        //assert_eq!(blk.items().len(), 1);
        assert_eq!(data, "");

        let mut data = r#"
            if ${val} == "1" {
                gx.echo { value  = "${PRJ_ROOT}/test/main.py" ; };
             }
             else {
                gx.echo { value  = "${PRJ_ROOT}/test/main.py" ; };
             }"#;
        let _ = run_gxl(gal_cond, &mut data).assert();
        //assert_eq!(blk.items().len(), 1);
        assert_eq!(data, "");
    }
    #[test]
    fn test_block_3() {
        let mut data = r#"
        {
            if ${val} == "1" {
                gx.echo { value  = "${PRJ_ROOT}/test/main.py" ; };
             }
             else {
                gx.echo { value  = "${PRJ_ROOT}/test/main.py" ; };
             }
        }"#;
        let _ = run_gxl(gal_block, &mut data).assert();
        assert_eq!(data, "");
    }
}
