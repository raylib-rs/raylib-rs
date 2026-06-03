# WS8c — `act` validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Validate the two release workflows from WS8b locally via `act` (Docker) without producing any crates.io traffic. Record the validation outcomes — including the documented limitation that `cargo publish --dry-run -p raylib` fails at registry resolution until `raylib-sys 6.0.0` is real — in `docs/superpowers/notes/ws8c-validation.md`.

**Architecture:** Three `act` runs plus four direct invocations of the sync-check helper. Each run has a precise expected outcome. The notes file captures the runbook for whoever does the final-release step later.

**Tech Stack:** `act` (CLI, runs GitHub Actions in Docker), Docker, bash.

**Spec reference:** `docs/superpowers/specs/2026-05-29-ws8-release-prep-checkpoint-design.md` §4 WS8c.

---

### Task 1: Verify `act` + Docker are available

**Files:** none (precondition check)

- [ ] **Step 1: Check `act` is on PATH**

Run: `gh act --version 2>&1 || act --version 2>&1`
Expected: prints a version like `act version 0.2.x`. If both commands fail with "command not found", `act` isn't installed — install via the `gh act` GitHub CLI extension (`gh extension install nektos/gh-act`) or from <https://github.com/nektos/act>.

- [ ] **Step 2: Check Docker is reachable**

Run: `docker info 2>&1 | head -5`
Expected: prints Docker daemon info. If Docker is not running, the act runs will fail; start Docker first.

- [ ] **Step 3: Choose the `act` invocation prefix and record it**

If `gh act` works, use `gh act` for the rest of this plan. If only `act` works, substitute `act` everywhere `gh act` appears below. Record the choice for the notes file in Task 5.

### Task 2: Direct sync-check sanity check (no `act` required)

**Files:** none (verification only)

These are the same calls from WS8b Task 4 but re-run here as the WS8c baseline so the notes file can record current passing/failing behavior.

- [ ] **Step 1: Soft path passes**

Run:
```bash
bash scripts/check-release-sync.sh raylib-sys
bash scripts/check-release-sync.sh raylib
```
Expected: both print `OK: ... @ 6.0.0, CHANGELOG @ 6.0.0` and exit 0.

- [ ] **Step 2: Date-required path fails**

Run: `bash scripts/check-release-sync.sh raylib-sys --require-date 2>&1; echo "exit=$?"`
Expected: stderr contains `CHANGELOG.md heading still says (unreleased)`; last line `exit=1`.

Record both outcomes for the notes file.

### Task 3: `act` run #1 — `release-sys.yml` with `dry_run=true` (must SUCCEED)

**Files:** none (validation run)

- [ ] **Step 1: Run the workflow under act**

Run:
```bash
gh act workflow_dispatch \
  -W .github/workflows/release-sys.yml \
  --input dry_run=true \
  -s CARGO_REGISTRY_TOKEN=dummy 2>&1 | tee /tmp/ws8c-run1.log
```
Expected: every step succeeds. Look for a final `Job succeeded` or equivalent. The `cargo publish (real)` step should be **skipped** (it has the `if: ${{ inputs.dry_run == false }}` gate).

- [ ] **Step 2: Verify the gated step was skipped**

Run: `grep -i 'skip\|publish (real)' /tmp/ws8c-run1.log | head -10`
Expected: a line showing the "cargo publish (real)" step was skipped due to the `if:` condition.

- [ ] **Step 3: Record outcome**

Save the exit code (`echo $?` after the run) and the last 20 lines of the log for inclusion in the notes file.

### Task 4: `act` run #2 — `release-safe.yml` with `dry_run=true` (EXPECTED to fail at the cargo step, validates everything before)

**Files:** none (validation run)

This run validates everything up to the `cargo publish --dry-run -p raylib` step. That step is expected to fail because `raylib-sys 6.0.0` doesn't exist on crates.io yet. The point is to confirm the YAML parses, the sync-check runs, and the workflow reaches the cargo step.

- [ ] **Step 1: Run the workflow under act**

Run:
```bash
gh act workflow_dispatch \
  -W .github/workflows/release-safe.yml \
  --input dry_run=true \
  -s CARGO_REGISTRY_TOKEN=dummy 2>&1 | tee /tmp/ws8c-run2.log
```
Expected: the workflow fails at the `cargo publish (dry-run, always)` step with a dep-resolution error (something like `no matching package named raylib-sys found; required by raylib v6.0.0; perhaps a crate on crates.io is missing or not yet indexed`). Earlier steps (checkout, toolchain, jq install, sync-check) all pass.

- [ ] **Step 2: Verify the failure point is the cargo step, not earlier**

Run: `grep -B2 -A4 'cargo publish (dry-run' /tmp/ws8c-run2.log | head -20`
Expected: the cargo step is the failing step; the sync-check step succeeded before it.

