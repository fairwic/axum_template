#!/usr/bin/env bash
# 开发前置检查工具：在提交代码前运行，确保符合所有开发规范
# 用法：在项目根目录执行 scripts/dev/pre-commit-check.sh

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  开发规范前置检查${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

FAILED=0

# ========== 1. 文件行数检查 ==========
echo -e "${BLUE}[1/5]${NC} 检查文件行数限制..."
if ./scripts/dev/check_code_file_line_limit.sh; then
    echo -e "${GREEN}✓ 文件行数检查通过${NC}"
else
    echo -e "${RED}✗ 文件行数检查失败${NC}"
    FAILED=1
fi
echo ""

# ========== 2. 代码格式检查 ==========
echo -e "${BLUE}[2/5]${NC} 检查代码格式..."
if cargo fmt --all --check >/dev/null 2>&1; then
    echo -e "${GREEN}✓ 代码格式符合规范${NC}"
else
    echo -e "${RED}✗ 代码格式不符合规范${NC}"
    echo -e "${YELLOW}  修复建议: cargo fmt --all${NC}"
    FAILED=1
fi
echo ""

# ========== 3. Clippy 静态检查 ==========
echo -e "${BLUE}[3/5]${NC} 执行 Clippy 静态检查..."
CLIPPY_OUTPUT=$(cargo clippy --workspace --all-targets --all-features 2>&1)
CLIPPY_WARNINGS=$(echo "$CLIPPY_OUTPUT" | grep -c "^warning:" || true)

if [ "$CLIPPY_WARNINGS" -eq 0 ]; then
    echo -e "${GREEN}✓ Clippy 检查通过（零警告）${NC}"
else
    echo -e "${RED}✗ Clippy 发现 $CLIPPY_WARNINGS 个警告${NC}"
    echo "$CLIPPY_OUTPUT" | grep "^warning:" | head -5
    FAILED=1
fi
echo ""

# ========== 4. 编译检查 ==========
echo -e "${BLUE}[4/5]${NC} 执行编译检查..."
if cargo check --workspace --all-targets >/dev/null 2>&1; then
    echo -e "${GREEN}✓ 编译检查通过${NC}"
else
    echo -e "${RED}✗ 编译失败${NC}"
    cargo check --workspace --all-targets 2>&1 | tail -20
    FAILED=1
fi
echo ""

# ========== 5. 单元测试 ==========
echo -e "${BLUE}[5/5]${NC} 运行单元测试..."
TEST_OUTPUT=$(cargo test --workspace 2>&1)
TEST_FAILED=$(echo "$TEST_OUTPUT" | grep -c "test result:.*FAILED" || true)

if [ "$TEST_FAILED" -eq 0 ]; then
    PASSED=$(echo "$TEST_OUTPUT" | grep "test result: ok" | tail -1)
    echo -e "${GREEN}✓ 测试通过: $PASSED${NC}"
else
    echo -e "${RED}✗ 测试失败${NC}"
    echo "$TEST_OUTPUT" | grep "test result:.*FAILED"
    FAILED=1
fi
echo ""

# ========== 结果汇总 ==========
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 所有检查通过，可以提交代码！${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    exit 0
else
    echo -e "${RED}💥 检查失败，请修复后再提交${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    exit 1
fi
