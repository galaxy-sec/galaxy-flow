pub mod ai_executor;
pub mod chat_executor;
pub mod tool;

// 向后兼容性导出
pub use ai_executor::AiExecutor as GxAIFun;
pub use chat_executor::ChatExecutor as GxAIChat;
