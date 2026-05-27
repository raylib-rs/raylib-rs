# WS6-prep complete — clippy `-Dwarnings` blockers cleared + deferred quality PRs folded in

**Status:** DONE on branch `6.0-rc` (pending the push-to-fork CI confirmation that closes this note). Spec: `docs/superpowers/specs/2026-05-27-ws6-platform-cicd-design.md` §3. Plan: `docs/superpowers/plans/2026-05-27-ws6-prep-blockers-and-pr-foldin.md`.

WS6-prep settles the safe-crate API surface and makes `raylib-sys` + `raylib` clippy-`-Dwarnings`-clean so WS6a can flip the quality gates on a clean tree. Executed subagent-driven (implementer + verification per task).

## Blocker fixes

- **build.rs acronyms** (`d1ed89f`): `Platform::{DRM→Drm, RPI→Rpi}`, `PlatformOS::{BSD→Bsd, OSX→Osx}` — all 19 sites; cmake define strings unchanged. Clears `clippy::upper_case_acronyms`.
- **Dead deprecated audio callback** (`7e65250`): removed the `#[deprecated]` `custom_audio_stream_callback` trampoline + the `RaylibHandle::set_audio_stream_callback` method that invoked it (the self-deprecation `-Dwarnings` blocker), plus the now-dead `AUDIO_STREAM_CALLBACK` static / `audio_stream_callback()` accessor / `RustAudioStreamCallback` alias. Live path is the generic `set_audio_stream_callback` in `core/callbacks/audio_stream_callback.rs` (distinct `AUDIO_STREAM_CALLBACK_SLOT`). No in-workspace caller used the removed method. Owner-confirmed outright removal (no courtesy bridge).

## Idiom PRs folded in (with attribution)

- **#272** (`f9b4eda`, AmityWilder): `c"..."` literals over `CStr::from_bytes_with_nul` — audio.rs (1 const) + file.rs (10 doctest/test sites). All PR sites still present; only string-literal sites converted.
- **#268** (`b6ae353`, AmityWilder): `impl Into<T> for U` → `impl From<U> for T` — 11 impls across color.rs/camera.rs/texture.rs/vr.rs; transmute bodies preserved; clears `from_over_into`.
- **#266** (`1838b2b`, AmityWilder): sealed `AudioSample` via `private::AudioSample` supertrait + blanket impl over u8/i16/f32 (closes #213).

## Clippy `-Dwarnings` sweep (Task 5b, `44ce49c`, attribution AmityWilder for #224)

Cleared ALL remaining clippy lints in `raylib-sys` + `raylib` (the plan had assumed only the two named blockers; reality had ~dozen lint families). 25 files; idiom-only, no behavior/API change. Lints fixed: `useless_conversion`, `needless_return`, `needless_lifetimes`, `needless_borrow`, `missing_safety_doc` (≈76, via `# Safety` docs on the `impl_wrapper!`/`gen_from_raw_wrapper!` macros), `double_must_use`, `unnecessary_cast`, `ptr_arg` (`&Vec<u8>`→`&[u8]`, backward-compatible widening), `manual_map`, `legacy_numeric_constants`, `transmute_ptr_to_ref`, `missing_transmute_annotations`, `derivable_impls` (kept `TraceLogLevel::default()=LOG_INFO` with a justified `#[allow]` — derive would wrongly give LOG_ALL), `needless_doctest_main`, `type_complexity`, `redundant_field_names`, `unused_unit`, `unused_imports`, `suspicious_doc_comments`. No crate-level blanket `#![allow]`. **Independently reviewed** (diff + all four clippy commands + doctests + render_shapes/render_text re-run): APPROVED, all transmute/cast edits verified identity/`#[repr(C)]`-safe.

Four gate commands confirmed clean (`-D warnings`, exit 0): `clippy -p raylib-sys`; `clippy -p raylib-sys --no-default-features --features software_renderer`; `clippy -p raylib --lib --features <module set>`; same with `software_renderer`.

## Soundness PRs

- **#277** (`aff23b0`, AmityWilder, **PARTIAL + deferred**): removed the 2 genuinely-unsound, unused `AsRef`/`AsMut<ffi::AudioStream> for Sound` impls (they exposed the stream's raw `*mut` pointer fields to safe mutation). **The bulk is deferred by owner decision:** the `deref_impl_wrapper!` macro auto-generates `AsRef`/`AsMut`/`Deref`/`DerefMut` for ~19 pointer-owning wrappers (`Mesh`/`Model`/`Image`/`Material`/`Shader`/…), and they are load-bearing — `drawing.rs` (10+ sites) and `models.rs` use `impl AsRef<ffi::Font/Mesh/Model>` / `AsMut<ffi::Model>` as parameter bounds. Fully removing them (typed `&X`/`&mut X` params + thick-wrapper/clone infra, as PR #277 does) is a **WS3-scale API refactor**, not a fold-in. → **tracked-deferred follow-up (post-WS6).**
- **Mesh accessors #257 / #118 / #256** (`eef0144`, AmityWilder + strizhkindenis + meisei4, closes #262): all 10 `RaylibMesh` slice accessors were unsound (`slice::from_raw_parts(null, n)` with no guard) — added null/zero guards returning `&[]`. Fixed a real length bug: `indices`/`indices_mut` used `vertexCount` → corrected to `triangleCount * 3`. Added 4 safe texcoords accessors (`texcoords`/`_mut`/`texcoords2`/`_mut`, `&[Vector2]`). TDD: a zeroed-Mesh null test asserts every accessor is empty. 58/58 unit + 16 doctests pass. (The `AsRef`/macro/param-bound rework from #256 was left out per the #277 deferral.)

## Formatting

- `2223f54`: `cargo fmt --all` over the 6 touched source files (rustfmt collapses the now-shorter expressions). Tree is `cargo fmt --all --check`-clean.

## Tracked-deferred follow-ups (carried past WS6-prep)

1. **Full #277 wrapper-soundness refactor** — remove the macro-generated `AsRef`/`AsMut`/`Deref`/`DerefMut` on pointer-owning wrappers; convert the `impl AsRef<ffi::X>` parameter bounds across drawing.rs/models.rs to typed references; add thick-wrapper/clone infrastructure. WS3-scale; its own future task.
2. **`get_gamepad_button_pressed` transmute** (`input.rs`) — `transmute::<u32, GamepadButton>` where `GamepadButton` is `#[repr(i32)]`; latent UB on an out-of-range C return. Pre-existing (not introduced by the sweep); candidate for a checked `from_repr`. Small follow-up.

## Outcome

`raylib-sys` + `raylib` are clippy-`-Dwarnings`-clean and fmt-clean; the API surface is settled. **WS6a may now add the `full` alias, `deny.toml`, the doc gate, and the layered workflows.** Commits `d1ed89f`→`2223f54`.
