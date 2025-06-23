pub mod code_spc;
pub mod gxl_block;
pub(crate) mod gxl_cond;
pub mod gxl_env;
pub mod gxl_extend;
pub mod gxl_flow;
pub mod gxl_intercept;
pub mod gxl_loop;
pub mod gxl_mod;
pub mod gxl_spc;
pub mod gxl_utls;
pub mod gxl_var;
pub mod prelude;

pub use crate::model::components::{
    gxl_env::env::{GxlEnv, GxlEnvHold},
    gxl_flow::{flow::FlowHold, flow::GxlFlow},
    gxl_mod::{GxlMod, ModHold},
    gxl_var::{RgVars, VarsHold},
};

#[derive(Debug, Clone, Default, Getters)]
pub struct RgJson {}
