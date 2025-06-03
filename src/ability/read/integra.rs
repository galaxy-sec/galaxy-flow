use async_trait::async_trait;
use derive_more::From;

use crate::{
    ability::prelude::{AsyncRunnableTrait, ComponentMeta, VTResult, VarSpace},
    context::ExecContext,
    meta::GxlMeta,
    ExecReason,
};

use super::{cmd::CmdDTO, file::FileDTO, stdin::StdinDTO};

#[derive(Debug, Default, Builder, PartialEq, Clone, Getters, From)]
pub struct GxRead {
    imp: ReadMode,
}
#[derive(Debug, PartialEq, Clone, Default, From)]
pub enum ReadMode {
    #[default]
    UNDEF,
    CMD(CmdDTO),
    FILE(FileDTO),
    STDIN(StdinDTO),
}

#[async_trait]
impl AsyncRunnableTrait for GxRead {
    async fn async_exec(&self, ctx: ExecContext, vars_dict: VarSpace) -> VTResult {
        self.execute_impl(ctx, vars_dict)
    }
}

impl ComponentMeta for GxRead {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::build_ability("gx.read")
    }
}
impl GxRead {
    fn execute_impl(&self, ctx: ExecContext, dict: VarSpace) -> VTResult {
        match &self.imp {
            ReadMode::CMD(cmd_dto) => cmd_dto.execute(ctx, dict),
            ReadMode::FILE(ini_dto) => ini_dto.execute(ctx, dict),
            ReadMode::STDIN(stdin_dto) => stdin_dto.execute(ctx, dict),
            _ => Err(ExecReason::Exp(String::from("not implementation")).into()),
        }
    }
}
