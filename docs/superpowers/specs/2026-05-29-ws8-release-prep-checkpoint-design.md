# WS8 — Release prep + checkpoint

**Status:** design approved 2026-05-29. Branch: `6.0-rc`. Implementation forthcoming under `docs/superpowers/plans/`.

WS8 is **a checkpoint, not a release.** It bumps versions, drafts the publish workflows, validates them locally via `act`, folds in agreed release-hygiene items, and ends at a fork-only PR (`Dacode45:6.0-rc` → `Dacode45:unstable`) so the owner can review every diff before the project advances to WS9.

No code is published to crates.io in WS8. No merge into `raylib-rs/raylib-rs` happens in WS8. Those steps move to a future final-release workstream that runs after WS9 (showcase rewrite) plus any further work the owner decides to add at this checkpoint.

Spec inputs: roadmap `docs/superpowers/specs/2026-05-25-raylib-rs-6.0-roadmap-design.md` §6 (WS8 row) + §7 (`release.yml` originally tag-triggered — superseded here by the checkpoint scope); WS7 done-state `docs/superpowers/notes/ws7-complete.md`; user-decided structural answers recorded below.

---

## 1. Goals

1. Bring both crate manifests + the book version snippet from `5.7.0` to `6.0.0` in lock-step so the fork's `unstable` branch carries the right version when the owner reviews.
2. Author two publish workflows — `release-sys.yml` and `release-safe.yml` — so the future final-release step has a known-working publish path with a real-publish safety gate (default `dry_run: true`).
3. Validate both workflows locally via `act` (Docker) without producing any crates.io traffic. Document any `act`-specific gotchas hit and the fallback bash sequence for a maintainer who can't use `act`.
4. Fold in the three release-hygiene items the owner authorized: Node.js-20 actions bump, `paste` cargo-deny advisory resolution (`deny.toml` ignore + tracked-deferred follow-up), and `samples/` deletion in favor of `showcase/`.
5. Open a single PR `Dacode45:6.0-rc` → `Dacode45:unstable` for the owner's exhaustive line-by-line review. Merge with `--no-ff` ("Create a merge commit" on GitHub) so all per-WS commits and backlog-PR `--author` attribution survive.

Done-criteria are in §10.

## 2. Non-goals

- **No crates.io publishing.** WS8 authors and tests the workflows; it never runs them with `dry_run: false`.
- **No canonical merge.** `Dacode45:unstable` → `raylib-rs:unstable` is the future final-release step.
- **No CHANGELOG date flip.** `## 6.0.0 (unreleased)` stays as-is; the date is set when the actual release happens.
- **No `v6.0.0` tag.** Tagging is part of the final-release step.
- **No GitHub release creation.**
- **No new safe-crate behavior changes** beyond the structopt removal that falls out of the samples deletion.

## 3. Locked decisions (owner-confirmed during brainstorm)

| # | Decision | Resolution |
|---|----------|------------|
| D1 | Version bump scope | Both crates → `6.0.0` lock-step, single commit (continues the existing `5.7.0`-everywhere pattern). |
| D2 | `release.yml` shape | **Two separate workflows**, both `workflow_dispatch`-only, each with `dry_run: bool` input (default `true`). |
| D3 | CHANGELOG date timing | Date flip is **not** WS8 work. `## 6.0.0 (unreleased)` heading stays. |
| D4 | crates.io publish ownership | Owner has publish rights + token. Token wired to canonical only, not the fork, at final-release time. |
| D5 | Merge strategy | **Merge commit** (`--no-ff` / "Create a merge commit") for both PR #1 (fork-internal, this WS) and PR #2 (canonical, future). Preserves per-WS commits and backlog-PR `--author` headers. |
| D6 | Tracked-deferred fold-ins | **In WS8**: Node.js actions bump, `paste` deny.toml ignore + tracked-deferred follow-up, `samples/` deletion (+ `structopt` removal). **Not in WS8**: full `paste` rewrite/swap investigation, PR #277 wrapper-soundness refactor, gamepad transmute (fold opportunistically if input.rs is touched — it's not in WS8). |
| D7 | Dry-run rehearsal protocol | `act`-based validation in WS8c is the rehearsal. No fork-CI rehearsal, no pre-publishing of `raylib-sys` for `raylib` dep resolution. |
| D8 | GitHub release notes | Not applicable to WS8 (future step). |

