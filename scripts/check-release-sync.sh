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

## SemVer-shaped version, including optional pre-release tag (`-rc.1`,
## `-alpha.2`, `-beta+build`, etc.) per <https://semver.org/#spec-item-9>.
## Build-metadata (`+...`) intentionally omitted — cargo doesn't surface it
## via `cargo metadata` and it doesn't participate in version equality.
changelog_version=$(grep -oE '^## [0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?' CHANGELOG.md | head -1 | awk '{print $2}')

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
  ## Allow SemVer pre-release suffixes between the version and the em-dash
  ## (e.g. `## 6.0.0-rc.1 — 2026-06-01`).
  if ! grep -qE '^## [0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)? — [0-9]{4}-[0-9]{2}-[0-9]{2}' CHANGELOG.md; then
    offending=$(grep -E '^## [0-9]' CHANGELOG.md | head -1)
    echo "ERROR: CHANGELOG.md heading '## X.Y.Z[-pre]' must be followed by ' — YYYY-MM-DD' (em-dash U+2014 + ISO date) before real publish" >&2
    echo "  found: $offending" >&2
    exit 1
  fi
fi

if [ "$crate" = "raylib" ]; then
  # Pad regex with \s* so TOML whitespace variations (cargo accepts any) don't cause silent
  # pipefail failures. The trailing `|| true` neutralizes pipefail on no-match; the explicit
  # `[ -z "$sys_req" ]` guard below produces the actual error message.
  sys_req=$(grep -E '^raylib-sys\s*=\s*.*version\s*=\s*"[0-9.]+(-[0-9A-Za-z.-]+)?"' raylib/Cargo.toml \
    | grep -oE '"[0-9.]+(-[0-9A-Za-z.-]+)?"' | head -1 | tr -d '"' || true)
  if [ -z "$sys_req" ]; then
    echo "ERROR: could not parse raylib-sys dep pin from raylib/Cargo.toml (line should look like: raylib-sys = { version = \"X.Y.Z\", ... })" >&2
    exit 1
  fi
  if [ "$sys_req" != "$manifest_version" ]; then
    echo "ERROR: raylib-sys dep pin $sys_req != raylib version $manifest_version" >&2
    exit 1
  fi
fi

echo "OK: $crate @ $manifest_version, CHANGELOG @ $changelog_version${require_date:+, date-required passed}"
