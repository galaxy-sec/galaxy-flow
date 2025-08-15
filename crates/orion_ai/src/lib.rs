mod capabilities;
pub mod client;
pub mod config;
mod const_val;
mod error;
pub mod factory;
mod infra;
pub mod provider;
pub mod providers;
pub mod router;
pub mod thread;

// 重新导出主要类型和trait
pub use capabilities::*;
pub use config::*;
pub use factory::AiClientEnum;
pub use router::*;

// 客户端相关导出
pub use client::{AiClient, AiClientTrait, AiCoreClient};
pub use error::{AiErrReason, AiError, AiResult};
pub use thread::recorder::{ThreadClient, ThreadFileManager};
// Thread相关导出
pub use thread::ThreadConfig;
// mod tests; // 已移除到 config 模块中

// DeepSeek 测试模块
