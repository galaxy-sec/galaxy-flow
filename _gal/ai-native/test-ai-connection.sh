#!/bin/bash
# GXL AI原生验证 - 第1步：直接AI连接测试
# Usage: ./test-ai-connection.sh [message]

API_KEY="${SEC_KIMI_KEY:-$1}"
TEST_MESSAGE="${2:-测试GXL-AI原生连接，回应当前工作流能力验证}"

if [ -z "$API_KEY" ]; then
    echo "❌ 需要OPENAI_API_KEY环境变量" >&2
    exit 1
fi

RESPONSE=$(curl -s -X POST "https://api.moonshot.cn/v1/chat/completions" \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "kimi-k2-turbo-preview",
    "messages": [
      {"role": "system", "content": "你是一个高效的DevOps助手，任务是验证GXL-AI集成是否正常工作。用一行回应。"},
      {"role": "user", "content": "测试消息: '"$TEST_MESSAGE"'，请确认AI连接成功并简短描述当前处理能力"}
    ],
    "max_tokens": 50
}' | jq -r '.choices[0].message.content')

echo "🎯 AI响应: $RESPONSE"
echo "✅ GXL-AI集成验证完成"
