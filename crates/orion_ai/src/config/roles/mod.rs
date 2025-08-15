pub mod loader;
pub mod manager;
pub mod types;

// 重新导出主要的公共接口
pub use loader::RoleConfigLoader;
pub use manager::RoleConfigManager;
pub use types::{RoleConfig, RulesConfig};
