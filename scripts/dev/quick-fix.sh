#!/usr/bin/env bash
# 快速修复工具：自动修复可自动化的规范问题
# 用法：在项目根目录执行 scripts/dev/quick-fix.sh

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  快速修复工具${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# ========== 1. 自动格式化 ==========
echo -e "${BLUE}[1/3]${NC} 自动格式化代码..."
cargo fmt --all
echo -e "${GREEN}✓ 代码格式化完成${NC}"
echo ""

# ========== 2. Clippy 自动修复 ==========
echo -e "${BLUE}[2/3]${NC} 执行 Clippy 自动修复..."
cargo clippy --workspace --all-targets --fix --allow-dirty --allow-staged 2>&1 | grep -E "Fixed|Fixing" || echo -e "${YELLOW}  无可自动修复的问题${NC}"
echo -e "${GREEN}✓ Clippy 自动修复完成${NC}"
echo ""

# ========== 3. SQLx 离线缓存更新 ==========
echo -e "${BLUE}[3/3]${NC} 更新 SQLx 离线缓存..."
if [ -n "${DATABASE_URL:-}" ]; then
    cargo sqlx prepare --workspace
    echo -e "${GREEN}✓ SQLx 缓存已更新${NC}"
else
    echo -e "${YELLOW}⚠  DATABASE_URL 未设置，跳过 SQLx 缓存更新${NC}"
    echo -e "${YELLOW}  提示: export DATABASE_URL=postgres://...${NC}"
fi
echo ""

# ========== 验证修复结果 ==========
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🔧 快速修复完成${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${YELLOW}建议运行完整检查: ./scripts/dev/pre-commit-check.sh${NC}"
