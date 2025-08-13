mod capabilities;
pub mod client;
pub mod config;
pub mod context;
mod error;
pub mod factory;
pub mod provider;
pub mod providers;
pub mod router;
pub mod thread;

// 重新导出主要类型和trait
pub use capabilities::*;
pub use config::*;
pub use context::*;
pub use error::*;
pub use factory::AiClientEnum;
pub use router::*;

// 客户端相关导出
pub use client::{AiClient, AiClientTrait, AiSendClient};
pub use thread::recorder::{ThreadClient, ThreadFileManager};

// Thread相关导出
pub use thread::ThreadConfig;
// mod tests; // 已移除到 config 模块中

// DeepSeek 测试模块
