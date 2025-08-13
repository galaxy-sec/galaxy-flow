pub mod recorder;

// 重新导出Thread相关类型和组件
pub use crate::ai::config::ThreadConfig;
pub use recorder::{SummaryExtractor, ThreadClient, ThreadFileManager};
