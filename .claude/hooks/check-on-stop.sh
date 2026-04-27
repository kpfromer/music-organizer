#!/usr/bin/env bash
# Stop hook: run the full check matrix from tdds/implementation/conventions.md §8.
#
# Configured with `asyncRewake: true` in settings.json:
#   - Runs in background; does NOT block Claude returning to the user.
#   - On failure: exit 2 with summary on stdout → Claude wakes and is fed the
#     failure to fix on the next turn.
#   - On success: exit 0 silently.
#
# Two cheapness optimizations:
#   1. Skip entirely when neither Cargo.toml nor frontend/package.json exists
#      (phase 1 setup state).
#   2. Skip when no source files have changed since the last successful check
#      (sentinel: .claude/local/last-check-success).

set -u

SENTINEL=".claude/local/last-check-success"

# ---- Skip 1: codebase doesn't exist yet ---------------------------------
[ -f Cargo.toml ] || [ -f frontend/package.json ] || exit 0

# ---- Skip 2: nothing changed since last successful check ----------------
if [ -f "$SENTINEL" ]; then
  changed=$(find . \
      \( -path ./target -o \
         -path ./node_modules -o \
         -path ./frontend/node_modules -o \
         -path ./frontend/dist -o \
         -path ./.git -o \
         -path ./.claude/local \
      \) -prune \
      -o \( -name '*.rs' \
         -o -name '*.ts' -o -name '*.tsx' \
         -o -name '*.js' -o -name '*.jsx' \
         -o -name '*.json' -o -name '*.jsonc' \
         -o -name '*.css' \
         -o -name 'Cargo.toml' -o -name 'Cargo.lock' \
         -o -name 'package.json' -o -name 'pnpm-lock.yaml' \
         -o -name 'biome.json' -o -name 'tsconfig.json' \
         -o -name '*.sql' -o -name 'atlas.hcl' \
      \) \
      -newer "$SENTINEL" \
      -type f -print 2>/dev/null | head -1)
  [ -z "$changed" ] && exit 0
fi

# ---- Run checks ---------------------------------------------------------
failed=()
output=""

run_check() {
  local label="$1"
  shift
  local result rc
  result=$("$@" 2>&1)
  rc=$?
  if [ $rc -ne 0 ]; then
    failed+=("$label")
    local snippet
    snippet=$(printf '%s\n' "$result" | tail -n 50)
    output="${output}--- ${label} failed (exit ${rc}) ---
${snippet}

"
  fi
}

if [ -f Cargo.toml ] && command -v cargo >/dev/null 2>&1; then
  run_check "cargo fmt --check"  cargo fmt --check --all
  run_check "cargo clippy"        cargo clippy --workspace --all-targets -- -D warnings
  run_check "cargo build"         cargo build --workspace --all-targets
  run_check "cargo test"          cargo test --workspace --all-targets
fi

if [ -f frontend/package.json ] && command -v pnpm >/dev/null 2>&1; then
  pushd frontend >/dev/null
  run_check "frontend biome ci" pnpm --silent biome ci .
  run_check "frontend typecheck" pnpm --silent typecheck
  run_check "frontend test"      pnpm --silent test --run
  popd >/dev/null
fi

# ---- Decision -----------------------------------------------------------
if [ ${#failed[@]} -gt 0 ]; then
  printf 'Failed: %s\n\n%sFix these before the next return.\n' \
    "$(IFS=', '; echo "${failed[*]}")" \
    "$output"
  exit 2
fi

# All checks passed — record sentinel for next-run skip.
mkdir -p "$(dirname "$SENTINEL")"
: > "$SENTINEL"
exit 0