## 4. Sub-step plan

WS8 splits into 5 sub-steps, each landing on `6.0-rc` as one commit (or one tight commit cluster). Each gets its own plan file at `docs/superpowers/plans/2026-05-29-ws8N-*.md`.

### WS8a — version bump + book snippet sync

- `raylib/Cargo.toml`: `version = "5.7.0"` → `"6.0.0"` (line ~3); inter-crate dep `raylib-sys = { version = "5.7.0", ... }` → `"6.0.0"` (line 17).
- `raylib-sys/Cargo.toml`: `version = "5.7.0"` → `"6.0.0"` (line ~3).
- `book/src/getting-started/quickstart.md`: `raylib = "5.7"` snippet → `"6.0"`.
- Grep sweep: `book/src/getting-started/install-*.md` for any version references. Update opportunistically.
- `CHANGELOG.md` heading: **no change**. `## 6.0.0 (unreleased)` stays.
- `Cargo.lock`: gitignored (per `.gitignore`); no commit needed.

**Verification**: `cargo build --workspace`, `cargo test -p raylib`, `mdbook build book`, `mdbook test book -L target/debug/deps`. Quick sanity check on the doctests since the version string appears in the quickstart.

### WS8b — `release-sys.yml` + `release-safe.yml` + `scripts/check-release-sync.sh`

Two workflows, identical shape, parameterized by crate name. Skeleton:

```yaml
# .github/workflows/release-sys.yml
name: Release raylib-sys
on:
  workflow_dispatch:
    inputs:
      dry_run:
        description: "Dry-run only (no cargo publish)"
        type: boolean
        default: true
jobs:
  publish-sys:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with: { submodules: recursive }
      - uses: dtolnay/rust-toolchain@stable
      - name: Version + CHANGELOG sync check
        run: bash scripts/check-release-sync.sh raylib-sys ${{ inputs.dry_run == false && '--require-date' || '' }}
      - name: cargo publish (dry-run)
        run: cargo publish -p raylib-sys --dry-run
      - name: cargo publish
        if: ${{ inputs.dry_run == false }}
        env: { CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_IO_TOKEN }} }
        run: cargo publish -p raylib-sys
```

`release-safe.yml` is identical with `raylib-sys` replaced by `raylib`. Plan WS8b records the parameterization mechanism (two near-identical YAML files vs reusable workflow) and picks the simpler option (two files, less ceremony for two-call CI).

**`scripts/check-release-sync.sh`** — repo-local helper, single file, no dependencies beyond `bash` + `cargo` + `jq` + `grep`:

```bash
#!/usr/bin/env bash
# Usage: check-release-sync.sh <crate-name> [--require-date]
# Verifies:
#   1. Cargo.toml version matches CHANGELOG.md latest heading version.
#   2. If --require-date: CHANGELOG heading must include an ISO date (rejects "(unreleased)").
#   3. For raylib: the raylib-sys dep version-req matches the actual raylib-sys version.
set -euo pipefail
crate="$1"; require_date="${2:-}"
manifest_version=$(cargo metadata --no-deps --format-version 1 \
  | jq -r ".packages[] | select(.name == \"$crate\") | .version")
changelog_version=$(grep -oE '^## [0-9]+\.[0-9]+\.[0-9]+' CHANGELOG.md | head -1 | awk '{print $2}')
[ "$manifest_version" = "$changelog_version" ] \
  || { echo "Version mismatch: Cargo.toml=$manifest_version, CHANGELOG=$changelog_version"; exit 1; }
if [ "$require_date" = "--require-date" ]; then
  grep -qE '^## [0-9.]+ — [0-9]{4}-[0-9]{2}-[0-9]{2}' CHANGELOG.md \
    || { echo "CHANGELOG heading still says (unreleased) — date must be set before real publish"; exit 1; }
fi
if [ "$crate" = "raylib" ]; then
  sys_req=$(grep -E '^raylib-sys = .*version = "[0-9.]+"' raylib/Cargo.toml \
    | grep -oE '"[0-9.]+"' | head -1 | tr -d '"')
  [ "$sys_req" = "$manifest_version" ] \
    || { echo "raylib-sys dep pin $sys_req != raylib version $manifest_version"; exit 1; }
fi
```

