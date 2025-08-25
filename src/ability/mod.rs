pub mod ai;
pub mod ai_fun;
pub mod archive;
pub mod assert;
pub mod cmd;
pub mod delegate;
pub mod echo;
pub mod gxl;
pub mod load;
pub mod prelude;
pub mod read;
pub mod tpl;
//pub mod vault;
pub mod shell;
pub mod version;
use prelude::VarSpace;

use crate::const_val::gxl_const;
use crate::{context::ExecContext, infra::once_init_log, traits::Setter, ExecResult};

pub struct StubFlowAbi {}

#[allow(dead_code)]
pub fn ability_env_init() -> (ExecContext, VarSpace) {
    once_init_log();
    let context = ExecContext::new(Some(false), false);
    let mut def = VarSpace::default();
    def.global_mut()
        .set(gxl_const::PRJ_ROOT, context.cur_path().as_str());
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
            val = i.parse::<i32>().unwrap_or_else(|_| panic!("bad number{i}"));
        }
        expect_vec.push(val);
    }
    expect_vec
}

pub use crate::ability::{
    ai_fun::GxAIFun,
    assert::GxAssert,
    cmd::GxCmd,
    echo::GxEcho,
    read::GxRead,
    tpl::GxTpl,
    tpl::TplDTO,
    tpl::TplDTOBuilder,
    version::{GxlVersion, GxlVersionBuilder},
};
use crate::evaluator::EnvExpress;

pub trait EnvRender {
    fn render(&mut self, exp: &EnvExpress) -> ExecResult<()>;
}

pub use gxl::GxRun;
pub use gxl::GxRunBuilder;

pub use load::{GxDownLoad, GxDownLoadBuilder, GxUpLoad, GxUpLoadBuilder};
