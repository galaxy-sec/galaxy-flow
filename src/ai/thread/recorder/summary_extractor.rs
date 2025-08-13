/// 总结性内容提取器，负责从AI响应中提取总结性内容
pub struct SummaryExtractor {
    keywords: Vec<String>,
}

impl SummaryExtractor {
    /// 创建新的总结提取器
    pub fn new(keywords: &[String]) -> Self {
        Self {
            keywords: keywords.to_vec(),
        }
    }

    /// 提取总结性内容，带长度限制
    pub fn extract_with_length_limits(
        &self,
        content: &str,
        min_length: usize,
        max_length: usize,
    ) -> String {
        let paragraphs: Vec<&str> = content
            .split('\n')
            .filter(|p| !p.trim().is_empty())
            .collect();

        // 1. 寻找包含总结关键字的段落
        if let Some(summary_paragraph) = self.find_summary_paragraph(&paragraphs) {
            return self.truncate_to_length(summary_paragraph, min_length, max_length);
        }

        // 2. 如果没有找到总结段落，尝试合并最后几个段落
        let summary = self.merge_last_paragraphs(&paragraphs, min_length, max_length);
        if !summary.is_empty() {
            return summary;
        }

        // 3. 如果没有合适的段落，返回空字符串
        String::new()
    }