**Notes on the workflow design**:
- `default: true` on `dry_run` forces a conscious toggle-off before real publish.
- The sync-check uses `--require-date` **only** when `dry_run == false`, so `act` validation passes with the CHANGELOG still saying `(unreleased)`.
- The `cargo publish --dry-run` step always runs (catches packaging errors); the real `cargo publish` step is gated by `if: ${{ inputs.dry_run == false }}`.
- `CRATES_IO_TOKEN` is only consumed when `dry_run: false`. The secret is **not** added to the fork; it lives only on canonical, added at final-release time.

### WS8c — validate workflows with `act`

Five commands, run from the repo root:

```bash
# 1. Dry-run on release-sys — must succeed.
gh act workflow_dispatch -W .github/workflows/release-sys.yml \
  --input dry_run=true \
  -s CARGO_REGISTRY_TOKEN=dummy

# 2. Dry-run on release-safe — expected to fail at `cargo publish --dry-run`
#    because raylib-sys 6.0.0 isn't on crates.io. Validates everything up to
#    that step (YAML parses, sync-check runs, packaging starts).
gh act workflow_dispatch -W .github/workflows/release-safe.yml \
  --input dry_run=true \
  -s CARGO_REGISTRY_TOKEN=dummy

# 3. Negative test on release-sys with dry_run=false + dummy token —
#    must fail at the real publish step (auth error), NOT earlier.
gh act workflow_dispatch -W .github/workflows/release-sys.yml \
  --input dry_run=false \
  -s CARGO_REGISTRY_TOKEN=dummy

# 4. Sync-check direct run, no act — fastest feedback during iteration.
bash scripts/check-release-sync.sh raylib-sys
bash scripts/check-release-sync.sh raylib
# Both should pass (no --require-date).
bash scripts/check-release-sync.sh raylib-sys --require-date
# Should FAIL because CHANGELOG still says (unreleased) — confirms gate works.
```

**Known limitations recorded in `docs/superpowers/notes/ws8c-validation.md`**:
- `release-safe.yml`'s `cargo publish --dry-run -p raylib` will fail at registry resolution until `raylib-sys 6.0.0` is published for real. This is by design (no pre-publishing in WS8). The workflow shape is still validated up to that step. Full end-to-end verification happens during final-release.
- `act` requires Docker. If Docker can't run on the dev machine, the fallback is the direct bash sequence (item 4 above) plus manual YAML lint via `actionlint` if available.
- `actions/checkout@v5` with `submodules: recursive` is slow on the first `act` run (raylib C source is a large submodule).

**Exit criteria for WS8c**:
- Items 1, 3, and 4 produce the expected outcomes.
- Item 2's failure is documented as the known limitation, with the step it fails at recorded for the final-release runbook.
- `docs/superpowers/notes/ws8c-validation.md` exists with the validation commands + any `act`-specific gotchas hit.

### WS8d — release-hygiene fold-ins

Three commits, one per fold-in, so PR #1 review can address each independently.

