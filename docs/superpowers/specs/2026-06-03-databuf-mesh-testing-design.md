# DataBuf + Mesh testing workstream — design

**Date:** 2026-06-03
**Status:** approved (maintainer sign-off via brainstorming session)
**Closes:** flexible-queue item 11 — WS8e review comments **8** ("There need to
be a lot more tests for edge cases and various uses of databuf. We can use the
software renderer mode here to ensure allocations. Maybe this is where we pull
in valgrind.") and **10** ("Double check that no rlgl functions are missing").

## Goal

1. Every public fn on the raylib-allocated wrapper family has edge-case +
   lifetime tests (or a documented untestable rationale), Tier-1 and Tier-2.
2. A complete per-fn disposition of the unwrapped `ffi::rl*` surface, with the
   trivial gaps wrapped and tested.
3. A memory-checking CI leg that gives real evidence for the lifetime
   invariants (drop-exactly-once, no double-free, no leak).

## Locked decisions

| Decision | Choice | Rationale |
|---|---|---|
| PR shape | **Two PRs, one workstream** | PR-A: wrapper-family tests + `alloc_from_clone` fix + `ci:` ASAN commit. PR-B: rlgl disposition table + trivial wraps + probes. Smaller reviews, independent reverts. |
| Memory-checking leg | **Extend existing ASAN leg** (no valgrind) | `sanitizers.yml` already runs ASAN/UBSAN over FFI (informational, D8). Add a Tier-1 ASAN+LSAN step (`detect_leaks=1`) over the new windowless test binary; keep `detect_leaks=0` for the render tests (window/driver-adjacent noise). Valgrind would duplicate ASAN's class coverage at higher CI cost — recorded as the disposition of the owner's valgrind suggestion. |
| rlgl wrap appetite | **Trivial gaps only** | Wrap only fns completing already-wrapped families (e.g. `rlVertex2i`); everything else dispositioned escape-hatch or future-work. Minimal API churn pre-6.0.0-publish. |

## Inventory (census of record at design time)

Wrapper family (the test subjects), from `grep ManuallyDrop|DataBuf` across
`raylib/src`:

| Wrapper | Where | Free path | Tests before |
|---|---|---|---|
| `DataBuf<T>` / `DataBuf<[T]>` / `RlManaged<T>` | `core/databuf.rs` (661 ln) | `ffi::MemFree` | 3 in-file |
| `compress_data` / `decompress_data` / `encode_data_base64` / `decode_data_base64` | `core/data.rs` | via `DataBuf` | 1 doctest |
| `ImageColors`, `ImagePalette` (`make_rslice!`) | `core/texture.rs` | `UnloadImageColors` / `UnloadImagePalette` | none |
| `RSliceGlyphInfo` (hand-rolled rslice), `Codepoints` (pub(crate)) | `core/text.rs` | `UnloadFontData` / `MemFree` | none |
| `FilePathList` / `DroppedFilePathList` | `core/file.rs` | `UnloadDirectoryFiles` / `UnloadDroppedFiles` | ~5 in-file (fake lists, `ManuallyDrop`-guarded) |
| Mesh slice accessors (8 pairs: vertices/normals/texcoords/texcoords2/tangents/colors/indices + mut) | `core/models.rs` | n/a (borrows) | null→empty-slice test |
| `ModelAnimations` | `core/models.rs` | `UnloadModelAnimations` | `integration_model_animations.rs` |

**Zero integration tests** under `raylib/tests/` touch `DataBuf`,
`FilePathList`, `ImageColors`, `ImagePalette`, or `RSliceGlyphInfo` — the gap
the owner flagged.

rlgl census (regenerate during execution; script in the done-note):
**161 `rl*` FFI fns** in bindings → **30 wrapped** in `raylib/src/rlgl/` →
**131 uncovered**, of which 6 are used internally elsewhere in the safe crate
(`rlGetMatrixModelview`, `rlGetMatrixProjection`, `rlGetShaderIdDefault`,
`rlGetShaderLocsDefault`, `rlSetMatrixModelview`, `rlSetMatrixProjection`).

## Known bug folded in: `DataBuf::<[T]>::alloc_from_clone`

`databuf.rs:529` — `alloc_from_clone(src: &[T]) where T: Copy` is byte-identical
to `alloc_from_copy` (its doctest even calls `alloc_from_copy`). Fix: make it a
true `T: Clone` element-wise clone. Loosening `Copy` → `Clone` is non-breaking;
the crate is pre-publish (6.0.0 not yet on crates.io). The element-wise loop
must be panic-safe: if a `clone()` panics mid-loop, the partially-initialized
buffer must not be dropped as `[T]`. Required behavior: write through
`MaybeUninit`, track the initialized prefix with a drop guard, and on unwind
drop the prefix then free the allocation (no leak, no uninit drop). Covered by
a panicking-clone test. CHANGELOG entry under the 6.0.0 block.

## Test architecture

**Tier rule:** Tier-2 (`with_headless`, SR harness) only when the test needs an
initialized raylib — `RaylibHandle` methods, GPU upload, rendering.
`MemAlloc`/`MemFree`, compression, base64 need no init → Tier-1.

### Tier-1 — new `raylib/tests/databuf_lifetimes.rs` + expanded in-file tests

| Subject | Edge cases |
|---|---|
| `DataBuf::<T>::alloc` / `alloc_from` / `alloc_from_clone` / `alloc_from_copy` | ZST → `AllocationError::ZeroBytes`; `alloc_from` error returns `val` intact (destructure the `(err, val)` tuple); `write` → value readable; `assume_init` roundtrip |
| `DataBuf::<[T]>::alloc` | `count = 0` → `ZeroBytes`; `Layout::array` overflow → `IntoUIntFailed`; byte-size > `u32::MAX` → `IntoUIntFailed`; large-but-valid (≥1 MB) succeeds and is fully writable |
| `realloc` | grow preserves prefix (init via `alloc_from_copy`, realloc up, assert prefix); shrink keeps prefix; `new_count = 0` → error tuple returns the **original buffer still usable** (read it afterward) |
| Drop semantics | slice drop runs element `Drop` exactly `len` times (slice version of `test_drop_value`); `into_inner` suppresses drop → manual `RlManaged::mem_free` (and the leak-if-forgotten case is what the LSAN leg catches) |
| View parity | `Deref` / `DerefMut` / `AsRef` / `AsMut` agree on contents |
| `slice_from_raw` guards | `count < 1` assertion (in-file `#[should_panic]`; fn is `pub(crate)`) |
| `data.rs` round-trips | compress→decompress at 1 B / 4 KB / 1 MB; decompress garbage → `CompressionError`; base64 encode→decode roundtrip; decode invalid input → `Base64Error::DecodeFailed`; empty-input behavior pinned for all four fns |
| `alloc_from_clone` fix | non-`Copy` clone-counting type: clone count == len, drop count == len after drop; plus the panic-safety behavior asserted/documented |
| `FilePathList` (in-file) | extend the existing fake-list tests: iter `count`/`len` parity, `next_back`/`nth` over the fake list |

Untestable without fault injection (documented in the done-note, not tested):
`ffi::MemAlloc` returning null (`AllocationError::NullAlloc` on the alloc
path) — requires near-OOM or allocator interposition (pitfall: don't fake it).

### Tier-2 — new `raylib/tests/render_alloc_lifetimes.rs` + extensions to existing files

All Tier-2 commands use the **verbatim** CLAUDE.md feature list:

```
cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION
```

| Subject | Tests |
|---|---|
| Mesh accessors | `gen_mesh_*` (upload needs context) → each of the 8 accessor pairs: len == `vertexCount` (or `triangleCount * 3`); mutate-through-`_mut` then read back; null attribute → empty slice for **all 8** accessors (extend the existing single-case test) |
| `ImageColors` / `ImagePalette` | from `gen_image_*` and from a `render_frame` readback: `len == w*h`; palette len ≤ requested max; values sane (probe a known pixel); drop runs (ASAN evidence) |
| `RSliceGlyphInfo` / `Codepoints` | load font data from bytes → glyph slice non-empty, indexable; drop through `UnloadFontData` clean under ASAN |
| `FilePathList` (real) | `rl.load_directory_files` on a tempdir with N known files → count == N, paths match; empty dir behavior pinned; drop via `UnloadDirectoryFiles` |
| `ModelAnimations` | review `integration_model_animations.rs`; extend only if a lifetime edge case is missing |
| DataBuf under init'd raylib | one test driving alloc/realloc/free **after** `with_headless` init, to exercise raylib's allocator in a real context (the owner's "use software renderer mode to ensure allocations") |

**File layout constraint:** nextest gives process-per-test, but the sanitizers
job runs plain `cargo test --test-threads=1` — any new Tier-2 file added to its
run-set must keep at most one `with_headless` per test fn (the existing files'
pattern) and accept that the job is `continue-on-error` informational.

## rlgl coverage audit

- **Census script** (bash one-liner over the bindgen output vs
  `ffi::rl*` callsites in `raylib/src/rlgl/`) recorded in the done-note so the
  audit is reproducible after future raylib bumps.
- **Disposition table** in the done-note: one row per uncovered fn →
  `wrapped` | `escape-hatch` | `future-work`. The escape-hatch line follows the
  policy already declared in `rlgl/mod.rs` docs: GL-object lifecycle
  (load/unload texture/shader/framebuffer/vertex-buffer), render batches,
  SSBO/compute, stereo, and framebuffer plumbing stay raw FFI.
- **Trivial wraps** (PR-B): `rlVertex2i` + any same-shape inconsistencies the
  table flags inside already-wrapped families (estimate 1–3 fns). Each wrap:
  trait method on `RaylibRlgl` matching existing style (`#[inline]`, SAFETY
  comment, doc), plus a Tier-2 probe in `render_rlgl.rs`.
- `rlgl/mod.rs` module docs gain a short **coverage policy** paragraph pointing
  at the disposition table.

## CI changes (own `ci:` commit inside PR-A)

In `sanitizers.yml` (stays informational / `continue-on-error`, per D8):

1. New step **"ASAN+LSAN — Tier-1 wrapper lifetime tests"**: same nightly
   `-Zsanitizer=address` build, `ASAN_OPTIONS=detect_leaks=1`, running
   `--test databuf_lifetimes` only (windowless → minimal leak-noise surface).
2. Add the new Tier-2 file(s) to the existing ASAN/UBSAN render-test steps'
   `--test` lists (`detect_leaks=0` unchanged there).

No new workflow files; no valgrind job (decision above). Per
`separate-ci-optimization-from-feature-work`, no cache/runner changes ride
along.

## Deliverables

- **PR-A `test:`** — Tier-1 + Tier-2 wrapper-family tests, `alloc_from_clone`
  fix (+ CHANGELOG line), `ci:` sanitizers commit.
- **PR-B `feat(rlgl):`** — disposition table, module-doc policy paragraph,
  trivial wraps + probes (+ CHANGELOG line if API added).
- **Done-note** `docs/superpowers/notes/databuf-mesh-testing-complete.md`:
  wrapper-fn × edge-case matrix (as executed), test census (3 in-file → N
  total), full 131-row disposition table + census script, memory-checking
  decision record (ASAN+LSAN chosen, valgrind declined and why).

## Done criteria

- Every public fn on the wrapper family has edge-case tests or a documented
  untestable rationale; Tier-1 + Tier-2 green with the verbatim commands.
- rlgl disposition table complete and committed; wraps it produced are tested.
- Memory-checking leg implemented (ASAN+LSAN step) and visible in a green
  (informational) sanitizers run.
- Both PRs merged to canonical `unstable`; done-note written; queue item 11
  closed.

## Pitfalls carried into planning

1. Single-init per process — all Tier-2 invocations through nextest; the
   sanitizers `cargo test` legs follow the existing one-`with_headless`-per-fn
   pattern and stay informational.
2. SR readback is BGRA + Y-flipped — use `render_frame` (normalized); feature
   list copied verbatim (all-5-modules link requirement).
3. Don't fake alloc-failure paths; test Rust-side validation
   (`AllocationError` variants) and document the rest.
4. No new public API for testability — `pub(crate)` / `#[cfg(test)]` seams
   only. The two deliberate API changes (`alloc_from_clone` semantics,
   `rlVertex2i`-class wraps) are flagged in CHANGELOG.
5. The audit's deliverable is the table; wrapping stays trivial-only
   (scope-discovery rule: fix the genuine gap, record the rest).