- [ ] **Step 3: Record outcome**

This is a known limitation, documented in the notes file. Capture the exact error message for the final-release runbook so whoever runs the real release knows what to expect (and that running `release-safe.yml` immediately after `release-sys.yml` will succeed because `raylib-sys 6.0.0` will exist by then).

### Task 5: `act` run #3 — `release-sys.yml` with `dry_run=false` + dummy token (EXPECTED to fail at auth, not earlier)

**Files:** none (validation run)

This is the negative test that proves the `if: ${{ inputs.dry_run == false }}` gate actually exposes the real-publish step. With a dummy token, the real publish must fail at crates.io auth — not before.

- [ ] **Step 1: Run the workflow under act**

Run:
```bash
gh act workflow_dispatch \
  -W .github/workflows/release-sys.yml \
  --input dry_run=false \
  -s CARGO_REGISTRY_TOKEN=dummy 2>&1 | tee /tmp/ws8c-run3.log
```
Expected: the workflow runs through checkout, toolchain, jq install, then **fails at the sync-check step** (because the workflow now passes `--require-date` and the CHANGELOG still says `(unreleased)`).

- [ ] **Step 2: Verify the failure point**

Run: `grep -B2 -A4 'CHANGELOG.md heading still says\|Version + CHANGELOG sync check' /tmp/ws8c-run3.log | head -20`
Expected: the failure is `CHANGELOG.md heading still says (unreleased)` from the sync-check step. The real-publish step never runs.

This actually validates **two** things:
1. The `--require-date` gate fires when `dry_run: false`.
2. We never reach the real publish during validation, so no token-leak risk.

To exercise the real-publish step itself (which would also fail at auth with a dummy token), we'd need to first flip the CHANGELOG to a date. That's not WS8 work; record this as a future validation in the notes file.

- [ ] **Step 3: Record outcome**

### Task 6: Write `docs/superpowers/notes/ws8c-validation.md`

**Files:**
- Create: `docs/superpowers/notes/ws8c-validation.md`

- [ ] **Step 1: Write the notes file**

Create `docs/superpowers/notes/ws8c-validation.md` with content of this shape (fill in the actual command outputs from Tasks 2–5 where indicated):

```markdown
# WS8c — release workflow validation via `act`

**Spec:** `docs/superpowers/specs/2026-05-29-ws8-release-prep-checkpoint-design.md` §4 WS8c
**Plan:** `docs/superpowers/plans/2026-05-29-ws8c-act-validation.md`

Three `act` runs validate the two release workflows from WS8b without any
crates.io traffic. The runbook here doubles as the documentation a future
maintainer needs when running the real publish.

## Tooling

- `act` invocation prefix used: `gh act` / `act` (pick one based on Task 1).
- Docker version: <fill in from `docker --version`>.
- `act` runner image: default (`catthehacker/ubuntu:act-latest`); pin in `~/.actrc` if a specific image was required.

## Direct sync-check baseline (no `act`)

- `bash scripts/check-release-sync.sh raylib-sys` → exit 0, "OK: raylib-sys @ 6.0.0".
- `bash scripts/check-release-sync.sh raylib` → exit 0, "OK: raylib @ 6.0.0".
- `bash scripts/check-release-sync.sh raylib-sys --require-date` → exit 1, "CHANGELOG.md heading still says (unreleased)".

These confirm the sync-check helper works before any workflow-level validation.

## Run 1 — `release-sys.yml` with `dry_run=true`

**Command**: `gh act workflow_dispatch -W .github/workflows/release-sys.yml --input dry_run=true -s CARGO_REGISTRY_TOKEN=dummy`
**Outcome**: SUCCESS. All steps pass; the `cargo publish (real)` step is skipped via the `if` gate.

Last 10 lines of log:
```
<paste tail of /tmp/ws8c-run1.log>
```

## Run 2 — `release-safe.yml` with `dry_run=true` (KNOWN LIMITATION)

**Command**: `gh act workflow_dispatch -W .github/workflows/release-safe.yml --input dry_run=true -s CARGO_REGISTRY_TOKEN=dummy`
**Outcome**: FAILS at the `cargo publish --dry-run -p raylib` step with a dep-resolution error against crates.io.

**Why**: `raylib-sys 6.0.0` is not yet on crates.io. `cargo publish --dry-run -p raylib` strips the path-dep and resolves only the version-req (`raylib-sys = "6.0.0"`), which crates.io can't satisfy until `release-sys.yml` runs for real.

**Implication for the final-release runbook**: run `release-sys.yml` (real publish, then verify on crates.io) **before** running `release-safe.yml`. At that point this validation step will pass.

Exact error:
```
<paste the dep-resolution error from /tmp/ws8c-run2.log>
```

## Run 3 — `release-sys.yml` with `dry_run=false` + dummy token (negative test)

**Command**: `gh act workflow_dispatch -W .github/workflows/release-sys.yml --input dry_run=false -s CARGO_REGISTRY_TOKEN=dummy`
**Outcome**: FAILS at the sync-check step because `--require-date` rejects the still-unreleased CHANGELOG heading.

**What this validates**:
- The `--require-date` gate fires when `dry_run: false`.
- The real-publish step is never reached during validation, so dummy-token usage is safe.

**Pending validation that this WS8 can't perform** (requires flipping the CHANGELOG date, which is the future final-release step's work):
- Behavior of the real `cargo publish (real)` step with a dummy token. Expected: failure at crates.io auth.

## Gotchas hit / mitigations

<populate from actual run experience; e.g. "first run took 8 minutes because of submodule recursion in checkout", "had to add CARGO_NET_RETRY=10 because docker network was flaky", etc.>

## Fallback validation if `act` can't run

If Docker or `act` is unavailable on a future maintainer's machine, the equivalent direct validation is:

```bash
# Sync-check both crates
bash scripts/check-release-sync.sh raylib-sys
bash scripts/check-release-sync.sh raylib

