# Final-release workstream — COMPLETE (6.0.0 published)

**Status:** DONE. `raylib-sys 6.0.0` and `raylib 6.0.0` are live on
crates.io. This is the final cut; the rc.2 milestone
(`final-release-rc2-complete.md`) was the dress rehearsal and remains the
canonical reference for the release-pipeline incident write-ups.

## What shipped

| Crate | Version | crates.io indexed |
|-------|---------|-------------------|
| `raylib-sys` | 6.0.0 | 2026-06-10T06:13:30Z |
| `raylib`     | 6.0.0 | 2026-06-10T06:17:57Z |

Tags on canonical:
- `v6.0.0` force-moved `47e126f → 2b0b3e3` (the published final-cut commit).
  The pre-rc.2 snapshot it previously marked is no longer tagged; the
  maintainer call this session (over the rc.2 "keep it" call) was to point
  `v6.0.0` at what was actually published.
- `v6.0.0-rc.2` still on `bddca73` (rc.2 publish source); unchanged.
- `final-release-complete` on the closeout commit.

## The final cut (this session)

1. CI green on `unstable` HEAD (`04c66a9`) — all legs.
2. Bumped `6.0.0-rc.2 → 6.0.0` in `raylib/`, `raylib-sys/`, `showcase/`
   Cargo.toml (versions + inter-crate pins).
3. CHANGELOG: folded `## Unreleased` into a single `## 6.0.0 — 2026-06-09`
   entry — the post-rc.2 items (log feature, `raylib-sys` no_std #327,
   Vector tuple/array `From` #332, wrapper soundness #331/#276, MSRV
   1.85→1.88 #330, `paste` removal #329, nobindgen/nobuild fixes) merged into
   the matching sections of the rc.2 body; rc-rationale blockquote removed.
4. Version snippets in `README.md` + `book/.../quickstart.md` +
   `book/.../features.md` dropped from the explicit `6.0.0-rc.2` pin to
   caret `6.0`; quickstart rc-note blockquote removed. README reviewed
   end-to-end (already fully 6.0-current otherwise).
5. Local `cargo build -p raylib -p raylib-sys --features full` green;
   both crates pass `check-release-sync.sh --require-date`.
6. PR [#345](https://github.com/raylib-rs/raylib-rs/pull/345) opened →
   31/31 checks green → squash-merged as `2b0b3e3`.
7. Tag decision (maintainer): move `v6.0.0` onto the cut. Done.
8. Publish sequence (all dry→real, sequential, never parallel):

   | Workflow | dry-run | real |
   |----------|---------|------|
   | `release-sys.yml`  | run `27256925642` ✅ | run `27257029746` ✅ |
   | `release-safe.yml` | run `27257133421` ✅ | run `27257212352` ✅ |

   `release-safe` dry-run confirmed the WS8c "dry-run needs real
   raylib-sys" limitation cleared once `raylib-sys 6.0.0` indexed.
9. `CLAUDE.md` status flipped to `final-release ✅` (published).
10. Announcement drafts in `release-announcement.md` finalized (date +
    verified links) but **NOT posted** — deliberately held per the
    session instruction; posting is a manual follow-up.

## Pitfalls confirmed clear (from the rc.2 path)

- `CRATES_IO_TOKEN` secret → `CARGO_REGISTRY_TOKEN` env var: worked, no
  re-create needed.
- Release-workflow Linux build deps (PR #298): the dry-run verify build
  passed `FindX11`/cmake on all four dispatches.
- Sync-check `--require-date` gate fired correctly on the real runs.

## Outstanding / post-release

- **Announcement**: post the `release-announcement.md` drafts (Reddit
  r/rust + r/gamedev, Discord, lib.rs/This-Week-in-Rust, raylib Discord).
  Re-confirm the two version-pinned `crates.io/.../6.0.0` links + docs.rs
  build before posting.
- **Pages**: gallery live at <https://raylib-rs.github.io/raylib-rs/>;
  canonical book+gallery deploy continues on `unstable` pushes.
- **bevy-raylib crate** ([[post-release-bevy-raylib]]); the flexible queue
  in `ws8e-checkpoint-review-feedback.md`; showcase clippy cleanup; wasm
  build-speed (re-enable sccache once GHA cache healthy); re-triage the
  `post-6.0`-labeled issue backlog.

## Note on dates

CHANGELOG heading is `## 6.0.0 — 2026-06-09` (commit-day, local TZ);
crates.io indexed timestamps are 2026-06-10 UTC. Same off-by-one as rc.2
(CHANGELOG 06-02, indexed 06-03) — commit day vs UTC publish day.
