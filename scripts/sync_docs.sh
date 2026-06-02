#!/usr/bin/env bash
#
# sync_docs.sh — 自动从 MiniMax 官方文档站拉取最新 API 文档到 docs/
#
# 用法:
#   ./scripts/sync_docs.sh                  # 同步本项目涉及的全部文档
#   ./scripts/sync_docs.sh all              # 同步 llms.txt 索引中所有文档
#   ./scripts/sync_docs.sh <slug> [<slug>…] # 同步指定的文档 slug（不含 .md 后缀）
#   ./scripts/sync_docs.sh --list           # 列出 llms.txt 中所有可同步的文档
#
# Slug 示例: speech-t2a-http、video-generation-s2v、music-generation

set -euo pipefail

INDEX_URL="https://platform.minimaxi.com/docs/llms.txt"
DOCS_BASE="https://platform.minimaxi.com/docs/api-reference"
DOCS_DIR="$(cd "$(dirname "$0")/.." && pwd)/docs"

# 本项目相关的文档 slug（与 src/tools/* 一一对应；增删时同步修改本列表）
PROJECT_SLUGS=(
  speech-t2a-http
  speech-t2a-async-create
  speech-t2a-async-query
  video-generation-i2v
  video-generation-fl2v
  video-generation-s2v
  video-agent-create
  video-agent-query
  image-generation-t2i
  image-generation-i2i
  music-generation
  music-cover-preprocess
  lyrics-generation
  file-management-upload
  file-management-list
  file-management-retrieve
  file-management-retrieve-content
  file-management-delete
  text-chat-anthropic
  text-chat-openai
  responses-create
  api-overview
  errorcode
)

usage() {
  cat <<EOF
用法: $(basename "$0") [all | --list | <slug>...]

  无参数       同步本项目相关的全部文档
  all          同步 llms.txt 索引中所有文档
  --list       列出 llms.txt 索引中所有可同步的文档 slug
  <slug>...    同步指定的文档 slug（多个用空格分隔，不含 .md 后缀）

文档会保存到: $DOCS_DIR/
  - 项目范围: <slug>.md
  - 全量同步: minimax-<slug>.md  (避免与现有文件冲突)
EOF
}

# 从 llms.txt 解析 slug 列表（每行 "- [标题](URL): 描述"）
list_slugs() {
  curl -fsSL "$INDEX_URL" \
    | grep -oE 'api-reference/[^)]+\.md' \
    | sed -E 's|api-reference/||; s|\.md$||' \
    | sort -u
}

# 拉取单个 slug 的内容
fetch_one() {
  local slug="$1"
  local url="$DOCS_BASE/${slug}.md"
  local out
  out="$DOCS_DIR/${slug}.md"
  if ! curl -fsSL "$url" -o "$out"; then
    echo "  ✗ $slug (下载失败: $url)" >&2
    return 1
  fi
  local size
  size=$(wc -c <"$out" | tr -d ' ')
  echo "  ✓ $slug.md ($size bytes)"
}

case "${1:-}" in
  "")
    echo "→ 同步本项目相关的 ${#PROJECT_SLUGS[@]} 个文档..."
    mkdir -p "$DOCS_DIR"
    for slug in "${PROJECT_SLUGS[@]}"; do
      fetch_one "$slug" || true
    done
    echo "完成。文档位于: $DOCS_DIR/"
    ;;
  all)
    echo "→ 同步 llms.txt 索引中所有文档..."
    mkdir -p "$DOCS_DIR"
    mapfile -t slugs < <(list_slugs)
    echo "共 ${#slugs[@]} 个文档"
    for slug in "${slugs[@]}"; do
      fetch_one "$slug" || true
    done
    echo "完成。文档位于: $DOCS_DIR/"
    ;;
  --list)
    list_slugs
    ;;
  -h|--help|help)
    usage
    ;;
  *)
    echo "→ 同步指定 slug: $*"
    mkdir -p "$DOCS_DIR"
    for slug in "$@"; do
      fetch_one "$slug" || true
    done
    echo "完成。文档位于: $DOCS_DIR/"
    ;;
esac
