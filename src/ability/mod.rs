pub mod assert;
pub mod cmd;
pub mod delegate;
pub mod echo;
pub mod err_conv;
pub mod prelude;
pub mod read;
pub mod tpl;
//pub mod vault;
pub mod version;

use crate::{
    context::ExecContext, infra::once_init_log, traits::Setter, var::VarsDict, ExecResult,
};

pub struct StubFlowAbi {}

#[allow(dead_code)]
pub fn ability_env_init() -> (ExecContext, VarsDict) {
    once_init_log();
    let context = ExecContext::new(false);
    let mut def = VarsDict::default();
    def.set("RG_PRJ_ROOT", context.cur_path().as_str());
    (context, def)
}

pub fn sudo_cmd(sudo: bool) -> String {
    if sudo {
        "sudo".to_string()
    } else {
        "".to_string()
    }
}

pub fn parse_suc_code(suc: &str) -> Vec<i32> {
    let expect: Vec<&str> = suc.split(',').collect();
    let mut expect_vec = Vec::new();
    for i in expect {
        let i = i.trim();
        let mut val = 0;
        if !i.is_empty() {
            val = i
                .parse::<i32>()
                .unwrap_or_else(|_| panic!("bad number{}", i));
        }
        expect_vec.push(val);
    }
    expect_vec
}

pub use crate::ability::{
    assert::GxAssert,
    cmd::GxCmd,
    echo::GxEcho,
    read::{GxRead, RgReadDto},
    tpl::GxTpl,
    tpl::RgTplDto,
    tpl::RgTplDtoBuilder,
    version::{RgVersion, RgVersionBuilder},
};
use crate::evaluator::EnvExpress;

pub trait EnvRender {
    fn render(&mut self, exp: &EnvExpress) -> ExecResult<()>;
}
