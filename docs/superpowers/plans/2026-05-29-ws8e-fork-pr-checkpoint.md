# WS8e — Fork PR checkpoint Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Open PR #1 from `Dacode45:6.0-rc` to `Dacode45:unstable` so the owner can review every WS1–WS8 diff before any work advances. Merge with "Create a merge commit" preserving all per-WS commits and backlog-PR `--author` headers. Update the `CLAUDE.md` status line to ✅ WS8, and write `docs/superpowers/notes/ws8-complete.md` as the WS8 done-note.

**Architecture:** GitHub PR opens for owner review. Review feedback lands as new commits on `6.0-rc` (force-push avoided where additive commits suffice; never amend a commit already in PR history). Final merge is via the GitHub UI selecting "Create a merge commit" (`--no-ff` semantics).

**Tech Stack:** `gh` CLI, GitHub UI.

**Spec reference:** `docs/superpowers/specs/2026-05-29-ws8-release-prep-checkpoint-design.md` §4 WS8e.

---

### Task 1: Verify pre-PR state on `6.0-rc` and `Dacode45:unstable`

**Files:** none (precondition check)

- [ ] **Step 1: Confirm we're on `6.0-rc` at the WS8d HEAD**

Run: `git rev-parse --abbrev-ref HEAD && git log -1 --pretty='%h %s'`
Expected: branch `6.0-rc`; HEAD commit is the last WS8d commit (Fold-in 3 — samples removal).

- [ ] **Step 2: Confirm `6.0-rc` is pushed to fork and CI is green**

Run: `git fetch fork && git log fork/6.0-rc..HEAD --oneline 2>&1 | head -5`
Expected: zero output (fork is up to date with local `6.0-rc`). If non-empty, push: `git push fork 6.0-rc`.

Run: `gh run list -R Dacode45/ms-raylib-rs -L 5 --json status,conclusion,workflowName,headSha --jq '.[] | "\(.workflowName): \(.status) / \(.conclusion)"'`
Expected: latest 5 runs (or however many ran for the most recent push) all show `completed / success`.

- [ ] **Step 3: Confirm `Dacode45:unstable` exists as the target branch**

Run: `git ls-remote fork unstable 2>&1`
Expected: prints the current `Dacode45:unstable` SHA. If the branch doesn't exist on the fork, create it from canonical's `unstable`:

```bash
git fetch origin unstable
git push fork origin/unstable:refs/heads/unstable
```

- [ ] **Step 4: Confirm the PR base is reachable**

Run: `git log --oneline fork/unstable..fork/6.0-rc 2>&1 | wc -l`
Expected: a number of commits ahead — the WS1–WS8 commits we're shipping. (This is the size of PR #1.)

### Task 2: Construct the PR body

**Files:**
- Create: `/tmp/ws8e-pr-body.md` (transient — only used by `gh pr create`)

- [ ] **Step 1: Write the PR body to a temp file**

Create `/tmp/ws8e-pr-body.md` with this content:

