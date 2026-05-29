#!/usr/bin/env bash
# Usage: check-release-sync.sh <crate-name> [--require-date]
# Verifies:
#   1. Cargo.toml version matches CHANGELOG.md latest heading version.
#   2. If --require-date: CHANGELOG heading must include an ISO date (rejects "(unreleased)").
#   3. For raylib: the raylib-sys dep version-req matches the actual raylib-sys version.
set -euo pipefail

crate="${1:-}"
require_date="${2:-}"

if [ -z "$crate" ]; then
  echo "Usage: $0 <crate-name> [--require-date]" >&2
  exit 2
fi

manifest_version=$(cargo metadata --no-deps --format-version 1 \
  | jq -r ".packages[] | select(.name == \"$crate\") | .version")

if [ -z "$manifest_version" ] || [ "$manifest_version" = "null" ]; then
  echo "ERROR: could not read version for crate '$crate' from cargo metadata" >&2
  exit 1
fi

changelog_version=$(grep -oE '^## [0-9]+\.[0-9]+\.[0-9]+' CHANGELOG.md | head -1 | awk '{print $2}')

if [ -z "$changelog_version" ]; then
  echo "ERROR: could not parse latest version from CHANGELOG.md heading" >&2
  exit 1
fi

if [ "$manifest_version" != "$changelog_version" ]; then
  echo "ERROR: version mismatch for $crate" >&2
  echo "  Cargo.toml: $manifest_version" >&2
  echo "  CHANGELOG.md latest: $changelog_version" >&2
  exit 1
fi

if [ "$require_date" = "--require-date" ]; then
  if ! grep -qE '^## [0-9.]+ — [0-9]{4}-[0-9]{2}-[0-9]{2}' CHANGELOG.md; then
    echo "ERROR: CHANGELOG.md heading still says (unreleased) — flip to '## X.Y.Z — YYYY-MM-DD' before real publish" >&2
    exit 1
  fi
fi

if [ "$crate" = "raylib" ]; then
  sys_req=$(grep -E '^raylib-sys = .*version = "[0-9.]+"' raylib/Cargo.toml \
    | grep -oE '"[0-9.]+"' | head -1 | tr -d '"')
  if [ -z "$sys_req" ]; then
    echo "ERROR: could not parse raylib-sys dep pin from raylib/Cargo.toml" >&2
    exit 1
  fi
  if [ "$sys_req" != "$manifest_version" ]; then
    echo "ERROR: raylib-sys dep pin $sys_req != raylib version $manifest_version" >&2
    exit 1
  fi
fi

echo "OK: $crate @ $manifest_version, CHANGELOG @ $changelog_version${require_date:+, date-required passed}"
