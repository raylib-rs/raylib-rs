# WS8d — Release-hygiene fold-ins Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Three release-hygiene fold-ins, each its own commit so PR #1 review can address them independently:
1. Node.js-20 actions bump across the 5 existing workflows.
2. `paste` cargo-deny advisory acceptance via rationale'd `deny.toml` ignore entry + tracked-deferred follow-up.
3. Delete `samples/`, remove `structopt` dev-dep (auto-resolves that advisory), update README/CLAUDE.md/CHANGELOG to point at `showcase/`.

**Architecture:** Three independent, small commits. Each verifies via either CI on the fork or local `cargo deny check`/`cargo build --workspace`.

**Tech Stack:** GitHub Actions YAML, TOML, Markdown, Rust + Cargo, cargo-deny.

**Spec reference:** `docs/superpowers/specs/2026-05-29-ws8-release-prep-checkpoint-design.md` §4 WS8d.

**Verified context** (from pre-plan grep):
- `actions/checkout@v4` appears 10 times across 5 workflows (`book.yml:20`, `check.yml:15,22,38,55,63`, `sanitizers.yml:23`, `test.yml:21,43,67,96`, `web.yml:17`).
- `dtolnay/rust-toolchain@{stable,1.85.0,nightly}` — these tag pins don't depend on Node.js version of the action; leave alone unless the runner image reports deprecation.
- `peaceiris/actions-mdbook@v2` appears once (`book.yml:30`); verify Node.js status during Fold-in 1.
- `structopt` is only imported by `samples/*.rs` (14 files); removing `samples/` makes it a clean dev-dep deletion.
- `deny.toml` has `[advisories] ... ignore = []` (empty) and `unmaintained = "none"` (the unmaintained-scope check is currently disabled, so `paste` isn't a CI blocker today — but the ignore entry is forward-defensive for when/if that scope is enabled).

---

## Fold-in 1 — Node.js-20 actions bump

### Task 1: Bump `actions/checkout` from `@v4` to `@v5` across all 5 existing workflows

**Files:**
- Modify: `.github/workflows/book.yml:20`
- Modify: `.github/workflows/check.yml:15,22,38,55,63`
- Modify: `.github/workflows/sanitizers.yml:23`
- Modify: `.github/workflows/test.yml:21,43,67,96`
- Modify: `.github/workflows/web.yml:17`

- [ ] **Step 1: Confirm current state**

Run: `grep -n 'actions/checkout@v4' .github/workflows/*.yml`
Expected: 10 lines of output across 5 files.

- [ ] **Step 2: Perform the bump on all 10 occurrences**

For each file in `book.yml`, `check.yml`, `sanitizers.yml`, `test.yml`, `web.yml`, replace every occurrence of:

```
uses: actions/checkout@v4
```

with:

```
uses: actions/checkout@v5
```

(`actions/checkout@v5` runs on Node.js 24. If the indentation differs by file, preserve the existing indentation — only swap `@v4` → `@v5`.)

- [ ] **Step 3: Verify the bump landed**

Run: `grep -n 'actions/checkout@v[45]' .github/workflows/*.yml`
Expected: 10 lines, all showing `@v5`. Zero `@v4` references remain.

- [ ] **Step 4: Verify the two new release workflows from WS8b are already on `@v5`**

Run: `grep -n 'actions/checkout' .github/workflows/release-sys.yml .github/workflows/release-safe.yml`
Expected: both files reference `@v5` (authored that way in WS8b).

### Task 2: Check `peaceiris/actions-mdbook@v2` Node.js status

**Files:**
- Possibly modify: `.github/workflows/book.yml:30`

- [ ] **Step 1: Inspect the current pin**

Run: `grep -n 'peaceiris/actions-mdbook' .github/workflows/book.yml`
Expected: one line at `book.yml:30` showing `@v2`.

- [ ] **Step 2: Check the action's current Node.js status**

Visit `https://github.com/peaceiris/actions-mdbook/releases` or run `gh api repos/peaceiris/actions-mdbook/releases/latest --jq '.tag_name'` to find the latest `v2.x` release tag. If `v2` was re-cut on Node.js 24, no change needed — the `@v2` floating ref already resolves to the Node.js 24 build. If a `v3` exists on Node.js 24 with no behavior changes, bump.

- [ ] **Step 3: Apply the change (if needed)**

If bump is warranted, change `book.yml:30` from `uses: peaceiris/actions-mdbook@v2` to whatever the new pin is. If no change, skip.

### Task 3: Local build sanity-check before pushing

**Files:** none (verification only)

- [ ] **Step 1: Workflow YAML still parses**

Run:
```bash
for f in .github/workflows/*.yml; do
  python -c "import yaml,sys; yaml.safe_load(open('$f')); print('OK', '$f')" || echo "FAIL $f"
done
```
Expected: every file prints `OK`.

### Task 4: Commit Fold-in 1

**Files:**
- Modify: 5 existing workflow YAML files (and possibly `book.yml` for `peaceiris/actions-mdbook` if bumped).

- [ ] **Step 1: Confirm working-tree contents**

Run: `git status --short`
Expected: only the workflow files modified. Untracked working files (`TODO.md`, `prompt.md`, `next-session-prompt.md`) remain untracked.

- [ ] **Step 2: Stage explicitly**

```bash
git add .github/workflows/book.yml .github/workflows/check.yml .github/workflows/sanitizers.yml .github/workflows/test.yml .github/workflows/web.yml
```

- [ ] **Step 3: Verify diff is minimal**

Run: `git diff --cached --stat`
Expected: 5 files modified (or 5 + 1 if `book.yml` got two distinct edits). No insertions of unrelated lines.

- [ ] **Step 4: Commit**

```bash
git commit -m "$(cat <<'EOF'
chore(ws8d): bump Node.js-20 actions to Node.js-24 versions

actions/checkout@v4 -> @v5 across all 5 existing workflows
(10 occurrences). peaceiris/actions-mdbook@v2 floats to latest
v2 build (or bumped if needed, see book.yml).

dtolnay/rust-toolchain pins (@stable / @1.85.0 / @nightly) left
alone — these are toolchain version pins, not action runtime pins.

The 2 new release workflows from WS8b are authored on @v5 from
the start; this commit only touches the legacy 5.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 5: Push and watch CI on the fork**

Run: `git push fork 6.0-rc 2>&1 | tail -5`
Expected: push succeeds.

Run: `gh run list -R Dacode45/ms-raylib-rs -L 5`
Expected: a new run appears for each of the 5 CI workflows. Wait for completion (or use `gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status` as the last command of the call so the exit status is reported).

If any workflow fails, the most likely cause is an `actions/checkout@v5` breaking change. Inspect the run log; if needed, revert that file to `@v4` and document in the commit reason. Re-attempt with an alternative bump path.

---

## Fold-in 2 — `paste` cargo-deny advisory acceptance

### Task 5: Look up the current RUSTSEC ID for `paste`

**Files:** none (research step)

- [ ] **Step 1: Query crates.io / RustSec advisory DB for the active advisory on `paste`**

Run: `cargo deny check advisories 2>&1 | grep -iE 'paste|RUSTSEC' | head -10`
Expected: either zero matches (if `unmaintained = "none"` is suppressing the notice) or an advisory line like `unmaintained: paste = "1.0.x" (RUSTSEC-2024-XXXX)`. Record the exact RUSTSEC ID.

If `cargo deny check` shows nothing because `unmaintained = "none"` suppresses it, query the RustSec DB directly:

Run: `curl -s https://rustsec.org/advisories/?package=paste 2>&1 | head -30`
or browse <https://rustsec.org/advisories/> for the `paste` entry. Record the ID.

### Task 6: Add the rationale'd ignore entry to `deny.toml`

**Files:**
- Modify: `deny.toml` (the `ignore = []` line under `[advisories]`)

- [ ] **Step 1: Locate the ignore line**

Run: `grep -n 'ignore = ' deny.toml`
Expected: one line showing `ignore = []`.

- [ ] **Step 2: Edit `deny.toml` to add the entry**

Replace the `ignore = []` line and the immediately surrounding comments with:

```toml
# paste 1.0 is unmaintained (see tracked-deferred follow-up). Used by
# raylib/src/core/callbacks/stream_processor_with_user_data_wrapper.rs.
# No known soundness issue; rewrite or library swap is deferred to a
# future workstream.
ignore = [
    { id = "RUSTSEC-XXXX-YYYY", reason = "paste is unmaintained; rewrite/swap tracked-deferred to a future workstream" },
]
```

Replace `RUSTSEC-XXXX-YYYY` with the actual ID looked up in Task 5.

- [ ] **Step 3: Verify the edit**

Run: `grep -A4 '^ignore' deny.toml`
Expected: shows the new entry with the actual RUSTSEC ID.

### Task 7: Verify `cargo deny check` passes

**Files:** none (verification only)

- [ ] **Step 1: Run the full advisory check**

Run: `cargo deny check 2>&1 | tail -20`
Expected: `error[unmaintained]` lines for `paste` (if any) are now suppressed. Final line says `all checks passed` or similar.

Note: because `unmaintained = "none"` was already in `deny.toml` (suppressing the scope entirely), this step may behave the same as before the edit. The explicit `ignore` entry is forward-defensive — if the scope is ever re-enabled, paste is still allowed.

### Task 8: Add a CHANGELOG note for the advisory acceptance

**Files:**
- Modify: `CHANGELOG.md` (within the `## 6.0.0 (unreleased)` block)

- [ ] **Step 1: Locate the `### Internal` (or `### Fixed`) section**

Run: `grep -n '^### Internal\|^### Fixed' CHANGELOG.md | head -5`
Expected: at least one section heading.

- [ ] **Step 2: Add a single-line bullet under `### Internal`**

Append to the `### Internal` list (or extend `### Fixed` if `### Internal` doesn't exist in the `## 6.0.0` block):

```
- Accepted `paste 1.0` cargo-deny unmaintained advisory (RUSTSEC-XXXX-YYYY) with rationale in `deny.toml`. Rewrite or library swap tracked for a future workstream.
```

Replace `RUSTSEC-XXXX-YYYY` with the actual ID.

- [ ] **Step 3: Verify**

Run: `grep -n 'paste 1.0 cargo-deny\|RUSTSEC-XXXX-YYYY' CHANGELOG.md`
Expected: one matching line in the `## 6.0.0` block with the actual ID.

### Task 9: Commit Fold-in 2

**Files:**
- Modify: `deny.toml`, `CHANGELOG.md`

- [ ] **Step 1: Stage explicitly**

```bash
git add deny.toml CHANGELOG.md
```

- [ ] **Step 2: Verify staged**

Run: `git diff --cached`
Expected: small additions to `deny.toml` (ignore entry + comment) and one bullet to `CHANGELOG.md`. No other changes.

- [ ] **Step 3: Commit**

```bash
git commit -m "$(cat <<'EOF'
chore(ws8d): accept paste cargo-deny advisory with rationale

paste 1.0 is unmaintained but used by
raylib/src/core/callbacks/stream_processor_with_user_data_wrapper.rs.
No known soundness issue; rewrite or library swap is deferred to a
future workstream.

deny.toml gains a narrowly-scoped ignore entry citing the RUSTSEC ID
and the rationale. CHANGELOG records the acceptance under
"### Internal".

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

## Fold-in 3 — Delete `samples/` in favor of `showcase/`

### Task 10: Remove `samples/` directory

**Files:**
- Delete: `samples/` (entire directory tree)

- [ ] **Step 1: Confirm `samples/` is excluded from the workspace**

Run: `grep -n 'samples' Cargo.toml`
Expected: shows `samples` under workspace `exclude = [...]`. (If not, the deletion step below may fail the workspace build until `Cargo.toml` is updated — record and address in the same commit.)

- [ ] **Step 2: Delete via `git rm`**

Run: `git rm -r samples/ 2>&1 | tail -5`
Expected: long list of removed files (Rust source, Cargo.toml, possibly target/, etc.).

If `samples/target/` is listed, that's because it isn't gitignored at the right level. Inspect `git status` after to confirm `samples/target/*` aren't still untracked-and-present.

- [ ] **Step 3: Update workspace `Cargo.toml` if `samples` was in `exclude` or `members`**

Run: `grep -B1 -A4 '^members\|^exclude' Cargo.toml`
Expected: workspace table. If `samples` appears in either `members` or `exclude`, remove that one entry. (`samples` was always excluded, so the entry is dead now and can go.)

- [ ] **Step 4: Verify workspace still builds**

Run: `cargo build --workspace 2>&1 | tail -10`
Expected: clean build.

### Task 11: Remove `structopt` dev-dependency from `raylib/Cargo.toml`

**Files:**
- Modify: `raylib/Cargo.toml:32` (currently `structopt = "0.3"`)

- [ ] **Step 1: Locate the line**

Run: `grep -n '^structopt' raylib/Cargo.toml`
Expected: `32:structopt = "0.3"` (or similar line).

- [ ] **Step 2: Delete the line**

Remove the `structopt = "0.3"` line from `raylib/Cargo.toml` `[dev-dependencies]`. Leave the rest of `[dev-dependencies]` (like `rand = "0.9"`) intact.

- [ ] **Step 3: Verify**

Run: `grep -E 'structopt' raylib/Cargo.toml`
Expected: zero matches.

Run: `grep -rE '^use structopt|extern crate structopt' raylib/ raylib-sys/ raylib-test/ showcase/ 2>&1 | head -5`
Expected: zero matches. (All usage was in the deleted `samples/`.)

- [ ] **Step 4: Verify safe-crate tests still build**

Run: `cd raylib && cargo test --lib --no-run 2>&1 | tail -10 && cd ..`
Expected: clean build of test binaries; structopt isn't referenced anymore.

### Task 12: Update `README.md` to point at `showcase/`

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Find the samples reference**

Run: `grep -n 'samples/\|samples ' README.md | head -10`
Expected: one or more references to `cd samples && cargo run --bin <name>` or similar.

- [ ] **Step 2: Replace the samples block with a showcase pointer**

For each reference, replace with a brief pointer to `showcase/`. Suggested text (adapt to match README's existing tone):

```markdown
### Runnable examples

Rust ports of raylib's official examples live in `showcase/`. To run one:

```sh
cd showcase && cargo run --bin <example_name>
```

The full set of examples is being ported; expect coverage to grow through the 6.0.x series.
```

If the README has a separate "Quick start" or "Hello world" snippet, leave it alone — only replace the references that pointed at `samples/`.

- [ ] **Step 3: Verify no stale `samples/` references remain**

Run: `grep -n 'samples' README.md`
Expected: zero matches, or only historical/blame-style mentions (unlikely).

### Task 13: Update `CLAUDE.md` workspace-layout bullet

**Files:**
- Modify: `CLAUDE.md`

- [ ] **Step 1: Find the samples bullet**

Run: `grep -n 'samples/' CLAUDE.md | head -10`
Expected: at least one line. The relevant one currently reads (or close to):

```
- `samples/` — runnable examples (excluded from workspace). **Being retired** — WS9 folds these into `showcase/`.
```

- [ ] **Step 2: Replace with the post-deletion text**

Change the line to:

```
- `samples/` — **removed in 6.0** (WS8); see `showcase/` for runnable Rust ports of raylib's examples.
```

If there are other `samples/` references elsewhere in CLAUDE.md (e.g., "Run a sample"), update them similarly to point at `showcase/`.

- [ ] **Step 3: Verify**

Run: `grep -n 'samples/' CLAUDE.md`
Expected: just the one updated reference, plus any historical mentions in the WS9/inventory sections (those stay).

### Task 14: Add CHANGELOG entries for the samples removal + structopt drop

**Files:**
- Modify: `CHANGELOG.md` (within the `## 6.0.0 (unreleased)` block)

- [ ] **Step 1: Locate the `### Breaking` section**

Run: `grep -n '^### Breaking' CHANGELOG.md | head -3`
Expected: one line near the top of the 6.0.0 block.

- [ ] **Step 2: Add a `samples/` removal bullet under `### Breaking`**

Add to the `### Breaking` list:

```
- **Removed `samples/` directory.** Runnable Rust ports of raylib's examples now live in `showcase/`; full coverage completes through 6.0.x. Users who ran `cd samples && cargo run --bin <name>` should migrate to `cd showcase && cargo run --bin <name>`.
```

- [ ] **Step 3: Add a `structopt` drop bullet under `### Internal` (or `### Fixed`)**

Add:

```
- `structopt` dev-dependency dropped from `raylib/Cargo.toml` (was only used by the now-removed `samples/`). Auto-resolves the cargo-deny unmaintained advisory on `structopt`.
```

- [ ] **Step 4: Verify**

Run: `grep -n 'samples/\|structopt' CHANGELOG.md | head -5`
Expected: at least two matching lines in the 6.0.0 block.

### Task 15: Final grep sweep for any stragglers

**Files:** none (verification only)

- [ ] **Step 1: Repo-wide grep for `samples/` references**

Run: `grep -rn 'samples/' --include='*.md' --include='*.toml' --include='*.rs' --include='*.yml' . 2>&1 | grep -v '^Binary file' | head -30`
Expected: matches only in:
- `CLAUDE.md`/`README.md`/`CHANGELOG.md` (the updated references)
- `docs/superpowers/notes/*.md` and `docs/superpowers/inventory.md` (historical records — leave alone)
- `docs/superpowers/specs/*.md` (historical records — leave alone)
- `docs/superpowers/plans/2026-05-29-ws8*.md` (this plan and the spec — leave alone)
- `next-session-prompt.md` / `prompt.md` / `TODO.md` (untracked working files — leave alone)

Anything else should be evaluated; stale active references must be fixed.

### Task 16: Commit Fold-in 3

**Files:**
- Delete: `samples/` (tree)
- Modify: `raylib/Cargo.toml`, `README.md`, `CLAUDE.md`, `CHANGELOG.md`, possibly root `Cargo.toml`

- [ ] **Step 1: Stage explicitly**

```bash
git add -u  # picks up the deletions and modifications
git add raylib/Cargo.toml README.md CLAUDE.md CHANGELOG.md  # explicit safety net
```

Note: `git add -u` is safe here because it only affects files already tracked. It does NOT add untracked files (so `TODO.md`, `prompt.md`, `next-session-prompt.md` stay untracked).

- [ ] **Step 2: Verify staged**

Run: `git diff --cached --stat`
Expected: many `samples/*.rs` files marked as deleted, `raylib/Cargo.toml` modified (-1 line for structopt), `README.md` modified, `CLAUDE.md` modified, `CHANGELOG.md` modified.

- [ ] **Step 3: Commit**

```bash
git commit -m "$(cat <<'EOF'
feat(ws8d)!: remove samples/ — superseded by showcase/

Deletes the entire samples/ directory tree. Runnable Rust ports of
raylib's examples now live in showcase/ (full coverage being ported
through the 6.0.x series).

Side effects:
- structopt dev-dependency dropped from raylib/Cargo.toml — it was
  only used by samples/ binaries. Auto-resolves the cargo-deny
  unmaintained advisory on structopt.
- README.md and CLAUDE.md updated to point at showcase/.
- CHANGELOG.md ### Breaking gets the migration note.

This is a 6.0 BREAKING change for anyone who was running
`cd samples && cargo run --bin <name>` against the pre-release
6.0-rc branch.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

### Task 17: Verify all CI on the fork after WS8d's three commits

**Files:** none (verification only)

- [ ] **Step 1: Push**

Run: `git push fork 6.0-rc 2>&1 | tail -5`
Expected: clean push.

- [ ] **Step 2: Watch CI**

Run: `gh run list -R Dacode45/ms-raylib-rs -L 10`
Expected: 5 workflow runs newly triggered (check / test / web / sanitizers / book). The 2 release workflows do NOT auto-trigger (workflow_dispatch only).

For each non-required workflow, this is the watch idiom:
```bash
gh run watch <run-id> -R Dacode45/ms-raylib-rs --exit-status
```
Use this AS THE LAST COMMAND IN THE CALL so the exit status is reported.

- [ ] **Step 3: Confirm green**

Expected: all 5 workflows green. If any go red, the most likely cause is the `actions/checkout@v5` bump from Fold-in 1; inspect, fix in a new commit (do not amend the existing ones).

---

## WS8d complete when

- `.github/workflows/*.yml` (the 5 legacy ones) use `actions/checkout@v5`. The 2 release workflows from WS8b are already on `@v5`.
- `deny.toml` has the rationale'd `paste` ignore entry; CHANGELOG records the acceptance.
- `samples/` directory deleted; `structopt` dev-dep removed from `raylib/Cargo.toml`; README + CLAUDE.md updated; CHANGELOG has the `### Breaking` + `### Internal` entries.
- `cargo build --workspace`, `cargo test -p raylib --lib`, `cargo deny check` all clean locally.
- All 5 CI workflows green on the fork at the WS8d HEAD commit.
- Three commits recorded (one per fold-in) with the WS8d-shaped messages.

Next: WS8e (fork PR + checkpoint review).