    /// 寻找包含总结关键字的段落
    fn find_summary_paragraph<'a>(&self, paragraphs: &'a [&str]) -> Option<&'a str> {
        for &paragraph in paragraphs {
            if self.contains_summary_keyword(paragraph) {
                return Some(paragraph);
            }
        }
        None
    }

    /// 检查文本是否包含总结关键字
    fn contains_summary_keyword(&self, text: &str) -> bool {
        let lower_text = text.to_lowercase();
        self.keywords
            .iter()
            .any(|keyword| lower_text.contains(&keyword.to_lowercase()))
    }

    /// 合并最后几个段落以满足字数要求
    fn merge_last_paragraphs(
        &self,
        paragraphs: &[&str],
        min_length: usize,
        max_length: usize,
    ) -> String {
        if paragraphs.is_empty() {
            return String::new();
        }

        let mut merged = String::new();
        let mut start_idx = paragraphs.len().saturating_sub(1);

        // 从最后一段开始，向前合并直到满足最小字数要求
        for i in (0..=start_idx).rev() {
            let temp_merged = if i == start_idx {
                paragraphs[i].to_string()
            } else {
                format!("{} {}", paragraphs[i], merged)
            };

            // 如果超过最大长度，停止合并
            if temp_merged.len() > max_length {
                break;
            }

            merged = temp_merged;
            start_idx = i;

            // 如果已经满足最小长度，停止合并
            if merged.len() >= min_length {
                break;
            }
        }

        // 截断到最大长度
        self.truncate_to_length(&merged, min_length, max_length)
    }

    /// 将文本截断到指定长度范围
    fn truncate_to_length(&self, text: &str, min_length: usize, max_length: usize) -> String {
        let text = text.trim();

        // 如果文本在范围内，直接返回
        if text.len() >= min_length && text.len() <= max_length {
            return text.to_string();
        }

        // 如果文本太短，直接返回
        if text.len() < min_length {
            return text.to_string();
        }

        // 如果文本太长，在句子边界处截断
        if let Some(truncated) = self.truncate_at_sentence_boundary(text, max_length) {
            truncated
        } else {
            // 如果找不到句子边界，使用安全的字符截断
            self.safe_char_truncate(text, max_length)
        }
    }

    /// 在句子边界处截断文本
    fn truncate_at_sentence_boundary(&self, text: &str, max_length: usize) -> Option<String> {
        if text.len() <= max_length {
            return Some(text.to_string());
        }

        // 使用安全的字符截断先获取不超过max_length的文本
        let truncated = self.safe_char_truncate(text, max_length);

        // 寻找最后一个句子结束符
        let sentence_endings = ['。', '？', '！', '.', '?', '!', ';', '；', '\n'];

        // 从后向前查找句子结束符
        for (i, c) in truncated.char_indices().rev() {
            if sentence_endings.contains(&c) {
                let end_index = i + c.len_utf8();
                if let Some(truncated_text) = truncated.get(..end_index) {
                    return Some(truncated_text.trim().to_string());
                }
            }
        }

        // 如果没有找到句子结束符，尝试在逗号或分号处截断
        let clause_endings = [',', '，', ';', '；'];
        for (i, c) in truncated.char_indices().rev() {
            if clause_endings.contains(&c) {
                let end_index = i + c.len_utf8();
                if let Some(truncated_text) = truncated.get(..end_index) {
                    return Some(truncated_text.trim().to_string());
                }
            }
        }

        // 如果还是找不到，返回None
        None
    }

    /// 安全的字符截断，确保不会在UTF-8字符中间截断
    fn safe_char_truncate(&self, text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            return text.to_string();
        }

        let mut truncated_end = 0;
        for (i, _) in text.char_indices() {
            if i > max_length {
                truncated_end = i;
                break;
            }
            truncated_end = i;
        }

        text.get(..truncated_end).unwrap_or(text).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_extractor() -> SummaryExtractor {
        let keywords = vec![
            "总结".to_string(),
            "总之".to_string(),
            "结论".to_string(),
            "summary".to_string(),
            "conclusion".to_string(),
        ];
        SummaryExtractor::new(&keywords)
    }

    #[test]
    fn test_extract_with_summary_keyword() {
        let extractor = create_extractor();
        let content = r#"这是一个复杂的分析过程。
需要考虑多个因素。
总结：这个方案的优点是效率高，缺点是成本较大。"#;

        let result = extractor.extract_with_length_limits(content, 10, 50);
        assert!(result.contains("总结："));
        assert!(result.contains("效率高"));
    }

    #[test]
    fn test_extract_without_summary_keyword() {
        let extractor = create_extractor();
        let content = r#"第一步是分析需求。
第二步是设计方案。
第三步是实施部署。
最后，需要进行测试验证。"#;

        let result = extractor.extract_with_length_limits(content, 10, 50);
        assert!(result.contains("最后"));
        assert!(result.contains("测试验证"));
    }

    #[test]
    fn test_truncate_to_length() {
        let extractor = create_extractor();
        let long_text = "这是一个很长的句子，包含很多内容，需要被截断到合适的长度。这是一个很长的句子，包含很多内容，需要被截断到合适的长度。";

        let result = extractor.truncate_to_length(long_text, 10, 30);
        assert!(result.len() <= 30);
    }

    #[test]
    fn test_merge_paragraphs() {
        let extractor = create_extractor();
        let paragraphs = vec![
            "第一段：简短描述。",
            "第二段：相对较长的描述内容。",
            "第三段：最后的总结性内容。",
        ];

        let result = extractor.merge_last_paragraphs(&paragraphs, 20, 100);
        assert!(result.contains("第三段"));
        // 可能包含第二段，取决于字数要求
    }

    #[test]
    fn test_english_keywords() {
        let extractor = create_extractor();
        let content = r#"This is a detailed analysis.
Multiple factors need to be considered.
In summary, this approach is efficient but costly."#;

        let result = extractor.extract_with_length_limits(content, 10, 50);
        assert!(result.contains("In summary"));
        assert!(result.contains("efficient"));
    }

    #[test]
    fn test_empty_content() {
        let extractor = create_extractor();
        let result = extractor.extract_with_length_limits("", 10, 50);
        assert!(result.is_empty());
    }

    #[test]
    fn test_sentence_boundary_truncation() {
        let extractor = create_extractor();
        let text = "这是第一句。这是第二句。这是第三句。这是第四句。";

        let result = extractor.truncate_at_sentence_boundary(text, 20);
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.ends_with('。'));
        assert!(result.len() <= 20);
    }

    #[test]
    fn test_min_length_respected() {
        let extractor = create_extractor();
        let short_text = "简短";

        let result = extractor.extract_with_length_limits(short_text, 10, 50);
        assert_eq!(result, "简短");
    }

    #[test]
    fn test_max_length_respected() {
        let extractor = create_extractor();
        let long_text = "这是一个很长的文本，超过了最大长度限制，应该被截断。这是一个很长的文本，超过了最大长度限制，应该被截断。";

        let result = extractor.extract_with_length_limits(long_text, 10, 30);
        assert!(result.len() <= 30);
    }
}
