#!/usr/bin/env bash
# 检查源码文件行数限制：≤1000 行 WARN，≤2000 行硬限
# 用法：在项目根目录执行 scripts/dev/check_code_file_line_limit.sh

set -euo pipefail

WARN_LIMIT=1000
ERROR_LIMIT=2000

# 扫描的文件扩展名（Rust + 前端）
EXTENSIONS=("rs" "ts" "tsx" "js" "jsx" "mjs" "mts" "vue" "css" "scss")

RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

declare -a warnings=()
declare -a errors=()

echo "🔍 检查源码文件行数限制..."
echo "   - WARN:  > ${WARN_LIMIT} 行"
echo "   - ERROR: > ${ERROR_LIMIT} 行"
echo ""

# 构建 find 的 -name 参数
find_args=()
for ext in "${EXTENSIONS[@]}"; do
  if [ ${#find_args[@]} -gt 0 ]; then
    find_args+=("-o")
  fi
  find_args+=("-name" "*.${ext}")
done

# 扫描 crates/ 和 bins/ 目录
while IFS= read -r file; do
  line_count=$(wc -l < "$file" | tr -d ' ')

  if [ "$line_count" -gt "$ERROR_LIMIT" ]; then
    errors+=("$file: $line_count 行 (超过 $ERROR_LIMIT 行硬限)")
  elif [ "$line_count" -gt "$WARN_LIMIT" ]; then
    warnings+=("$file: $line_count 行 (超过 $WARN_LIMIT 行建议)")
  fi
done < <(find crates bins -type f \( "${find_args[@]}" \) 2>/dev/null)

# 输出结果
if [ ${#errors[@]} -gt 0 ]; then
  echo -e "${RED}❌ 发现 ${#errors[@]} 个文件超过 ${ERROR_LIMIT} 行硬限：${NC}"
  for err in "${errors[@]}"; do
    echo -e "   ${RED}$err${NC}"
  done
  echo ""
fi

if [ ${#warnings[@]} -gt 0 ]; then
  echo -e "${YELLOW}⚠️  发现 ${#warnings[@]} 个文件超过 ${WARN_LIMIT} 行建议：${NC}"
  for warn in "${warnings[@]}"; do
    echo -e "   ${YELLOW}$warn${NC}"
  done
  echo ""
fi

if [ ${#errors[@]} -eq 0 ] && [ ${#warnings[@]} -eq 0 ]; then
  echo -e "${GREEN}✅ 所有源码文件行数符合规范${NC}"
  exit 0
fi

if [ ${#errors[@]} -gt 0 ]; then
  echo -e "${RED}💥 检查失败：存在超过 ${ERROR_LIMIT} 行的文件，请拆分后再提交${NC}"
  exit 1
fi

if [ ${#warnings[@]} -gt 0 ]; then
  echo -e "${YELLOW}⚡ 建议：存在超过 ${WARN_LIMIT} 行的文件，建议拆分以提升可维护性${NC}"
  exit 0
fi
