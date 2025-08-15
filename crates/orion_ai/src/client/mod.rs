pub mod core;
pub mod client;
pub mod builder;
pub mod utils;

#[cfg(test)]
pub mod tests;

// 重新导出主要类型和trait
pub use core::{AiCoreClient, AiClientTrait};
pub use client::AiClient;
pub use builder::AiClientBuilder;
pub use utils::load_key_dict;