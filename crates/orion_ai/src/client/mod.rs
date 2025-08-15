pub mod builder;
pub mod core;
pub mod trais;
pub mod utils;

#[cfg(test)]
pub mod tests;

// 重新导出主要类型和trait
pub use builder::AiClientBuilder;
pub use core::AiClient;
pub use trais::{AiClientTrait, AiCoreClient};
pub use utils::load_key_dict;