# Package both crates without publishing
cargo publish -p raylib-sys --dry-run
# cargo publish -p raylib --dry-run  # will fail until raylib-sys is published for real

# Validate workflow YAML parses (optional)
python -c "import yaml; yaml.safe_load(open('.github/workflows/release-sys.yml'))"
python -c "import yaml; yaml.safe_load(open('.github/workflows/release-safe.yml'))"
```

This catches sync-check + packaging errors but not workflow-level YAML/step gating errors. `act` is the preferred validation.

## Final-release runbook teaser

The full final-release runbook is part of the future final-release workstream. The relevant WS8c outcome:

1. Flip `CHANGELOG.md` heading `## 6.0.0 (unreleased)` → `## 6.0.0 — YYYY-MM-DD`.
2. Trigger `release-sys.yml` with `dry_run: true` first — must pass cleanly now that the CHANGELOG has a date.
3. Trigger `release-sys.yml` with `dry_run: false` — publishes `raylib-sys 6.0.0`.
4. Verify on crates.io.
5. Trigger `release-safe.yml` with `dry_run: true` — now passes because `raylib-sys 6.0.0` resolves.
6. Trigger `release-safe.yml` with `dry_run: false` — publishes `raylib 6.0.0`.
```

- [ ] **Step 2: Fill in the captured outputs**

Replace the `<paste ...>` and `<populate ...>` placeholders with the actual command outputs and observations from Tasks 2–5.

- [ ] **Step 3: Spot-check no `<...>` placeholders remain**

Run: `grep -n '<paste\|<populate\|<fill in' docs/superpowers/notes/ws8c-validation.md`
Expected: zero matches.

### Task 7: Commit WS8c

**Files:**
- Create: `docs/superpowers/notes/ws8c-validation.md`

- [ ] **Step 1: Confirm working-tree contents**

Run: `git status --short`
Expected: one new untracked file `docs/superpowers/notes/ws8c-validation.md`. Untracked files in repo root (`TODO.md`, `prompt.md`, `next-session-prompt.md`) remain untracked.

- [ ] **Step 2: Stage the notes file**

```bash
git add docs/superpowers/notes/ws8c-validation.md
```

- [ ] **Step 3: Verify staged contents**

Run: `git diff --cached --stat`
Expected: one new file.

- [ ] **Step 4: Commit**

```bash
git commit -m "$(cat <<'EOF'
docs(ws8c): validate release workflows via act + record runbook notes

Three act runs validate release-sys.yml + release-safe.yml without
any crates.io traffic:

1. release-sys.yml + dry_run=true             -> SUCCESS
2. release-safe.yml + dry_run=true            -> FAILS at cargo step
   (raylib-sys 6.0.0 isn't on crates.io yet — documented limitation)
3. release-sys.yml + dry_run=false + dummy token -> FAILS at sync-check
   (--require-date rejects "(unreleased)" CHANGELOG; real publish
   step never reached, so dummy-token usage is safe during validation)

The notes file doubles as the final-release runbook teaser and the
fallback direct-validation sequence for maintainers without act.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 5: Verify the commit**

Run: `git log -1 --stat`
Expected: shows the WS8c notes commit.

---

## WS8c complete when

- All three `act` runs have been performed with the expected outcomes (succeed / known-limitation failure / sync-check-gate failure) recorded.
- `docs/superpowers/notes/ws8c-validation.md` exists and has no `<paste …>` / `<populate …>` placeholders.
- One commit recorded with the WS8c-shaped message.

Next: WS8d (release-hygiene fold-ins).
