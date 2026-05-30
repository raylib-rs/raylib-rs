# WS8 complete — release prep + checkpoint

**Status:** DONE on `Dacode45:unstable` (fork only; canonical merge deferred). PR #2 merged with merge-commit at `94f3fef`.

Spec: `docs/superpowers/specs/2026-05-29-ws8-release-prep-checkpoint-design.md`.
Plans: `docs/superpowers/plans/2026-05-29-ws8{a,b,c,d,e}-*.md`.

Builds on WS7 (`docs/superpowers/notes/ws7-complete.md`) — 28-chapter mdBook + CHANGELOG 6.0.0 entry + 5 CI workflows green.

---

## WS8a — version bump

**Commits:**
- `a00cf53` — feat(ws8a): bump raylib + raylib-sys to 6.0.0 + sync book snippets
- `8364c73` — fix(ws8a-plan): correct mdbook -L path to workspace-root target/debug/deps
- `197372f` — fix(ws8a): bump README + features.md version snippets to 6.0 (plan-scope gap)
- `5efb366` — fix(ws8a-plan): broaden Task 5 grep sweep to whole book + README

What shipped:
- `raylib/Cargo.toml` + `raylib-sys/Cargo.toml` versions 5.7.0 → 6.0.0; inter-crate dep pin synced.
- `book/src/getting-started/quickstart.md`, `book/src/core-concepts/features.md`, `README.md` version snippets all → `6.0` / `6.0.0`.
- CHANGELOG heading deliberately stays `## 6.0.0 (unreleased)` — date flip is the future final-release step's job.

---

## WS8b — release workflows

**Commits:**
- `6be0881` — feat(ws8b): draft release-sys.yml + release-safe.yml + sync-check helper
- `82dca54` — fix(ws8b): harden check-release-sync.sh against silent failures (review fixes)

What shipped:
- `.github/workflows/release-sys.yml` and `release-safe.yml` — both `workflow_dispatch`-only with `dry_run: bool` (default `true`).
- `scripts/check-release-sync.sh` — version/CHANGELOG/dep-pin sync gate; `--require-date` is the safety gate fired only when `dry_run: false`.
- Real publish step gated behind `if: ${{ inputs.dry_run == false }}` so `CARGO_REGISTRY_TOKEN` is only consumed during intentional real publishes.

---

## WS8c — act validation

**Commit:** `726e550` — docs(ws8c): validate release workflows via act + record runbook notes.

What shipped:
- Three documented `act` runs:
  - `release-sys.yml + dry_run=true` → SUCCESS.
  - `release-safe.yml + dry_run=true` → known limitation (fails at cargo step until raylib-sys 6.0.0 is published for real; host-side direct `cargo publish --dry-run` reproduces the planned outcomes).
  - `release-sys.yml + dry_run=false + dummy token` → safety gate fires at sync-check; real-publish step never reached.
- `docs/superpowers/notes/ws8c-validation.md` — runbook + fallback + final-release runbook teaser.

---

## WS8d — release-hygiene fold-ins

**Commits (one per fold-in):**
- `2f1b813` — chore(ws8d): bump Node.js-20 actions to Node.js-24 versions (12 occurrences of `actions/checkout@v4` → `@v5` across 5 legacy workflows; `peaceiris/actions-mdbook@v2` left at latest v2).
- `9dc6e73` — chore(ws8d): accept `paste` cargo-deny advisory with rationale (RUSTSEC-2024-0436 ignore entry in `deny.toml`; CHANGELOG `### Internal` bullet).
- `9225b6d` — feat(ws8d)!: remove `samples/` — superseded by `showcase/` (73-file deletion; `structopt` dev-dep dropped + advisory auto-resolves; README + CLAUDE.md + CHANGELOG + typos.toml updated).

---

## WS8e — fork PR checkpoint

**Commits:**
- `58c5224` — fix(ws8e-review): address 4 of 10 PR review comments + record dispositions
- `94f3fef` — Merge pull request #2 from Dacode45/6.0-rc (the `--no-ff` merge commit)
- `<this-commit>` — docs(ws8): mark WS8 complete in CLAUDE.md + done-note

PR #2 opened `Dacode45:6.0-rc` → `Dacode45:unstable`. Owner reviewed line-by-line with 10 inline comments. Disposition recorded in `docs/superpowers/notes/ws8e-checkpoint-review-feedback.md`:
- 4 fixed in `58c5224` (book.yml caching, release-sys publish verification, controls.rs debug_asserts).
- 1 already covered by existing test (matrix ordering — `raylib-sys/tests/conversions.rs:251`).
- 5 deferred to future workstreams (nobuild CI, Color From&, thiserror, DataBuf+more tests, rlgl audit).

