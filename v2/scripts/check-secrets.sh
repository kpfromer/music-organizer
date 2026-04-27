#!/usr/bin/env bash
# Aborts commit if staged files contain credential-shaped strings.

set -euo pipefail

if [ "$#" -eq 0 ]; then
    exit 0
fi

PATTERNS=(
    'AKIA[0-9A-Z]{16}'
    '-----BEGIN (RSA|OPENSSH|EC|DSA|PGP) PRIVATE KEY-----'
    'MM_SOULSEEK_PASSWORD=[^[:space:]]+'
    'MM_ACOUSTID_API_KEY=[^[:space:]]{16,}'
    'gh[ps]_[A-Za-z0-9]{36}'
)

found=0
for file in "$@"; do
    [ -f "$file" ] || continue
    for pat in "${PATTERNS[@]}"; do
        if grep -E -q "$pat" "$file"; then
            echo "secret-like string in $file (pattern: $pat)" >&2
            found=1
        fi
    done
done

exit "$found"
