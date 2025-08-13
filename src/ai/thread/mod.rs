pub mod recorder;

pub use recorder::ThreadRecordingClient;

// 重新导出Thread相关类型
pub use crate::ai::config::ThreadConfig;
