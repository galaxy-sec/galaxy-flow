#!/bin/bash
# GXL AI-Native OpenAI Test Runner
# Usage: ./test_openai.sh [message]

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 GXL AI-Native Runtime Test${NC}"
echo -e "${BLUE}================================${NC}\n"

# Set defaults if no message provided
TEST_MESSAGE="${1:-Hello from Galaxy Flow AI!}"
TEST_TYPE="${2:-chat}"

# Check for API key
if [[ -n "$OPENAI_API_KEY" ]]; then
    echo -e "${GREEN}✅ Using OpenAI API with key${NC}"
else
    echo -e "${YELLOW}⚠️  No OPENAI_API_KEY detected - using Mock provider${NC}"
    export GXL_SIMPLE_MODEL=mock
fi

# Create temporary Rust file for test
cat > /tmp/test_ai.rs << 'EOF'
use galaxy_flow::ai::config::AiConfig;
use galaxy_flow::ai::{AiCapability, AiClient};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = AiConfig::load()?;
    let client = AiClient::new(config)?;

    // Simple message test
    let response = client.smart_request(
        AiCapability::Generate,
        "用一句话解释什么是零依赖AI",
    ).await?;

    println!("{}", response.content);
    Ok(())
}
EOF

# Run the test
if cargo run --bin gx-ai test --message "解释零依赖系统" 2>/dev/null; then
    echo -e "${GREEN}✅ AI-Native Runtime 验证成功!${NC}"
else
    echo -e "${YELLOW}⚙️  构建中...${NC}"
    cargo build --bin gx-ai

    echo -e "\n🔬 运行基础测试..."
    RUST_LOG=info cargo run --bin gx-ai test --message "${TEST_MESSAGE}"
fi

echo -e "\n${GREEN}🎉 测试完成！${NC}"
echo -e "${BLUE}💡 其他测试命令:${NC}"
echo "  ./test_openai.sh 'your message here'  # 自定义输入"
echo "  cargo run --bin gx-ai list           # 查看可用功能"
echo "  cargo run --bin gx-ai smart-commit   # 智能git提交"
