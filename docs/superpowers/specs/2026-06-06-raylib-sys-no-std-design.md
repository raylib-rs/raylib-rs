# raylib-sys no_std — design

**Date:** 2026-06-06
**Status:** approved (maintainer sign-off on topology + proof leg, this session)
**Queue:** post-release flexible-queue item 18 (`ws8e-checkpoint-review-feedback.md`, queued via PR #307)
**Adopts:** community PR #251 (nbe1233), adapted for 6.0 — see "Attribution" below.

## Goal

Make `raylib-sys` compile under `#![no_std]`, proven by a CI check against a
real no-std target. Scope is **raylib-sys only** — the safe `raylib` crate uses
`String`/`Vec`/`Box` throughout and is not a no_std candidate. If the safe
crate breaks because of this change, that is a bug in this change.

## Sequencing decision

crates.io is still at `6.0.0-rc.2` (final 6.0.0 publish outstanding). The queue
memory suggested landing no_std after 6.0.0 ships. **Maintainer decision
(2026-06-06): proceed now** — the change is additive and rides into 6.0.0 (or a
6.0.x) whenever the publish happens.

## std-usage census (verified 2026-06-06 on `unstable`)

| Location | `std::` refs | Class |
|---|---|---|
| generated `bindings.rs` | 1,495 | bindgen default paths → fixed wholesale by `.use_core()` |
| `src/vector_math.rs` | 27 | all `std::ops::*` trait impls |
| `src/matrix_quat_math.rs` | 16 | all `std::ops::*` trait impls |
| `src/color.rs` | 1 | `std::num::ParseIntError` (identical type exists in `core::num`) |
| `build.rs` | 12 | host-side; build scripts always run with std — not a no_std concern |
| `tests/*.rs` | 19 | separate test crates; they link std regardless — unchanged |

No float-math methods (`.sqrt()` etc.) anywhere in `src/` — all math goes
through the raymath C shim (WS2a), so the source sweep is purely mechanical
`std::` → `core::` path swaps. Nothing in the library code needs std.

## Decision 1: feature topology — unconditional `#![no_std]`

**Chosen: Approach A — plain `#![no_std]`, zero feature changes.**

- `lib.rs` gets `#![no_std]`. No `std` feature, no `default` split, no new
  public feature names.
- A no_std *library* crate is fully consumable by std code. Consumers observe
  nothing: same types, same impls (`core::num::ParseIntError` *is*
  `std::num::ParseIntError`).
- The safe crate's `raylib-sys = { default-features = false }` +
  `default = ["raylib-sys/default"]` forwarding is untouched. Existing
  `default-features = false` users see no semantic change.
- No `clippy::std_instead_of_core` lint needed: under unconditional
  `#![no_std]` the compiler rejects any `std::` path in the lib crate, which is
  strictly stronger.
- If a future addition genuinely needs std, adding a default-on `std` feature
  then is additive and non-breaking for hosted users. `core::error::Error`
  (stable since 1.81, MSRV is 1.85) removes the classic reason to need one.

**Rejected — B: PR #251's `default = ["std", "default_cmake_build_options"]`
split.** The `std` feature would gate nothing today (a no-op feature, its own
confusion) and the split adds two public feature names that exist only to
serve it.

**Rejected — C: explicit `no_std` opt-in feature.** Non-additive feature
semantics; broken by cargo feature unification.

## Decision 2: proof leg — thumb check with host-generated binding

**Chosen: (b1)** — new CI job (in `.github/workflows/check.yml`, landed as its
own `ci:` commit), two steps in one job:

1. **Generate binding on host:** `cargo build -p raylib-sys --features nobuild`
   (bindgen runs; no C compile), then locate
   `target/debug/build/raylib-sys-*/out/bindings.rs`.
2. **Check on a real no-std target:**
   `rustup target add thumbv7em-none-eabihf`, then

   ```
   RAYLIB_BINDGEN_LOCATION=<abs path to bindings.rs> \
     cargo check -p raylib-sys --target thumbv7em-none-eabihf \
       --no-default-features --features nobuild,nobindgen
   ```

Self-refreshing (no committed binding artifact, no drift). The binding is
host-flavored (x86_64 layouts) compiled for ARM — fine for a compile-only
proof; it never runs. Hosted `--no-default-features` builds were rejected as
the proof: under unconditional `#![no_std]` every existing build already
polices `std::` paths; only a std-less target proves the crate graph builds
without std.

**Anti-silent-skip verification (required, pre-CI-wiring):** temporarily add a
`std::` reference locally and confirm the verbatim command above goes red, then
revert. A gated check that silently compiles out proves nothing.

**Ruleset note:** this is an *added* job (no rename/delete), so the
branch-protection required-checks list is unaffected. Add it as required — it
is fast.

**Contingency (resolved during implementation):** the host-generated binding
failed the thumb check on bindgen *layout assertions* (host 64-bit sizes vs
32-bit ARM), not on missing-std paths. The spec's original fallback (pass
`--target` to bindgen's clang args) would require target libc headers
(newlib) on every generating host, so instead `nobuild` bindings are
generated with `.layout_tests(false)` — the proof leg is a compile-only
no-std check, not an ARM ABI claim; hosted default builds keep layout tests
(plus `tests/layout_compat.rs`). Rejected-for-now alternative (b2):
committing a reference `bindings.rs` — large generated file that drifts with
every raylib/bindgen bump; revisit with the nobuild-CI-matrix queue item
(13) if it wants one.

**Plan refinements:** the CI job also checks `--features
nobuild,nobindgen,mint` to prove the `mint` adapter is no-std-clean, and
`thumbv7em-none-eabihf` is added to `rust-toolchain.toml` targets so local
runs need no manual `rustup target add`.

## Code changes

- `raylib-sys/src/lib.rs`: `#![no_std]` at top; existing `allow` attrs stay.
- `raylib-sys/src/vector_math.rs`, `matrix_quat_math.rs`: `std::ops::` →
  `core::ops::`.
- `raylib-sys/src/color.rs`: `std::num::ParseIntError` →
  `core::num::ParseIntError`.
- `raylib-sys/build.rs`: add `.use_core()` to the bindgen builder, and gate
  `gen_bindings()` behind `#[cfg(not(feature = "nobindgen"))]` so `nobindgen`
  does what its docs claim (planning discovery: build.rs ran bindgen even
  under `nobindgen`, which would have made the proof leg run bindgen against
  the thumb target). Build scripts otherwise unchanged — they run on the
  host with std.
- `raylib-sys/tests/`: unchanged.
- `raylib-sys/Cargo.toml`: unchanged (no feature edits).

## Bindgen regen verification

`.use_core()` flips ~1,495 `::std::` paths to `::core::` in the generated
binding. Required step: generate before/after, diff, confirm the **only**
change class is the path prefix — nothing else may shift under bindgen 0.72.1.
Since `nobindgen` users consume pregenerated bindings, use_core output is what
any future reference binding must be generated from.

## Optional deps (v1 disposition)

- `mint` — core-only; works on no-std targets as-is.
- `glam`, `serde` — current declarations pull std. Hosted targets: nothing
  changes. No-std targets: enabling those features won't build. **Documented,
  not gated** (there is no `std` feature to gate on). Relaxing via
  `default-features = false` dep shapes is recorded as future work in the
  done-note.

## Attribution & PR mechanics

- Adopting commit: `Co-authored-by: nbe1233 <27390193+nbe1233@users.noreply.github.com>`
  + the standard Claude trailer. PR body: "Closes #251, adapted for 6.0"
  (adaptations: post-WS1 feature list, bindgen 0.72.1, post-WS2a raymath files,
  simpler unconditional-no_std topology).
- PR #307 (docs-only, queues this item): merge as-is first; fallback, fold into
  this workstream's PR.
- CHANGELOG entry under unreleased.
- Done-note: `docs/superpowers/notes/raylib-sys-no-std-complete.md` capturing
  the before/after census, the topology decision, the verbatim proof-leg
  command, and the optional-dep disposition.

## Testing

- Existing Tier-1/Tier-2 suites unchanged (they exercise raylib-sys through std
  test crates).
- The proof leg is the new test; local reproduction copies the CI command
  verbatim (memory: `plan-clippy-vs-ci-command-divergence`).
- Full quality gates apply: `cargo fmt -p raylib-sys`, clippy `-Dwarnings`,
  doctests, MSRV 1.85.

## Done criteria

- `raylib-sys` compiles for `thumbv7em-none-eabihf` (proof leg green in CI,
  verified able-to-fail).
- Default/hosted builds completely unchanged; existing
  `default-features = false` users unaffected.
- PR #251 closed with attribution; PR #307 merged or folded.
- CHANGELOG entry; done-note written; queue item 18 closed.
