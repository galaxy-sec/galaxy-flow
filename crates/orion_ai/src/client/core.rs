use crate::config::RoleConfigManager;
use crate::error::{AiError, AiResult};
use crate::provider::{AiProvider, AiProviderType, AiRequest, AiResponse};
use crate::roleid::AiRoleID;
use crate::{AiConfig, AiErrReason, AiRouter};
use async_trait::async_trait;
use getset::Getters;
use log::error;
use orion_error::{ErrorWith, ToStructError, UvsConfFrom};
use std::collections::HashMap;
use std::sync::Arc;

use super::trais::AiClientTrait;

/// 主AI客户端，统一的API接口
#[derive(Getters)]
#[getset(get = "pub")]
pub struct AiClient {
    pub providers: HashMap<AiProviderType, Arc<dyn AiProvider>>,
    pub config: AiConfig,
    pub router: AiRouter,
    pub roles: RoleConfigManager,
}

#[async_trait]
impl AiClientTrait for AiClient {
    async fn send_request(&self, request: AiRequest) -> AiResult<AiResponse> {
        let provider_type = self.router.select_provider(&request.model, &self.config);

        if let Some(provider) = self.providers.get(&provider_type) {
            provider
                .send_request(&request)
                .await
                .with(format!("provide: {provider_type}"))
        } else {
            for key in self.providers().keys() {
                error!("registed provider: {key}");
            }
            Err(AiError::from(AiErrReason::NoProviderAvailable)).with(provider_type.to_string())
        }
    }

    /// 基于角色的智能请求处理 - 用户只需选择角色，系统自动选择推荐模型
    async fn smart_role_request(&self, role: &AiRoleID, user_input: &str) -> AiResult<AiResponse> {
        let request = self.build_ai_request(role, user_input)?;
        // 3. 发送请求
        let mut response = self.send_request(request).await?;

        // 4. 在响应中添加角色信息
        response.content = format!("[角色: {}]\n\n{}", role.description(), response.content);

        Ok(response)
    }
}

impl AiClient {
    /// 构建基于角色的系统提示
    fn build_role_system_prompt(&self, role: &AiRoleID) -> String {
        // 从配置文件中获取角色系统提示词
        if let Some(role_config) = self.roles.get_role_config(&role.to_string()) {
            let mut system_prompt = role_config.system_prompt().clone();

            // 尝试加载角色特定的规则配置
            if let Ok(Some(role_rules)) = self.roles.get_role_rules_config(&role.to_string()) {
                system_prompt = self.enhance_system_prompt_with_rules(system_prompt, &role_rules);
            }
            system_prompt
        } else {
            "".to_string()
        }
    }

    /// 使用规则增强系统提示词
    fn enhance_system_prompt_with_rules(
        &self,
        base_prompt: String,
        rules: &crate::config::roles::RulesConfig,
    ) -> String {
        let mut enhanced_prompt = base_prompt;

        // 添加规则集合
        if !rules.rules.is_empty() {
            enhanced_prompt.push_str("\n\n## 规则\n");
            for rule in &rules.rules {
                enhanced_prompt.push_str(&format!("- {rule}\n"));
            }
        }
        enhanced_prompt
    }

    /// 获取所有可用的provider
    pub fn available_providers(&self) -> Vec<AiProviderType> {
        self.providers.keys().copied().collect()
    }

    /// 检查特定provider是否可用
    pub fn is_provider_available(&self, provider: AiProviderType) -> bool {
        self.providers.contains_key(&provider)
    }

    pub fn build_ai_request(&self, role: &AiRoleID, user_input: &str) -> AiResult<AiRequest> {
        // 1. 使用角色推荐模型
        let conf = self
            .roles
            .get_role_config(role.as_str())
            .ok_or_else(|| AiErrReason::from_conf(format!("miss role:{role} conf")).to_err())?;

        let model = conf
            .used_model()
            .as_ref()
            .unwrap_or(self.roles.default_model());
        // 2. 构建系统提示词
        let system_prompt = self.build_role_system_prompt(role);
        Ok(AiRequest::builder()
            .model(model)
            .system_prompt(system_prompt)
            .user_prompt(user_input.to_string())
            .role(role.clone())
            .build())
    }

    /// 列出指定provider的所有可用模型
    pub async fn list_models(
        &self,
        provider: &AiProviderType,
    ) -> AiResult<Vec<crate::provider::ModelInfo>> {
        if let Some(provider_arc) = self.providers.get(provider) {
            provider_arc.list_models().await
        } else {
            Err(AiErrReason::from_conf(format!("Provider {provider} not available")).to_err())
        }
    }
}