**Fold-in 1 — Node.js actions bump.** Across all 5 existing workflows (the 2 new release workflows from WS8b are authored on the latest action versions from the start, so this fold-in only touches the legacy 5):
- `actions/checkout@v4` → `@v5`.
- `peaceiris/actions-mdbook@v2` — verify Node.js 24 status; bump if needed.
- Grep for any other action on Node.js 20 and bump opportunistically (`dtolnay/rust-toolchain`, `Swatinem/rust-cache`, etc.).
- Verification: push to fork, wait for the 5 CI workflows to re-run, confirm green + no deprecation warnings in run logs.

**Fold-in 2 — `paste` cargo-deny advisory.** Owner direction: skip the inspection, add the ignore now, defer the rewrite/swap to a future workstream.
- Add to `deny.toml` under `[advisories]`:
  ```toml
  ignore = [
    # paste 1.0 is unmaintained (RUSTSEC-XXXX-YYYY). Used by
    # raylib/src/core/callbacks/stream_processor_with_user_data_wrapper.rs.
    # No known soundness issue; rewrite or swap deferred — see tracked-deferred.
    { id = "RUSTSEC-XXXX-YYYY", reason = "paste is unmaintained; rewrite/swap tracked-deferred to a future workstream" },
  ]
  ```
  (Replace `RUSTSEC-XXXX-YYYY` with the actual ID when WS8d looks it up.)
- Add to CHANGELOG `### Internal` (or `### Fixed`): "Accepted `paste 1.0` cargo-deny unmaintained advisory with rationale; rewrite/swap tracked for a future workstream."
- Verification: `cargo deny check` clean.

**Fold-in 3 — delete `samples/`.** Biggest behavior change in WS8.
1. `git rm -r samples/`.
2. `raylib/Cargo.toml`: remove `structopt = "0.3"` from `[dev-dependencies]`. Confirm by `grep -r "use structopt" raylib/` — should produce zero matches.
3. `README.md`: replace the "Run a sample" section with a "See `showcase/`" pointer + a note that the showcase port is in progress (WS9).
4. `CLAUDE.md`: update the Workspace layout bullet — current text says "Being retired — WS9 folds these into `showcase/`"; flip to "Removed in 6.0; see `showcase/` for runnable examples."
5. `CHANGELOG.md`: extend the `### Breaking` block with "Removed: `samples/` directory. Use `showcase/` instead; full example port completes in 6.0.x." Also note `structopt` dev-dep dropped (advisory auto-resolves).
6. Grep sweep: `grep -rni "samples/" --include="*.md" --include="*.toml" --include="*.rs" .` — clean any stragglers.
7. Verification: `cargo build --workspace`, `cargo test -p raylib`, `cargo deny check`, all 5 + 2 CI workflows green on the fork.

### WS8e — fork PR (the checkpoint)

1. Push `6.0-rc` to the fork (already there; final commits from WS8a-d land here).
2. Create the PR: `gh pr create -R Dacode45/ms-raylib-rs --base unstable --head 6.0-rc --title "raylib-rs 6.0 — release prep checkpoint" --body "$(...)"`. PR body summarizes WS1–WS8 with one paragraph each, linking the `docs/superpowers/notes/wsN-complete.md` files.
3. **Owner reviews exhaustively.** No time pressure — this is the checkpoint.
4. Address review feedback as new commits on `6.0-rc`. Force-push of the PR branch is allowed if needed for fixups, but prefer additive commits where possible (preserves review-thread continuity).
5. When owner is satisfied, merge via the GitHub UI selecting "Create a merge commit" (`--no-ff`). Do **not** select squash or rebase-and-merge.

**WS8 done-line**: PR #1 merged into `Dacode45:unstable`. The fork's `unstable` HEAD is the "ready for evaluation" snapshot.

## 5. CI surface after WS8

Existing 5 workflows continue running on every push to `6.0-rc` and on PR open/update:
- `check.yml`, `test.yml`, `web.yml`, `sanitizers.yml`, `book.yml`.

