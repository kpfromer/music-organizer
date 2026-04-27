#!/usr/bin/env bash
# PostToolUse hook: format files Claude has just written/edited.
# - .rs        → rustfmt
# - .ts/tsx/json/css → biome (only if frontend/biome.json exists)
# Silent no-op on missing tooling, missing files, or pre-existing build state.
# Always exits 0 — formatting failures must not block Claude's iteration.

set -u

# Stdin is hook JSON. Pull the affected file path.
file=$(jq -r '.tool_response.filePath // .tool_input.file_path // empty' 2>/dev/null)
[ -z "$file" ] && exit 0
[ -f "$file" ] || exit 0

case "$file" in
  *.rs)
    command -v rustfmt >/dev/null 2>&1 || exit 0
    rustfmt --edition 2021 "$file" 2>/dev/null || true
    ;;
  *.ts|*.tsx|*.js|*.jsx|*.json|*.jsonc|*.css)
    [ -f frontend/biome.json ] || exit 0
    command -v pnpm >/dev/null 2>&1 || exit 0
    (cd frontend && pnpm --silent biome format --write "$file" 2>/dev/null) || true
    ;;
esac

exit 0
