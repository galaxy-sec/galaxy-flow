pub mod recorder;

// 重新导出Thread相关类型和组件
pub use crate::config::ThreadConfig;
pub use recorder::{SummaryExtractor, ThreadClient, ThreadFileManager};
