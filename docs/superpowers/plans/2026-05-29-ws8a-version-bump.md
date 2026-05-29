# WS8a — Version bump + book snippet sync Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bump both crate manifests + the book version snippet from `5.7.0` to `6.0.0` in lock-step so the fork's `unstable` carries the right version when the owner reviews. CHANGELOG date flip is **not** WS8a's job.

**Architecture:** Mechanical text edits to three files (`raylib/Cargo.toml`, `raylib-sys/Cargo.toml`, `book/src/getting-started/quickstart.md`), plus a grep sweep over the install guides for stray version references. Verify via `cargo build --workspace`, `cargo test -p raylib`, and `mdbook build book`.

**Tech Stack:** Rust + Cargo, mdBook.

**Spec reference:** `docs/superpowers/specs/2026-05-29-ws8-release-prep-checkpoint-design.md` §4 WS8a.

---

### Task 1: Bump `raylib-sys/Cargo.toml` version

**Files:**
- Modify: `raylib-sys/Cargo.toml:3`

- [ ] **Step 1: Read the current line to confirm**

Run: `grep -n '^version' raylib-sys/Cargo.toml | head -1`
Expected: `3:version = "5.7.0"`

- [ ] **Step 2: Edit the version**

Change line 3 from `version = "5.7.0"` to `version = "6.0.0"`.

- [ ] **Step 3: Verify the edit landed**

Run: `grep -n '^version' raylib-sys/Cargo.toml | head -1`
Expected: `3:version = "6.0.0"`

### Task 2: Bump `raylib/Cargo.toml` version + inter-crate dep pin

**Files:**
- Modify: `raylib/Cargo.toml:3` (crate version)
- Modify: `raylib/Cargo.toml:17` (inter-crate dep pin)

- [ ] **Step 1: Read both lines to confirm**

Run: `sed -n '3p;17p' raylib/Cargo.toml`
Expected:
```
version = "5.7.0"
raylib-sys = { version = "5.7.0", path = "../raylib-sys", default-features = false }
```

- [ ] **Step 2: Edit line 3 (crate version)**

Change `version = "5.7.0"` to `version = "6.0.0"`.

- [ ] **Step 3: Edit line 17 (dep pin)**

Change `raylib-sys = { version = "5.7.0", path = "../raylib-sys", default-features = false }` to `raylib-sys = { version = "6.0.0", path = "../raylib-sys", default-features = false }`.

- [ ] **Step 4: Verify both edits landed**

Run: `sed -n '3p;17p' raylib/Cargo.toml`
Expected:
```
version = "6.0.0"
raylib-sys = { version = "6.0.0", path = "../raylib-sys", default-features = false }
```

### Task 3: Workspace build verifies version bump compiles

**Files:** none (verification only)

- [ ] **Step 1: Clean any stale Cargo.lock to force re-resolution**

