#!/usr/bin/env bash
# DTO 验证测试工具：交互式测试 DTO 验证规则
# 用法：scripts/dev/test-dto-validation.sh

set -euo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  DTO 验证规则测试${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

API_BASE="http://localhost:3000"

echo -e "${BLUE}测试目标:${NC} CreateAddressRequest 验证规则"
echo -e "${BLUE}API 端点:${NC} POST $API_BASE/api/addresses"
echo ""

# ========== 测试用例 ==========

echo -e "${BLUE}[测试 1]${NC} 正常请求（应成功）"
curl -s -X POST "$API_BASE/api/addresses" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "name": "张三",
    "phone": "13800138000",
    "detail": "北京市朝阳区望京街道xx路xx号",
    "lat": 39.9042,
    "lng": 116.4074,
    "is_default": true
  }' | jq '.' || echo -e "${RED}✗ 请求失败（可能服务未启动）${NC}"
echo ""

echo -e "${BLUE}[测试 2]${NC} 姓名超长（应失败：> 50 字符）"
curl -s -X POST "$API_BASE/api/addresses" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "name": "这是一个非常非常非常非常非常非常非常非常非常非常非常非常非常非常非常长的名字超过五十个字符",
    "phone": "13800138000",
    "detail": "北京市朝阳区",
    "is_default": false
  }' | jq '.message' || echo -e "${RED}✗ 请求失败${NC}"
echo ""

echo -e "${BLUE}[测试 3]${NC} 手机号格式错误（应失败：非 11 位数字）"
curl -s -X POST "$API_BASE/api/addresses" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "name": "李四",
    "phone": "12345",
    "detail": "上海市浦东新区",
    "is_default": false
  }' | jq '.message' || echo -e "${RED}✗ 请求失败${NC}"
echo ""

echo -e "${BLUE}[测试 4]${NC} 纬度超出范围（应失败：> 90.0）"
curl -s -X POST "$API_BASE/api/addresses" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "name": "王五",
    "phone": "13900139000",
    "detail": "广州市天河区",
    "lat": 100.0,
    "lng": 113.0,
    "is_default": false
  }' | jq '.message' || echo -e "${RED}✗ 请求失败${NC}"
echo ""

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}测试完成${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${BLUE}提示:${NC}"
echo "  - 确保服务已启动: cargo run -p axum-server"
echo "  - 替换 YOUR_TOKEN 为真实 JWT（或移除 Authorization 头如果接口无需认证）"
echo "  - 查看详细响应: 在测试 curl 命令后添加 -v 参数"
