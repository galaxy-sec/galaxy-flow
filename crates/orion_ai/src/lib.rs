pub mod client;
pub mod config;
mod const_val;
mod error;
pub mod factory;
pub mod func;
mod infra;
pub mod provider;
pub mod providers;
mod roleid;
pub mod router;
pub mod thread;
// 重新导出主要类型和trait
pub use config::*;
pub use factory::AiClientEnum;
pub use roleid::*;
pub use router::*;

// Function calling 相关导出
pub use func::global::GlobalFunctionRegistry;
pub use func::{executor::FunctionExecutor, registry::FunctionRegistry};
pub use provider::{FunctionCall, FunctionDefinition, FunctionParameter, FunctionResult};

// 客户端相关导出
pub use client::{AiClient, AiClientTrait, AiCoreClient};
pub use error::{AiErrReason, AiError, AiResult};
pub use thread::recorder::{ThreadClient, ThreadFileManager};
pub use thread::ThreadConfig;