Run: `cargo update -p raylib-sys -p raylib --workspace 2>&1 | tail -10`
Expected: cargo regenerates `Cargo.lock` referencing `raylib-sys 6.0.0` and `raylib 6.0.0`. (`Cargo.lock` is gitignored so we don't commit it.)

- [ ] **Step 2: Build the workspace**

Run: `cargo build --workspace 2>&1 | tail -20`
Expected: clean build, no errors. `warning:` lines about feature unification are acceptable; `error[...]` lines are not.

- [ ] **Step 3: Run safe-crate unit tests**

Run: `cd raylib && cargo test --lib 2>&1 | tail -20 && cd ..`
Expected: all tests pass. Note: `cargo test` (without `--lib`) opens a window; `--lib` keeps it headless.

### Task 4: Update book quickstart version snippet

**Files:**
- Modify: `book/src/getting-started/quickstart.md`

- [ ] **Step 1: Find the version snippet line**

Run: `grep -n 'raylib = "5\.7"' book/src/getting-started/quickstart.md`
Expected: one or more matching lines with line numbers.

- [ ] **Step 2: Edit each occurrence**

Replace `raylib = "5.7"` with `raylib = "6.0"` in every matching line. If the file also references the version in surrounding prose ("install raylib 5.7"), update those too — they're the same upgrade.

- [ ] **Step 3: Check for the footnote about WS8**

Run: `grep -n 'WS8\|bumps to 6.0\|6\.0 in WS8' book/src/getting-started/quickstart.md`
Expected: a footnote that says something like "version bumps to 6.0 in WS8". Update the prose to past tense ("version is 6.0 as of the WS8 bump") OR remove the footnote if it now reads redundantly. Pick whichever fits the chapter's tone.

- [ ] **Step 4: Verify no stale `5.7` references remain**

Run: `grep -n '5\.7' book/src/getting-started/quickstart.md`
Expected: zero matches (or only matches in CHANGELOG-style historical references; those are fine).

### Task 5: Grep sweep over install guides for version references

**Files:**
- Modify (if matches found): `book/src/getting-started/install-windows.md`, `install-macos.md`, `install-linux.md`, `install-web.md`

- [ ] **Step 1: Grep all install guides for version-like strings**

Run: `grep -n '5\.7\|5\.7\.0' book/src/getting-started/install-*.md`
Expected: zero or a small number of matches. Each match needs evaluating.

- [ ] **Step 2: For each match, update or leave**

If the match is a version snippet (`raylib = "5.7"` or similar in a code block), change to `"6.0"`.
If the match is a historical reference (e.g. "raylib 5.7 was the previous"), leave it.
If the match is in a deferred-targets note, leave it.

- [ ] **Step 3: Verify post-edit state**

Run: `grep -rn '5\.7' book/src/getting-started/` | tee /tmp/ws8a-grep-result.txt
Expected: only historical references remain. Audit the output; if anything looks like a stale active reference, fix it.

### Task 6: Build the book to verify markdown still parses + doctests still compile

**Files:** none (verification only)

- [ ] **Step 1: Build the book**

Run: `mdbook build book 2>&1 | tail -10`
Expected: `Book built`. No `error:` lines.

- [ ] **Step 2: Run mdbook doctests**

Run: `cargo build -p raylib --features full 2>&1 | tail -5 && mdbook test book -L target/debug/deps 2>&1 | tail -20`
Expected: doctests pass. Note the `-L target/debug/deps` path is the workspace-root target dir (Cargo workspaces consolidate artifacts there), not `raylib/target/debug/deps`. This matches the WS7 `book.yml` invocation. Doctests in quickstart that reference the version literally should still compile because the version string is in a Markdown code fence, not in compilable Rust.

### Task 7: Commit the version bump

**Files:**
- Modify: `raylib/Cargo.toml`
- Modify: `raylib-sys/Cargo.toml`
- Modify: `book/src/getting-started/quickstart.md`
- Modify (if Task 5 touched anything): `book/src/getting-started/install-*.md`

- [ ] **Step 1: Confirm working-tree contents**

Run: `git status --short`
Expected: only the files listed above show as modified. **Verify** untracked files like `TODO.md`, `prompt.md`, `next-session-prompt.md` are NOT in any staging list. If they are, `git reset` them.

- [ ] **Step 2: Stage the modified files explicitly (no `git add -A`)**

```bash
git add raylib/Cargo.toml raylib-sys/Cargo.toml book/src/getting-started/quickstart.md
# If Task 5 touched install guides, also:
git add book/src/getting-started/install-*.md
```

- [ ] **Step 3: Verify staged contents are exactly what we want**

Run: `git diff --cached --stat`
Expected: 2-5 files modified, all of them in `raylib/`, `raylib-sys/`, or `book/src/getting-started/`. CHANGELOG.md NOT in the list.

- [ ] **Step 4: Commit**

```bash
git commit -m "$(cat <<'EOF'
feat(ws8a): bump raylib + raylib-sys to 6.0.0 + sync book snippets

Version bump in lock-step:
- raylib/Cargo.toml: 5.7.0 -> 6.0.0 (+ inter-crate dep pin)
- raylib-sys/Cargo.toml: 5.7.0 -> 6.0.0
- book/src/getting-started/quickstart.md: raylib = "5.7" -> "6.0"
- book/src/getting-started/install-*.md: opportunistic updates

CHANGELOG heading deliberately stays as "## 6.0.0 (unreleased)";
the date flip is the future final-release step's job, not WS8's.

Cargo.lock is gitignored; not committed.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 5: Verify the commit**

Run: `git log -1 --stat`
Expected: shows the commit with the changed files and the WS8a message.

---

## WS8a complete when

- `raylib/Cargo.toml`, `raylib-sys/Cargo.toml`, and the book quickstart all reference `6.0.0` / `6.0`.
- The inter-crate dep pin in `raylib/Cargo.toml:17` matches the bumped versions.
- `cargo build --workspace` and `cargo test -p raylib --lib` are green.
- `mdbook build book` and `mdbook test book` are green.
- CHANGELOG `## 6.0.0 (unreleased)` heading is untouched (date flip deferred).
- One commit recorded with the WS8a-shaped message.

Next: WS8b (release workflows).