```markdown
## raylib-rs 6.0 — release prep checkpoint

This PR bundles **WS1–WS8** of the raylib 5.x → 6.0 upgrade for owner review.
It is a **checkpoint, not a release** — no crates.io publish, no canonical
merge, no `v6.0.0` tag. After this PR merges, the fork's `unstable` HEAD is
the snapshot the owner evaluates before WS9 (showcase rewrite) and any
further work they choose to add.

### Workstream summary

- **WS0** ✅ Baseline + roadmap — `docs/superpowers/specs/2026-05-25-raylib-rs-6.0-roadmap-design.md`.
- **WS1** ✅ raylib-sys at 6.0 parity (3-OS CI green) — `docs/superpowers/notes/ws1-config-reconcile.md`.
- **WS2a** ✅ Native math types (bindgen-generated `Vector2/3/4`, `Matrix`, distinct `Quaternion`).
- **WS2b** ✅ Math-ecosystem adapters opt-in only; `mint`/`glam`/`serde` no longer default.
- **WS3** ✅ Safe crate green vs 6.0; `ModelAnimations` RAII; Tier-1 tests — `docs/superpowers/notes/ws3-complete.md`.
- **WS4** ✅ Software-renderer + headless render-test harness — `docs/superpowers/notes/ws4b-complete.md`.
- **WS5** ✅ raygui 6.0 parity (57/57 fns) + safe `rlgl` immediate-mode module — `docs/superpowers/notes/ws5-complete.md`.
- **WS6** ✅ Platform matrix + layered CI/CD + quality hard-gates + MSRV 1.85 — `docs/superpowers/notes/ws6b-complete.md`.
- **WS7** ✅ 28-chapter mdBook + ~25 rustdoc enrichments + CHANGELOG 6.0.0 entry + backlog #284/#273 folded — `docs/superpowers/notes/ws7-complete.md`.
- **WS8** ✅ Version bump + draft release workflows (`release-sys.yml`, `release-safe.yml` — workflow_dispatch-only, dry_run-gated) + `act` validation + release-hygiene fold-ins (Node.js 24 actions, `paste` advisory acceptance, `samples/` removal) — `docs/superpowers/notes/ws8-complete.md` (added in this PR).

### What's deliberately NOT in this PR

- **No publish to crates.io.** `release-sys.yml` and `release-safe.yml` are authored but never auto-triggered. Real publish is the future final-release step.
- **No CHANGELOG date flip.** `## 6.0.0 (unreleased)` heading stays; the date is set at real publish time.
- **No `v6.0.0` tag.** Tagging is part of the final-release step.
- **No PR to `raylib-rs/raylib-rs`.** The canonical merge happens only after WS9 and any added work.

### Review focus areas

