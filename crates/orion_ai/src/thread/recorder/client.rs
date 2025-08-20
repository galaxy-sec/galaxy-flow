use chrono::Utc;
use std::sync::Arc;

use super::ThreadFileManager;
use crate::client::{AiClientTrait, AiCoreClient};
use crate::config::ThreadConfig;
use crate::error::AiResult;
use crate::provider::{AiRequest, AiResponse};
use crate::roleid::AiRoleID;

/// Thread记录客户端 - 嵌套式静态分发
pub struct ThreadClient {
    inner: AiCoreClient,       // 内部是AiClientEnum
    config: Arc<ThreadConfig>, // Thread配置
    file_manager: Arc<ThreadFileManager>,
}

impl ThreadClient {
    /// 创建新的Thread记录客户端
    pub fn new(inner: AiCoreClient, config: ThreadConfig) -> Self {
        let config_arc = Arc::new(config);
        Self {
            inner,
            config: config_arc.clone(),
            file_manager: Arc::new(ThreadFileManager::new((*config_arc).clone())),
        }
    }

    /// 检查是否启用Thread记录
    fn is_thread_enabled(&self) -> bool {
        self.config.enabled
    }

    /// 构建带有Thread通知的请求
    fn build_request_with_thread_info(&self, mut request: AiRequest) -> AiRequest {
        if self.config.inform_ai {
            // 在系统提示中添加Thread记录通知
            request.system_prompt = format!(
                "{}\n\n{}",
                request.system_prompt, self.config.inform_message
            );
        }
        request
    }

    /// 发送AI请求
    pub async fn send_request(&self, request: AiRequest) -> AiResult<AiResponse> {
        let start_time = Utc::now();

        // 如果需要通知AI，构建增强的请求
        let enhanced_request = if self.config.inform_ai {
            self.build_request_with_thread_info(request.clone())
        } else {
            request.clone()
        };
        let response = self.inner.send_request(enhanced_request).await;

        // 如果启用Thread记录且响应成功，则记录交互
        if self.is_thread_enabled() {
            if let Ok(ref resp) = response {
                if let Err(e) = self
                    .file_manager
                    .record_interaction(start_time, &request, resp)
                    .await
                {
                    eprintln!("Warning: Failed to record thread interaction: {e}");
                }
            }
        }

        response
    }

    /// 基于角色的智能请求处理
    pub async fn smart_role_request(
        &self,
        _role: &AiRoleID,
        _user_input: &str,
    ) -> AiResult<AiResponse> {
        todo!();
    }
}
