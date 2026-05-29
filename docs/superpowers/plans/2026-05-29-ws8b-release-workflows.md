# WS8b — Release workflows + sync-check helper Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Draft two `workflow_dispatch`-only release workflows (`release-sys.yml`, `release-safe.yml`) plus the shared `scripts/check-release-sync.sh` helper. Workflows are authored to never publish automatically (`dry_run: bool` input, default `true`); the secret token is consumed only when `dry_run: false`.

**Architecture:** Two near-identical YAML files (one per crate, since two-call CI is simpler than one reusable workflow with matrix). One bash helper enforces version/CHANGELOG consistency; `--require-date` is the safety gate for real publish. The workflows are validated end-to-end in WS8c via `act`; WS8b only verifies the sync-check helper directly.

**Tech Stack:** GitHub Actions YAML, bash, `cargo metadata`, `jq`.

**Spec reference:** `docs/superpowers/specs/2026-05-29-ws8-release-prep-checkpoint-design.md` §4 WS8b.

---

### Task 1: Create `scripts/check-release-sync.sh`

**Files:**
- Create: `scripts/check-release-sync.sh`

- [ ] **Step 1: Confirm `scripts/` directory state**

Run: `ls scripts/ 2>&1 || echo "missing"`
Expected: either lists existing files or prints `missing`. If missing, the file creation below will fail until `mkdir -p scripts` runs.

- [ ] **Step 2: Create the directory if needed**

Run: `mkdir -p scripts`
Expected: silent success or "directory exists".

- [ ] **Step 3: Write the helper script**

Create `scripts/check-release-sync.sh` with this exact content:

```bash
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
```

- [ ] **Step 4: Make it executable**

Run: `chmod +x scripts/check-release-sync.sh`
Expected: silent success.

- [ ] **Step 5: Confirm file mode**

Run: `ls -la scripts/check-release-sync.sh`
Expected: `-rwxr-xr-x` or similar (executable bit set).

### Task 2: Verify sync-check passes for raylib-sys without `--require-date`

**Files:** none (verification only)

- [ ] **Step 1: Run the helper**