- Per-WS commit grouping is preserved. Each WS lands as a sequence of small commits with `Co-Authored-By: Claude Opus 4.7` attribution. Backlog-PR cherry-picks (PR #284 LBreede, PR #273 AmityWilder, PR #272 AmityWilder, etc.) keep the original `--author` headers.
- Locked WS8 decisions are recorded in the spec §3.
- Tracked-deferred items (see WS8 spec §8 + each `wsN-complete.md`) carry forward.

### CI surface

All 5 existing workflows + 2 new release workflows present on this branch. The 5 auto-running ones are green at the PR HEAD. The 2 new release workflows are `workflow_dispatch`-only and never auto-trigger.

### How to merge

Select **"Create a merge commit"** in the GitHub UI (`--no-ff` semantics). Do **not** select squash or rebase-and-merge — those destroy the per-WS commit groupings and backlog-PR attribution headers.
```

(If `/tmp` isn't writable on this platform, use a repo-local path that's gitignored, e.g. `.git/ws8e-pr-body.md`.)

- [ ] **Step 2: Spot-check the body**

Run: `head -30 /tmp/ws8e-pr-body.md`
Expected: starts with the H2 title and the leading paragraph.

### Task 3: Open PR #1 (`Dacode45:6.0-rc` → `Dacode45:unstable`)

**Files:** none (uses `gh pr create`)

- [ ] **Step 1: Open the PR**

Run:
```bash
gh pr create \
  --repo Dacode45/ms-raylib-rs \
  --base unstable \
  --head 6.0-rc \
  --title "raylib-rs 6.0 — release prep checkpoint" \
  --body-file /tmp/ws8e-pr-body.md
```

Expected: prints the PR URL. Capture the URL for downstream steps.

- [ ] **Step 2: Confirm PR exists**

Run: `gh pr list -R Dacode45/ms-raylib-rs --head 6.0-rc --json number,title,url`
Expected: one entry showing the new PR with title `raylib-rs 6.0 — release prep checkpoint`.

### Task 4: Owner review window (no automated action — wait for owner)

**Files:** none (owner activity)

This is the checkpoint's primary purpose: the owner reviews every diff. No automated work happens here. Mark this task complete when the owner signals (in conversation or PR comments) that review is done and approved.

- [ ] **Step 1: Capture review-status snapshot before any merge**

Run: `gh pr view <PR-number> -R Dacode45/ms-raylib-rs --json state,reviews,mergeable --jq '{state, mergeable, reviews: .reviews | length}'`
Expected: `state: OPEN`, `mergeable: MERGEABLE` (assuming CI is green; if `MERGEABLE_UNKNOWN`, re-check after CI completes).

- [ ] **Step 2: When owner gives approval, proceed to Task 5 (review fixes if any) or Task 6 (merge)**

If owner requests no changes: skip Task 5, go to Task 6.
If owner requests changes: do Task 5 first.

### Task 5: Address review feedback (if any) as additive commits

**Files:** depends on review feedback

- [ ] **Step 1: For each piece of feedback, create a new commit on `6.0-rc`**

```bash
# After making the edit:
git add <specific files>
git commit -m "fix(ws8e-review): <one-line description of what was changed>

<more detail if needed>

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

Preference: **additive commits over amend / force-push.** PR review threads anchor to commits; amending breaks the anchors. The exception is a typo or trivial fix to the most recent commit that has no review comments — that's safe to amend.

- [ ] **Step 2: Push to fork after each fix**

Run: `git push fork 6.0-rc`
Expected: clean push. The PR auto-updates.

- [ ] **Step 3: Re-verify CI**

Wait for the fork's CI to re-run on the new HEAD; confirm all 5 workflows are green before re-requesting review.

- [ ] **Step 4: Re-request review**

Either in conversation or via the GitHub UI's "Re-request review" button.

Repeat Task 5 until the owner has no more change requests.

### Task 6: Merge the PR via "Create a merge commit"

**Files:** none (uses `gh pr merge` or the GitHub UI)

- [ ] **Step 1: Final pre-merge check**

Run: `gh pr view <PR-number> -R Dacode45/ms-raylib-rs --json state,mergeable,reviewDecision,statusCheckRollup --jq '{state, mergeable, reviewDecision, checks: [.statusCheckRollup[].conclusion]}'`
Expected: `state: OPEN`, `mergeable: MERGEABLE`, `reviewDecision: APPROVED`, all checks `SUCCESS`.

- [ ] **Step 2: Merge with merge-commit strategy**

Option A — via `gh`:
```bash
gh pr merge <PR-number> -R Dacode45/ms-raylib-rs --merge --body "Merge raylib-rs 6.0 release prep checkpoint (WS1-WS8)"
```

(`--merge` selects "Create a merge commit". `--squash` and `--rebase` are the alternatives we explicitly reject.)

Option B — via the GitHub UI:
1. Open the PR URL.
2. Click "Merge pull request" dropdown.
3. Select **"Create a merge commit"**.
4. Confirm merge.

- [ ] **Step 3: Verify the merge commit exists**

Run: `git fetch fork && git log fork/unstable --oneline -5`
Expected: top commit is a merge commit (two parents), with a message like "Merge pull request #N from Dacode45/6.0-rc". Below it are the WS8 commits (samples removal, paste advisory, Node.js bump, etc.) — they're reachable from `unstable` now.

Run: `git show fork/unstable --no-patch --pretty='%h %p %s'`
Expected: the merge commit shows TWO parents (`%p` prints two SHAs). Confirms `--no-ff` semantics.

### Task 7: Update CLAUDE.md status line (WS7 ✅ → WS8 ✅)

**Files:**
- Modify: `CLAUDE.md` (the status-line text in the "raylib 6.0 upgrade" section)

- [ ] **Step 1: Switch back to `6.0-rc` if needed and pull the merge**

Run: `git checkout 6.0-rc && git pull fork unstable 2>&1 | tail -5`
Expected: fast-forward or merge — `6.0-rc` now contains the merge commit too (or its parent state is identical to what's on `fork/unstable` minus the merge commit). The cleanest move is to do the next edits ON the merged `unstable` directly; see Step 2 alternative.

Alternative: do the next edits ON `fork/unstable` directly.

```bash
git checkout -B post-ws8 fork/unstable  # working branch from the merge
```

- [ ] **Step 2: Find the status line in CLAUDE.md**

Run: `grep -n '^\*\*Workstreams:\*\*' CLAUDE.md`
Expected: one line, likely near the end of the "raylib 6.0 upgrade" section, listing WS0 ✅ · WS1 ✅ · ... · WS7 ✅ ... · WS8 release ← NEXT · WS9 showcase → ...

- [ ] **Step 3: Edit the status line**

Change the WS8 entry from `**WS8 release ← NEXT**` (or whatever the current pending phrasing is) to:

```
WS8 ✅ — **release prep checkpoint done: version bumped 5.7.0 → 6.0.0, release-sys.yml + release-safe.yml drafted (workflow_dispatch-only, dry_run-gated), act-validated, samples/ removed in favor of showcase/, Node.js 24 actions, paste advisory accepted (see `docs/superpowers/notes/ws8-complete.md`; tracked-deferred: paste rewrite/swap, plus the WS6/WS7 carryover list)**
```

Update the trailing entries so the new pending workstream is `**WS9 showcase ← NEXT**` (or whatever the canonical next-step phrasing is — match the prior `← NEXT` style).

- [ ] **Step 4: Verify the edit**

Run: `grep -A1 'WS8' CLAUDE.md | head -5`
Expected: shows the new ✅ WS8 line.

### Task 8: Write `docs/superpowers/notes/ws8-complete.md`

**Files:**
- Create: `docs/superpowers/notes/ws8-complete.md`

- [ ] **Step 1: Inspect the WS7 done-note for shape reference**

Run: `head -40 docs/superpowers/notes/ws7-complete.md`
Expected: shows the WS7 done-note structure (title, status line, spec reference, per-sub-step sections, "Done-criteria sign-off", "Tracked-deferred follow-ups").

- [ ] **Step 2: Write the WS8 done-note**

Create `docs/superpowers/notes/ws8-complete.md` modeled on `ws7-complete.md`. Outline:

```markdown
# WS8 complete — release prep + checkpoint

**Status:** DONE on branch `Dacode45:unstable` (fork only; canonical merge deferred). PR #<N> merged with merge-commit at <SHA>.

Spec: `docs/superpowers/specs/2026-05-29-ws8-release-prep-checkpoint-design.md`.
Plans: `docs/superpowers/plans/2026-05-29-ws8a-version-bump.md`, `…ws8b-release-workflows.md`, `…ws8c-act-validation.md`, `…ws8d-release-hygiene.md`, `…ws8e-fork-pr-checkpoint.md`.

Builds on WS7 (`docs/superpowers/notes/ws7-complete.md`) — 28-chapter mdBook + CHANGELOG 6.0.0 entry + 5 CI workflows green.

---

## WS8a — version bump

**Commit:** `<SHA>` — `feat(ws8a): bump raylib + raylib-sys to 6.0.0 + sync book snippets`.

What shipped:
- `raylib/Cargo.toml` version 5.7.0 → 6.0.0; inter-crate dep pin synced.
- `raylib-sys/Cargo.toml` version 5.7.0 → 6.0.0.
- `book/src/getting-started/quickstart.md` snippet `raylib = "5.7"` → `"6.0"`.
- CHANGELOG heading deliberately stays `## 6.0.0 (unreleased)` — date flip is the future final-release step's job.

---

## WS8b — release workflows

**Commit:** `<SHA>` — `feat(ws8b): draft release-sys.yml + release-safe.yml + sync-check helper`.

What shipped:
- `.github/workflows/release-sys.yml` and `release-safe.yml` — both `workflow_dispatch`-only with `dry_run: bool` input (default `true`).
- `scripts/check-release-sync.sh` — enforces version/CHANGELOG consistency; `--require-date` gate is the safety net for real publish (only fired when `dry_run: false`).
- Real publish gated behind `if: ${{ inputs.dry_run == false }}` so `CARGO_REGISTRY_TOKEN` is only consumed during intentional real publishes.

---

## WS8c — act validation

**Commit:** `<SHA>` — `docs(ws8c): validate release workflows via act + record runbook notes`.

What shipped:
- `docs/superpowers/notes/ws8c-validation.md` — three documented `act` runs (`release-sys + dry_run=true` → SUCCESS; `release-safe + dry_run=true` → fails at cargo step, documented limitation until raylib-sys 6.0.0 exists; `release-sys + dry_run=false` + dummy token → fails at sync-check gate, real publish step never reached).
- Fallback direct-validation sequence for maintainers without `act`.
- Final-release runbook teaser.

---

## WS8d — release-hygiene fold-ins

**Commits:**
- `<SHA-1>` — `chore(ws8d): bump Node.js-20 actions to Node.js-24 versions` (actions/checkout@v4 → @v5 across 5 legacy workflows).
- `<SHA-2>` — `chore(ws8d): accept paste cargo-deny advisory with rationale` (deny.toml ignore entry + CHANGELOG note).
- `<SHA-3>` — `feat(ws8d)!: remove samples/ — superseded by showcase/` (samples/ deletion + structopt dev-dep drop + README/CLAUDE.md/CHANGELOG updates).

---

## WS8e — fork PR checkpoint

PR #<N> opened `Dacode45:6.0-rc` → `Dacode45:unstable`. Owner-reviewed line-by-line. Merged with "Create a merge commit" (`--no-ff`) preserving per-WS commit groupings and backlog-PR `--author` headers.

CLAUDE.md status line flipped: WS7 ✅ → WS8 ✅; WS9 showcase ← NEXT.

---

## Final CI inventory (fork's unstable at merge HEAD)

| Workflow | Status |
|----------|--------|
| `check.yml` | ✅ |
| `test.yml` | ✅ |
| `web.yml` | ✅ |
| `sanitizers.yml` | ✅ |
| `book.yml` | ✅ |
| `release-sys.yml` | ⚪ workflow_dispatch-only; act-validated |
| `release-safe.yml` | ⚪ workflow_dispatch-only; act-validated with known limitation |

---

## WS8 done-criteria sign-off (spec §10)

- ✅ `raylib/Cargo.toml` + `raylib-sys/Cargo.toml` at 6.0.0; dep pin matches.
- ✅ Book quickstart shows `raylib = "6.0"`.
- ✅ CHANGELOG `## 6.0.0 (unreleased)` intact (date flip deferred).
- ✅ `release-sys.yml` + `release-safe.yml` exist; `act`-validated.
- ✅ `scripts/check-release-sync.sh` works for both crates without `--require-date`; fails with `--require-date`.
- ✅ All 5 existing CI workflows + 2 release workflows present.
- ✅ Node.js 24 actions across all 5 legacy workflows.
- ✅ `deny.toml` has rationale'd `paste` ignore entry; `cargo deny check` clean.
- ✅ `samples/` removed; `structopt` dev-dep gone; README/CLAUDE.md/CHANGELOG updated.
- ✅ `docs/superpowers/notes/ws8c-validation.md` exists (no placeholders).
- ✅ PR #<N> merged with "Create a merge commit"; fork's `unstable` HEAD is the merge commit.
- ✅ Tracked-deferred list updated; `paste` rewrite/swap on it.
- ✅ `CLAUDE.md` status line flipped: WS8 ✅, WS9 ← NEXT.
- ✅ This document (`ws8-complete.md`) written.

**Out of done-criteria (future)**: crates.io publication, `v6.0.0` tag, canonical merge, GitHub release.

---

## Tracked-deferred follow-ups (carried out of WS8)

From WS6/WS7 (unchanged):
1. Full PR #277 wrapper-soundness refactor.
2. `get_gamepad_button_pressed` transmute (`raylib/src/core/input.rs`).
3. `raylib-test` delete-or-fix — revisit at WS9.
4. `rlsw` on wasm32.
5. UBSAN through the FFI boundary.
6. Full rustdoc rewrite of remaining stub-level items.
7. Public Pages deploy → WS9.
8. Custom book theme / brand styling → WS9.
9. Safe abstractions for `GuiGetIcons`/`GuiLoadIcons` + PR #296.
10. End-to-end showcase examples → WS9.

New from WS8:
11. **`paste` macro rewrite or library swap** — investigate (a) rewriting macros without `paste` or (b) swapping to `pastey`/`paste2`. Stopgap is the WS8d `deny.toml` ignore entry.

Out of WS8, queued for future final-release workstream:
- CHANGELOG date flip.
- `v6.0.0` tag.
- Manual `release-sys.yml` + `release-safe.yml` real-publish runs.
- PR #2 `Dacode45:unstable` → `raylib-rs:unstable`.
- GitHub release creation.

---

WS8 done. Next: **WS9 (showcase)** — port all of raylib's examples to Rust, build them on desktop + wasm where applicable, assemble the GitHub Pages site (book + showcase gallery), and deploy it.
```

Fill in `<SHA>` / `<SHA-1>` / `<SHA-2>` / `<SHA-3>` / `<N>` with the actual values from the merge.

- [ ] **Step 3: Spot-check no `<SHA>` / `<N>` placeholders remain**

Run: `grep -n '<SHA\|<N>' docs/superpowers/notes/ws8-complete.md`
Expected: zero matches.

### Task 9: Commit + push the final post-merge updates

**Files:**
- Modify: `CLAUDE.md`
- Create: `docs/superpowers/notes/ws8-complete.md`

- [ ] **Step 1: Confirm working-tree contents**

Run: `git status --short`
Expected: `M CLAUDE.md` and `?? docs/superpowers/notes/ws8-complete.md`. No other modifications. Untracked working files (`TODO.md`, `prompt.md`, `next-session-prompt.md`) remain untracked.

- [ ] **Step 2: Stage explicitly**

```bash
git add CLAUDE.md docs/superpowers/notes/ws8-complete.md
```

- [ ] **Step 3: Verify**

Run: `git diff --cached --stat`
Expected: 2 files; one modification, one new file.

- [ ] **Step 4: Commit**

```bash
git commit -m "$(cat <<'EOF'
docs(ws8): mark WS8 complete in CLAUDE.md + done-note

WS8 (release prep + checkpoint) is done on the fork's unstable branch.
PR #<N> merged with merge-commit at <SHA>.

CLAUDE.md status line: WS7 ✅ -> WS8 ✅; WS9 showcase <- NEXT.

docs/superpowers/notes/ws8-complete.md is the WS8 done-note,
modeled on the prior wsN-complete.md files; records the five
sub-step commits, the CI surface at the merge HEAD, and the
done-criteria sign-off.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

Replace `<N>` and `<SHA>` with the actual values.

- [ ] **Step 5: Push the final commits**

Run: `git push fork 6.0-rc:unstable` (if working from `6.0-rc`)
or: `git push fork post-ws8:unstable` (if working from the post-ws8 branch from Task 7)

Expected: clean push to `Dacode45:unstable` with the final commit.

- [ ] **Step 6: Sanity-check final state**

Run: `git log fork/unstable --oneline -8`
Expected: top entry is the `docs(ws8): mark WS8 complete` commit, then the merge commit (two parents), then the WS8d–WS8a commits, then the WS7 ones, etc.

---

## WS8e complete when

- PR #1 (`Dacode45:6.0-rc` → `Dacode45:unstable`) opened with the WS1–WS8 summary body.
- Owner has reviewed every diff (any feedback addressed as additive commits on `6.0-rc`).
- PR merged via "Create a merge commit" — fork's `unstable` HEAD is a two-parent merge commit.
- CLAUDE.md status line flipped: WS7 ✅ → WS8 ✅; WS9 ← NEXT.
- `docs/superpowers/notes/ws8-complete.md` written with no `<SHA>` / `<N>` placeholders.
- Final commit landed on `Dacode45:unstable`.

---

## WS8 done

The fork's `Dacode45:unstable` HEAD is the "ready for evaluation" snapshot. From here:
- WS9 (showcase rewrite + Pages deploy) is the next workstream.
- Final-release (crates.io publish + canonical merge + tag + GitHub release) waits until after WS9 + any further work the owner queues at this checkpoint.
