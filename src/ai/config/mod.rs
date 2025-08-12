pub mod loader;
pub mod structures;
pub mod traits;

pub use traits::*;

#[cfg(test)]
pub mod tests;
// 重新导出主要的类型和函数，保持向后兼容
pub use self::loader::ConfigLoader;
pub use self::structures::{AiConfig, FileConfig, ProviderConfig, RoutingRules, UsageLimits};