Two new workflows added in WS8b that **never run automatically** (`workflow_dispatch`-only):
- `release-sys.yml`, `release-safe.yml`.

After the WS8e merge, all 5 + 2 are present on `Dacode45:unstable`. The 5 auto-running ones must be green at the merge commit; the 2 manual ones are never triggered on the fork.

## 6. Dependencies + working model

- All work on `6.0-rc`. Push to `fork` (`Dacode45/ms-raylib-rs`). Do not push to `origin/unstable` or `origin/master` at any point in WS8.
- `gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status` is the standard CI watch command (matches WS6/WS7 pattern).
- Do not `git add -A` — the repo root carries untracked working files (`TODO.md`, `prompt.md`, `next-session-prompt.md`) that must stay untracked.
- Each WS8 sub-step ends with a commit ending in `Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>`.
- Plans live under `docs/superpowers/plans/`. Plan filenames: `2026-05-29-ws8a-version-bump.md`, `…ws8b-release-workflows.md`, `…ws8c-act-validation.md`, `…ws8d-release-hygiene.md`, `…ws8e-fork-pr-checkpoint.md`.

## 7. Risks + mitigations

1. **`act` can't model `workflow_dispatch` inputs cleanly.** Mitigation: WS8c documents the fallback direct-bash sequence as the alternative validation path. The workflow shape is correct either way; `act` only adds confidence the YAML parses.
2. **`samples/` deletion catches an unanticipated reference.** Mitigation: the grep sweep in WS8d Fold-in 3 plus CI on the fork should catch it. If something's missed and surfaces during PR #1 review, fix as a new commit on `6.0-rc`.
3. **`paste` ignore entry is too permissive.** Mitigation: the entry is narrowly scoped to the specific RUSTSEC advisory ID and carries a tracked-deferred follow-up. A future audit can revisit.
4. **`actions/checkout@v5` has a breaking change vs `@v4`.** Mitigation: WS8d Fold-in 1 verifies via the fork's CI re-run; if anything breaks, revert that specific bump and document.
5. **Owner review at WS8e identifies large-scale issues.** Expected — that's the checkpoint's purpose. Mitigation: address as new commits on `6.0-rc`; if scope grows substantially, write a new workstream (WS8.5 / WS10) rather than ballooning WS8.

## 8. Tracked-deferred follow-ups carried into the future

Existing (from WS6/WS7):
1. Full PR #277 wrapper-soundness refactor.
2. `get_gamepad_button_pressed` transmute (`raylib/src/core/input.rs`).
3. `raylib-test` delete-or-fix — revisit at WS9.
4. `rlsw` on wasm32 — `build.rs` `platform_from_target` reordering.
5. UBSAN through the FFI boundary — informational.
6. Full rustdoc rewrite of the remaining ~200 stub-level items.
7. Public Pages deploy → WS9.
8. Custom book theme / brand styling → WS9.
9. Safe abstractions for `GuiGetIcons`/`GuiLoadIcons` + PR #296.
10. End-to-end showcase examples → WS9.

New (from WS8):
11. **`paste` macro rewrite or library swap** — `raylib/src/core/callbacks/stream_processor_with_user_data_wrapper.rs` uses `paste!`. The `deny.toml` ignore in WS8d is a stopgap. A future workstream should investigate (a) rewriting the macros without `paste` or (b) swapping to a maintained alternative (`pastey`, `paste2`, etc.).

Carried but **not** in WS8:
- CHANGELOG date flip → final-release step.
- `v6.0.0` tag → final-release step.
- Manual `release-sys.yml` + `release-safe.yml` run with `dry_run: false` → final-release step.
- PR #2 `Dacode45:unstable` → `raylib-rs:unstable` → final-release step.
- GitHub release creation → final-release step.

## 9. Future "final-release" workstream (out of WS8 scope)

