//! 角色配置模块
//! 
//! 此模块提供了AI角色配置的管理功能，包括：
//! - 角色配置的加载和保存
//! - 规则配置的管理
//! - 分层配置系统（项目级 > 用户级）

pub mod types;
pub mod manager;
pub mod loader;

#[cfg(test)]
mod tests;

// 重新导出主要的公共接口
pub use types::{RoleConfig, RulesConfig};
pub use manager::RoleConfigManager;
pub use loader::RoleConfigLoader;