mod capabilities;
pub mod client;
pub mod config;
pub mod context;
pub mod error;
pub mod provider;
pub mod providers;
pub mod router;
pub mod thread;
pub use capabilities::*;
pub use client::AiClientEnum;
pub use config::*;
pub use context::*;
pub use error::*;
pub use router::*;
pub use thread::*;
// mod tests; // 已移除到 config 模块中

// DeepSeek 测试模块