Run: `bash scripts/check-release-sync.sh raylib-sys`
Expected: exits 0 with `OK: raylib-sys @ 6.0.0, CHANGELOG @ 6.0.0`. (Requires WS8a's version bump to have landed first.)

- [ ] **Step 2: Confirm exit code**

Run: `bash scripts/check-release-sync.sh raylib-sys; echo "exit=$?"`
Expected: last line `exit=0`.

### Task 3: Verify sync-check passes for raylib without `--require-date`

**Files:** none (verification only)

- [ ] **Step 1: Run the helper**

Run: `bash scripts/check-release-sync.sh raylib`
Expected: exits 0 with `OK: raylib @ 6.0.0, CHANGELOG @ 6.0.0`.

### Task 4: Verify sync-check FAILS with `--require-date` (proves the gate works)

**Files:** none (verification only)

- [ ] **Step 1: Run with the require-date flag**

Run: `bash scripts/check-release-sync.sh raylib-sys --require-date; echo "exit=$?"`
Expected: `ERROR: CHANGELOG.md heading still says (unreleased) ...` on stderr; last line `exit=1`.

- [ ] **Step 2: Same for raylib**

Run: `bash scripts/check-release-sync.sh raylib --require-date; echo "exit=$?"`
Expected: same failure shape; `exit=1`.

This is exactly the behavior we want — the workflows will only pass `--require-date` when `dry_run: false`, so `act` validation in WS8c (which always uses `dry_run: true`) sees the soft path, and the real publish path is gated.

### Task 5: Create `.github/workflows/release-sys.yml`

**Files:**
- Create: `.github/workflows/release-sys.yml`

- [ ] **Step 1: Confirm `.github/workflows/` exists**

Run: `ls .github/workflows/`
Expected: lists `book.yml`, `check.yml`, `sanitizers.yml`, `test.yml`, `web.yml`.

- [ ] **Step 2: Write the workflow**

Create `.github/workflows/release-sys.yml` with this exact content:

```yaml
name: Release raylib-sys

on:
  workflow_dispatch:
    inputs:
      dry_run:
        description: "Dry-run only (no cargo publish). Default: true. Set to false for real publish."
        type: boolean
        default: true

jobs:
  publish-sys:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v5
        with:
          submodules: recursive

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Install jq (sync-check dep)
        run: sudo apt-get update && sudo apt-get install -y jq

      - name: Version + CHANGELOG sync check
        run: |
          if [ "${{ inputs.dry_run }}" = "false" ]; then
            bash scripts/check-release-sync.sh raylib-sys --require-date
          else
            bash scripts/check-release-sync.sh raylib-sys
          fi

      - name: cargo publish (dry-run, always)
        run: cargo publish -p raylib-sys --dry-run

      - name: cargo publish (real)
        if: ${{ inputs.dry_run == false }}
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_IO_TOKEN }}
        run: cargo publish -p raylib-sys
```

- [ ] **Step 3: Verify the file landed**

Run: `head -5 .github/workflows/release-sys.yml`
Expected: starts with `name: Release raylib-sys`.

### Task 6: Create `.github/workflows/release-safe.yml`

**Files:**
- Create: `.github/workflows/release-safe.yml`

- [ ] **Step 1: Write the workflow**

Create `.github/workflows/release-safe.yml` with this exact content:

```yaml
name: Release raylib

on:
  workflow_dispatch:
    inputs:
      dry_run:
        description: "Dry-run only (no cargo publish). Default: true. Set to false for real publish. NOTE: cargo publish --dry-run for raylib requires raylib-sys 6.0.0 to exist on crates.io; the dry-run step will fail at registry resolution otherwise."
        type: boolean
        default: true

jobs:
  publish-safe:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v5
        with:
          submodules: recursive

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Install jq (sync-check dep)
        run: sudo apt-get update && sudo apt-get install -y jq

      - name: Version + CHANGELOG sync check
        run: |
          if [ "${{ inputs.dry_run }}" = "false" ]; then
            bash scripts/check-release-sync.sh raylib --require-date
          else
            bash scripts/check-release-sync.sh raylib
          fi

      - name: cargo publish (dry-run, always)
        run: cargo publish -p raylib --dry-run

      - name: cargo publish (real)
        if: ${{ inputs.dry_run == false }}
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_IO_TOKEN }}
        run: cargo publish -p raylib
```

- [ ] **Step 2: Verify the file landed**

Run: `head -5 .github/workflows/release-safe.yml`
Expected: starts with `name: Release raylib`.

### Task 7: Basic YAML syntax check on both workflows

**Files:** none (verification only)

- [ ] **Step 1: Use `python -c yaml.safe_load` as a lightweight parser check**

Run:
```bash
python -c "import yaml; yaml.safe_load(open('.github/workflows/release-sys.yml'))" && echo "release-sys OK"
python -c "import yaml; yaml.safe_load(open('.github/workflows/release-safe.yml'))" && echo "release-safe OK"
```
Expected: both print `OK`. If Python isn't on PATH, fall back to:
```bash
gh workflow view release-sys --repo Dacode45/ms-raylib-rs 2>&1 | head -5
```
(but `gh workflow view` requires the workflow to be on a pushed branch; if it isn't yet, skip and let WS8c's `act` run be the YAML validation).

- [ ] **Step 2: Spot-check for the expected gating pattern**

Run: `grep -n 'inputs.dry_run' .github/workflows/release-sys.yml .github/workflows/release-safe.yml`
Expected: each file shows the `if: ${{ inputs.dry_run == false }}` gate on the real publish step + the conditional `--require-date` invocation in the sync-check step.

### Task 8: Commit WS8b

**Files:**
- Create: `scripts/check-release-sync.sh`
- Create: `.github/workflows/release-sys.yml`
- Create: `.github/workflows/release-safe.yml`

- [ ] **Step 1: Confirm working-tree contents**

Run: `git status --short`
Expected: three new untracked files listed. **Verify** `TODO.md`, `prompt.md`, `next-session-prompt.md` are still untracked and NOT in any staging list.

- [ ] **Step 2: Stage the new files explicitly (no `git add -A`)**

```bash
git add scripts/check-release-sync.sh .github/workflows/release-sys.yml .github/workflows/release-safe.yml
```

- [ ] **Step 3: Verify staged contents**

Run: `git diff --cached --stat`
Expected: three new files, exactly the ones above.

- [ ] **Step 4: Commit**

```bash
git commit -m "$(cat <<'EOF'
feat(ws8b): draft release-sys.yml + release-safe.yml + sync-check helper

Two workflow_dispatch-only release workflows, each with a `dry_run: bool`
input (default `true`). Real publish is gated by
`if: ${{ inputs.dry_run == false }}` so the CARGO_REGISTRY_TOKEN secret
is only consumed during an intentional real publish.

`scripts/check-release-sync.sh` enforces:
- Cargo.toml version == CHANGELOG.md latest heading version
- (with --require-date) CHANGELOG heading has an ISO date, not "(unreleased)"
- For raylib only: the raylib-sys dep version-req matches the
  just-bumped raylib-sys version

The workflows pass --require-date to the sync-check only when
`dry_run: false`, so act-based validation in WS8c passes with
the CHANGELOG still saying "(unreleased)".

The workflows do NOT run automatically; no `on: push` or `on: tags`
triggers, only `workflow_dispatch`.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 5: Verify the commit**

Run: `git log -1 --stat`
Expected: shows the commit with the three new files and the WS8b-shaped message.

---

## WS8b complete when

- `scripts/check-release-sync.sh` exists, is executable, and passes for both crates without `--require-date`.
- Same script fails (exit 1) for both crates with `--require-date` (CHANGELOG still says `(unreleased)`).
- `.github/workflows/release-sys.yml` and `.github/workflows/release-safe.yml` exist.
- Both workflows are `workflow_dispatch`-only (no `on: push` or `on: tags`).
- Both workflows gate real publish behind `if: ${{ inputs.dry_run == false }}`.
- YAML parses (Python or `gh workflow view` or `act` in WS8c — at minimum one syntax check passed).
- One commit recorded with the WS8b-shaped message.

Next: WS8c (act-based validation).
