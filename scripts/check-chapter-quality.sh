#!/usr/bin/env bash
# 章节质量检查脚本
# 检查每章的 Mermaid 图数量、源码引用数量、源码路径有效性

set -euo pipefail

BOOK_DIR="$(cd "$(dirname "$0")/../book/src" && pwd)"
RESTORED_SRC="$(cd "$(dirname "$0")/../restored-src" && pwd)"
PASS=0
WARN=0
FAIL=0

printf "%-30s %8s %8s %8s\n" "章节" "Mermaid" "源码引用" "状态"
printf "%-30s %8s %8s %8s\n" "-----" "-------" "-------" "----"

for chapter in "$BOOK_DIR"/part*/ch*.md; do
    filename=$(basename "$chapter" .md)

    # 统计 mermaid 代码块数量
    mermaid_count=$(grep -c '```mermaid' "$chapter" 2>/dev/null || true)
    mermaid_count=${mermaid_count:-0}
    mermaid_count=$(echo "$mermaid_count" | tr -d '[:space:]')

    # 统计 restored-src/src/ 引用数量
    ref_count=$(grep -c 'restored-src/src/' "$chapter" 2>/dev/null || true)
    ref_count=${ref_count:-0}
    ref_count=$(echo "$ref_count" | tr -d '[:space:]')

    # 判断状态
    status="OK"
    if [ "$mermaid_count" -lt 1 ]; then
        status="WARN:无图"
        WARN=$((WARN + 1))
    elif [ "$ref_count" -lt 3 ]; then
        status="WARN:少引用"
        WARN=$((WARN + 1))
    else
        PASS=$((PASS + 1))
    fi

    printf "%-30s %8d %8d %8s\n" "$filename" "$mermaid_count" "$ref_count" "$status"
done

echo ""
echo "--- 源码路径有效性检查 ---"

invalid_paths=0
for chapter in "$BOOK_DIR"/part*/ch*.md; do
    # 提取所有 restored-src/src/ 路径（去掉行号部分）
    paths=$(grep -oE 'restored-src/src/[a-zA-Z0-9_./-]+\.(ts|tsx|js)' "$chapter" 2>/dev/null | sort -u || true)
    for path in $paths; do
        full_path="$RESTORED_SRC/src/${path#restored-src/src/}"
        if [ ! -f "$full_path" ]; then
            echo "  INVALID: $(basename "$chapter"): $path"
            invalid_paths=$((invalid_paths + 1))
        fi
    done
done

if [ "$invalid_paths" -eq 0 ]; then
    echo "  所有源码路径有效"
fi

echo ""
echo "--- 汇总 ---"
echo "通过: $PASS  警告: $WARN  无效路径: $invalid_paths"

if [ "$FAIL" -gt 0 ] || [ "$invalid_paths" -gt 5 ]; then
    exit 1
fi