Merged with "Create a merge commit" (`--no-ff`) at `94f3fef` — two parents: `ba73bca` (prior `unstable` HEAD) and `58c5224` (final tip of `6.0-rc`).

CLAUDE.md status line flipped: WS7 ✅ → WS8 ✅; WS9 ← NEXT.

---

## Final CI inventory (fork's unstable at merge HEAD)

| Workflow | Status |
|----------|--------|
| `check.yml` | ✅ |
| `test.yml` | ✅ |
| `web.yml` | ✅ |
| `sanitizers.yml` | ✅ (informational) |
| `book.yml` | ✅ (now with apt + rust caching) |
| `release-sys.yml` | ⚪ workflow_dispatch-only; act-validated |
| `release-safe.yml` | ⚪ workflow_dispatch-only; act-validated with known limitation |

---

## WS8 done-criteria sign-off (spec §10)

- [x] `raylib/Cargo.toml` version = `6.0.0`, `raylib-sys` dep pin = `6.0.0`.
- [x] `raylib-sys/Cargo.toml` version = `6.0.0`.
- [x] `book/src/getting-started/quickstart.md` shows `raylib = "6.0"`.
- [x] `book/src/getting-started/install-*.md` references are version-consistent.
- [x] `CHANGELOG.md` `## 6.0.0 (unreleased)` block intact (date flip deferred).
- [x] `.github/workflows/release-sys.yml` exists and validates clean under `act` with `dry_run=true`.
- [x] `.github/workflows/release-safe.yml` exists; YAML + sync-check validate under `act` with the documented `cargo publish --dry-run -p raylib` registry-resolution caveat.
- [x] `scripts/check-release-sync.sh` exists and passes for both crates without `--require-date`; fails with `--require-date` (proves the gate).
- [x] All 5 existing CI workflows + the 2 new ones present on `6.0-rc`. The 5 auto-running are green.
- [x] All 5 + 2 workflows use Node.js 24 actions (no deprecation warnings in run logs).
- [x] `deny.toml` has the rationale'd `paste` ignore entry; `cargo deny check` clean.
- [x] `samples/` directory removed.
- [x] `structopt` dev-dependency removed from `raylib/Cargo.toml`.
- [x] `README.md` + `CLAUDE.md` updated to point at `showcase/` instead of `samples/`.
- [x] `CHANGELOG.md` `### Breaking` includes the `samples/` removal note; `### Internal`/`### Fixed` notes the `paste` advisory acceptance and the `structopt` removal.
- [x] `docs/superpowers/notes/ws8c-validation.md` exists with the validation commands + gotchas.
- [x] PR #2 (`Dacode45:6.0-rc` → `Dacode45:unstable`) opened, reviewed by owner, merged with "Create a merge commit". Fork's `unstable` HEAD = the merge commit `94f3fef`.
- [x] Tracked-deferred list updated with the `paste` rewrite/swap follow-up.
- [x] `CLAUDE.md` status line flipped: WS7 ✅ · **WS8 ✅** · WS9 ← NEXT.
- [x] `docs/superpowers/notes/ws8-complete.md` written.

**Out of done-criteria (future)**: crates.io publication, `v6.0.0` tag, canonical merge to `raylib-rs/raylib-rs`, GitHub release.

---

## Tracked-deferred follow-ups carried out of WS8

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

New from WS8 (carrying out of this checkpoint):
11. **`paste` macro rewrite or library swap** (from WS8d Fold-in 2).
12. **Nobuild-mode CI matrix** (from review comment 4 on PR #2).
13. **Color `From<&Color>` vs Clone-only audit** (from review comment 5).
14. **`thiserror` migration crate-wide** (from review comment 7).
15. **DataBuf + Mesh + more-tests umbrella** (from review comment 8 + owner's "more tests" intent).
16. **rlgl safe-module coverage audit** (from review comment 10).

Owner's stated post-release direction:
- **bevy-raylib crate** — new crate depending on the published `raylib 6.0.0` (post final-release).

Out of WS8, queued for the future final-release workstream:
- CHANGELOG date flip.
- `v6.0.0` tag on canonical.
- Manual `release-sys.yml` + `release-safe.yml` real-publish runs.
- PR #2-style PR from `Dacode45:unstable` → `raylib-rs:unstable`.
- GitHub release creation.

---

WS8 done. Next: **WS9 (showcase + Pages finale)**.