After WS9 (showcase rewrite) and any additional work the owner adds at the WS8e checkpoint, a final-release workstream will:

1. Verify the fork's `unstable` is still green on all CI workflows.
2. Flip `CHANGELOG.md` heading `## 6.0.0 (unreleased)` → `## 6.0.0 — YYYY-MM-DD` (actual release date). Commit on `Dacode45:unstable` directly.
3. Open PR #2: `Dacode45:unstable` → `raylib-rs:unstable`. Final review by owner. Merge with "Create a merge commit".
4. Tag `v6.0.0` on canonical `unstable` HEAD. Push tag to canonical.
5. Add `CRATES_IO_TOKEN` to canonical repo secrets if not already present.
6. From canonical Actions: manually trigger `release-sys.yml` with `dry_run: true` first to confirm clean. Then `dry_run: false` for the real publish.
7. Verify on crates.io: `cargo search raylib-sys` shows `6.0.0`.
8. From canonical Actions: manually trigger `release-safe.yml` with `dry_run: true` (now this WILL pass since `raylib-sys 6.0.0` exists), then `dry_run: false`.
9. Verify on crates.io: `cargo search raylib` shows `6.0.0` resolving to `raylib-sys 6.0.0`.
10. Create GitHub release on canonical for `v6.0.0`, body links the `## 6.0.0 — YYYY-MM-DD` CHANGELOG block + a one-line summary.

This spec records the future shape; the final-release workstream gets its own design doc when the owner is ready to ship.

## 10. Done-criteria for WS8

WS8 is complete when **all** of:

- [ ] `raylib/Cargo.toml` version = `6.0.0`, `raylib-sys` dep pin = `6.0.0`.
- [ ] `raylib-sys/Cargo.toml` version = `6.0.0`.
- [ ] `book/src/getting-started/quickstart.md` shows `raylib = "6.0"`.
- [ ] `book/src/getting-started/install-*.md` references are version-consistent.
- [ ] `CHANGELOG.md` `## 6.0.0 (unreleased)` block intact (date flip deferred).
- [ ] `.github/workflows/release-sys.yml` exists and validates clean under `act` with `dry_run=true`.
- [ ] `.github/workflows/release-safe.yml` exists; YAML + sync-check validate under `act` with the documented `cargo publish --dry-run -p raylib` registry-resolution caveat.
- [ ] `scripts/check-release-sync.sh` exists and passes for both crates without `--require-date`; fails with `--require-date` (proves the gate).
- [ ] All 5 existing CI workflows + the 2 new ones present on `6.0-rc`. The 5 auto-running are green.
- [ ] All 5 + 2 workflows use Node.js 24 actions (no deprecation warnings in run logs).
- [ ] `deny.toml` has the rationale'd `paste` ignore entry; `cargo deny check` clean.
- [ ] `samples/` directory removed.
- [ ] `structopt` dev-dependency removed from `raylib/Cargo.toml`.
- [ ] `README.md` + `CLAUDE.md` updated to point at `showcase/` instead of `samples/`.
- [ ] `CHANGELOG.md` `### Breaking` includes the `samples/` removal note; `### Internal`/`### Fixed` notes the `paste` advisory acceptance and the `structopt` removal.
- [ ] `docs/superpowers/notes/ws8c-validation.md` exists with the validation commands + gotchas.
- [ ] PR #1 (`Dacode45:6.0-rc` → `Dacode45:unstable`) opened, reviewed by owner, merged with "Create a merge commit". Fork's `unstable` HEAD = the merge commit.
- [ ] Tracked-deferred list updated with the `paste` rewrite/swap follow-up.
- [ ] `CLAUDE.md` status line flipped: WS7 ✅ · **WS8 ✅** · WS9 ← NEXT.
- [ ] `docs/superpowers/notes/ws8-complete.md` written.

**Out of done-criteria (future)**: crates.io publication, `v6.0.0` tag, canonical merge, GitHub release.
