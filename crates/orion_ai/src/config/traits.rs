use crate::provider::AiProviderType;

use super::structures::AiConfig;

impl AiConfig {}

/// 配置相关的扩展 trait
pub trait ConfigExt {
    /// 检查配置是否有效
    fn is_valid(&self) -> bool;

    /// 获取所有启用的提供商
    fn enabled_providers(&self) -> Vec<AiProviderType>;

    /// 检查是否有足够的预算用于分析
    fn has_analysis_budget(&self, tokens: usize) -> bool;

    /// 检查是否有足够的预算用于审查
    fn has_review_budget(&self, tokens: usize) -> bool;
}

impl ConfigExt for AiConfig {
    fn is_valid(&self) -> bool {
        // 基本的验证逻辑
        !self.providers.is_empty()
            && !self.routing.simple.is_empty()
            && !self.routing.complex.is_empty()
            && !self.routing.free.is_empty()
            && self.limits.review_budget > 0
            && self.limits.analysis_budget > 0
    }

    fn enabled_providers(&self) -> Vec<AiProviderType> {
        self.providers
            .iter()
            .filter(|(_, config)| config.enabled)
            .map(|(provider, _)| *provider)
            .collect()
    }

    fn has_analysis_budget(&self, tokens: usize) -> bool {
        tokens <= self.limits.analysis_budget
    }

    fn has_review_budget(&self, tokens: usize) -> bool {
        tokens <= self.limits.review_budget
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config = AiConfig::example();

        assert!(config.is_valid());

        // 测试无效配置
        let mut invalid_config = config;
        invalid_config.routing.simple = "".to_string();
        assert!(!invalid_config.is_valid());
    }

    #[test]
    fn test_enabled_providers() {
        let config = AiConfig::example();

        let enabled = config.enabled_providers();
        assert!(enabled.contains(&AiProviderType::OpenAi));
        assert!(enabled.contains(&AiProviderType::DeepSeek));
        assert!(enabled.contains(&AiProviderType::Glm));
        assert!(enabled.contains(&AiProviderType::Kimi));
    }

    #[test]
    fn test_budget_checking() {
        let config = AiConfig::example();

        assert!(config.has_analysis_budget(1000));
        assert!(!config.has_analysis_budget(5000));

        assert!(config.has_review_budget(1000));
        assert!(!config.has_review_budget(3000));
    }
}
