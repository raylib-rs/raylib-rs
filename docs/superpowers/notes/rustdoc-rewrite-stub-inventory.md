# Rustdoc rewrite — per-file stub inventory

Snapshot of single-line `///` stubs (added by WS6a, not enriched by
WS7) that the Wave 1 implementers will enrich. Each Wave-1 dispatch
brief includes its file's section verbatim.

**Total stubs across all files: 207** (spec quotes ~183; actual count
is ~13% higher because WS7's enrichment primarily targeted items that
*already* had docs before WS6a — `Color`, `Rectangle`, `Image`,
`Texture2D`, `RenderTexture2D`, `Mesh`, `Model`, `Material`,
`ModelAnimations`, `Wave`, `Sound`, `Music`, `AudioStream`, `Shader`,
`Font`, the math types — rather than WS6a's minimal one-liners.
Only **one** WS6a stub was enriched by WS7: `RaylibDraw` in
`core/drawing.rs`. Spec §5 may want to update `~183` → `~207`).

**Per-batch totals:**

| Batch | Files | Stubs |
|-------|-------|-------|
| B1 | `core/error.rs` | 96 |
| B2 | `ease.rs` | 28 |
| B3 | `core/drawing.rs` | 16 |
| B4 | `core/camera.rs` | 13 |
| B5 | `core/models.rs` | 11 |
| B6 | `core/window.rs` | 7 |
| B7 | `core/mod.rs` | 5 |
| B8 | `core/macros.rs` | 4 (special; macro-internal `#[allow(missing_docs)]` — not `///` stubs) |
| B9 + B10 | 13 small files | 27 |
| **Total** | **21 files** | **207** |

**Source of truth for "WS6a-added" status:** the three WS6a commits
`5a297e8` (Batch 1 — error.rs), `131a1b0` (Batch 2 — ease/drawing/
camera/models/window/mod/macros), `3a928b3` (Batch 3 — 13 small
files). Anything in those three diffs that is still in stub state
today (one `///` line, no `# Examples`, no multi-paragraph prose) is
in scope.

**WS7-enriched items (out of scope, do NOT re-enrich):**
- `lib.rs` crate-level + `prelude.rs`
- `core/window.rs`: `RaylibHandle`, `RaylibThread`, `RaylibBuilder`
  (NB: these are not WS6a stubs; they live in `core/mod.rs`)
- `core/drawing.rs`: `RaylibDraw` trait
- `core/mod.rs`: `Color`, `Rectangle` (NB: these are not WS6a stubs;
  the types themselves live in `raylib-sys` not `mod.rs`)
- `core/texture.rs`: `Image`, `Texture2D`, `RenderTexture2D`
- `core/models.rs`: `Mesh`, `Model`, `Material`, `ModelAnimations`
- `core/audio.rs`: `RaylibAudio`, `Wave`, `Sound`, `Music`, `AudioStream`
- `core/shaders.rs`: `Shader`
- `core/text.rs`: `Font`
- `core/math.rs`: `Vector2/3/4`, `Matrix`, `Quaternion`
- `core/collision.rs`: module-level family example
- `test_harness.rs`: module-level + BGRA/Y-flip quirk note
- `rgui/mod.rs`: module-level
- `rlgl/mod.rs`: module-level

---

## raylib/src/core/error.rs (Wave-1 dispatch B1)

WS6a-added stubs: **96** (target: enrich all to Template A — variant
prose with `**Cause:**` + `**Recovery:**` paragraphs; parent enums
keep their existing one-liner AND get a Flavor-3 `# Examples` block).

WS7-enriched items in this file: **none**.

**Parent enums (require Flavor-3 `# Examples` block on the enum itself; one example per parent enum, NOT per variant):**

| Line | Enum |
|------|------|
| 7 | `AudioInitError` |
| 18 | `ExportWaveError` |
| 29 | `LoadSoundError` |
| 64 | `UpdateAudioStreamError` |
| 90 | `AllocationError` |
| 105 | `InvalidMeshError` |
| 134 | `GenMeshError` |
| 145 | `CompressionError` |
| 153 | `Base64Error` |
| 164 | `LoadModelError` |
| 178 | `LoadModelAnimError` |
| 189 | `SetMaterialError` |
| 200 | `LoadMaterialError` |
| 211 | `LoadFontError` |
| 230 | `InvalidImageError` |
| 262 | `UpdateTextureError` |
| 281 | `LoadTextureError` |
| 304 | `RaylibError` |

**Variant stubs (Template A: summary + Cause + Recovery):**

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 5 | `AudioInitError` (enum doc) | parent-summary | parent enum line 7 |
| 8 | `AudioInitError::DoubleInit` | Template A | |
| 11 | `AudioInitError::InitFailed` | Template A | |
| 16 | `ExportWaveError` (enum doc) | parent-summary | parent enum line 18 |
| 19 | `ExportWaveError::QoaBadSamples` | Template A | tuple variant |
| 22 | `ExportWaveError::ExportFailed` | Template A | |
| 27 | `LoadSoundError` (enum doc) | parent-summary | parent enum line 29 |
| 30 | `LoadSoundError::LoadFailed` | Template A | struct variant |
| 33 | `LoadSoundError::LoadFailed::path` | field-doc | field of struct variant |
| 36 | `LoadSoundError::LoadFromWaveFailed` | Template A | |
| 39 | `LoadSoundError::LoadWaveFromFileFailed` | Template A | struct variant |
| 42 | `LoadSoundError::LoadWaveFromFileFailed::path` | field-doc | |
| 45 | `LoadSoundError::Null` | Template A | |
| 48 | `LoadSoundError::LoadMusicFromFileFailed` | Template A | struct variant |
| 51 | `LoadSoundError::LoadMusicFromFileFailed::path` | field-doc | |
| 54 | `LoadSoundError::MusicNull` | Template A | |
| 59 | `UpdateAudioStreamError` (enum doc) | parent-summary (multi-line already; leave existing prose, add Flavor-3 `# Examples`) | parent enum line 64; this enum already has a multi-line note block from WS6a-prior; orchestrator triage: treat as enriched-summary + add `# Examples` only |
| 65 | `UpdateAudioStreamError::SampleSizeMismatch` | Template A | struct variant |
| 68 | `UpdateAudioStreamError::SampleSizeMismatch::expected` | field-doc | |
| 70 | `UpdateAudioStreamError::SampleSizeMismatch::provided` | field-doc | |
| 73 | `UpdateAudioStreamError::TooManyFrames` | Template A | struct variant |
| 76 | `UpdateAudioStreamError::TooManyFrames::max` | field-doc | |
| 78 | `UpdateAudioStreamError::TooManyFrames::provided` | field-doc | |
| 81 | `UpdateAudioStreamError::CallbackSlotBusy` | Template A | |
| 88 | `AllocationError` (enum doc) | parent-summary | parent enum line 90 |
| (91) | `AllocationError::NullAlloc` | already-enriched | pre-existing multi-line doc (see line 91); NOT a WS6a stub |
| (94) | `AllocationError::IntoUIntFailed` | already-enriched | pre-existing multi-line doc; NOT a WS6a stub |
| (98) | `AllocationError::ZeroBytes` | already-enriched | pre-existing one-liner but with `[MemAlloc]` intra-doc link; orchestrator triage: counts as enriched? If R1 disagrees, treat as Template A |
| 103 | `InvalidMeshError` (enum doc) | parent-summary | parent enum line 105 |
| 106 | `InvalidMeshError::TrianglePointMiscount` | Template A | |
| 109 | `InvalidMeshError::IndexOutOfBounds` | Template A | |
| 112 | `InvalidMeshError::VertexUnindexible` | Template A | tuple variant with `TryFromIntError` payload |
| 115 | `InvalidMeshError::TexcoordsMiscount` | Template A | |
| 118 | `InvalidMeshError::Texcoords2Miscount` | Template A | |
| 121 | `InvalidMeshError::NormalsMiscount` | Template A | |
| 124 | `InvalidMeshError::TangentsMiscount` | Template A | |
| 127 | `InvalidMeshError::ColorsMiscount` | Template A | |
| 132 | `GenMeshError` (enum doc) | parent-summary | parent enum line 134 |
| 135 | `GenMeshError::InvalidMesh` | Template A | `#[from] InvalidMeshError` |
| 138 | `GenMeshError::Allocation` | Template A | `#[from] AllocationError` |
| 143 | `CompressionError` (enum doc) | parent-summary | parent enum line 145 |
| 146 | `CompressionError::CompressionFailed` | Template A | |
| 151 | `Base64Error` (enum doc) | parent-summary | parent enum line 153 |
| 154 | `Base64Error::DecodeFailed` | Template A | |
| 157 | `Base64Error::EncodeFailed` | Template A | |
| 162 | `LoadModelError` (enum doc) | parent-summary | parent enum line 164 |
| 165 | `LoadModelError::LoadFromFileFailed` | Template A | struct variant |
| 168 | `LoadModelError::LoadFromFileFailed::path` | field-doc | |
| 171 | `LoadModelError::LoadFromMeshFailed` | Template A | |
| 176 | `LoadModelAnimError` (enum doc) | parent-summary | parent enum line 178 |
| 179 | `LoadModelAnimError::NoAnimationsLoaded` | Template A | struct variant |
| 182 | `LoadModelAnimError::NoAnimationsLoaded::path` | field-doc | |
| 187 | `SetMaterialError` (enum doc) | parent-summary | parent enum line 189 |
| 190 | `SetMaterialError::MeshIdOutOfBounds` | Template A | |
| 193 | `SetMaterialError::MaterialIdOutOfBounds` | Template A | |
| 198 | `LoadMaterialError` (enum doc) | parent-summary | parent enum line 200 |
| 201 | `LoadMaterialError::NoneLoaded` | Template A | struct variant |
| 204 | `LoadMaterialError::NoneLoaded::path` | field-doc | |
| 209 | `LoadFontError` (enum doc) | parent-summary | parent enum line 211 |
| 212 | `LoadFontError::LoadFromFileFailed` | Template A | struct variant |
| 217 | `LoadFontError::LoadFromFileFailed::path` | field-doc | |
| 220 | `LoadFontError::LoadFromImageFailed` | Template A | |
| 223 | `LoadFontError::LoadFromMemoryFailed` | Template A | |
| 228 | `InvalidImageError` (enum doc) | parent-summary | parent enum line 230 |
| 231 | `InvalidImageError::ZeroWidth` | Template A | |
| 234 | `InvalidImageError::ZeroHeight` | Template A | |
| 237 | `InvalidImageError::NullData` | Template A | |
| 240 | `InvalidImageError::NullDataFromFile` | Template A | |
| 243 | `InvalidImageError::InvalidFile` | Template A | |
| 246 | `InvalidImageError::NullDataFromMemory` | Template A | |
| 249 | `InvalidImageError::NullDataFromTexture` | Template A | |
| 252 | `InvalidImageError::UnsupportedFormat` | Template A | |
| 255 | `InvalidImageError::NonSquareKernel` | Template A | |
| 260 | `UpdateTextureError` (enum doc) | parent-summary | parent enum line 262 |
| 263 | `UpdateTextureError::WrongDataSize` | Template A | struct variant |
| 266 | `UpdateTextureError::WrongDataSize::expect` | field-doc | |
| 268 | `UpdateTextureError::WrongDataSize::actual` | field-doc | |
| 271 | `UpdateTextureError::OutOfBounds` | Template A | |
| 274 | `UpdateTextureError::NegativeSize` | Template A | |
| 279 | `LoadTextureError` (enum doc) | parent-summary | parent enum line 281 |
| 282 | `LoadTextureError::TextureFromFileFailed` | Template A | struct variant |
| 285 | `LoadTextureError::TextureFromFileFailed::path` | field-doc | |
| 288 | `LoadTextureError::CubemapFromImageFailed` | Template A | |
| 291 | `LoadTextureError::TextureFromImageFailed` | Template A | |
| 294 | `LoadTextureError::CreateRenderTextureFailed` | Template A | |
| 297 | `LoadTextureError::InvalidData` | Template A | |
| 302 | `RaylibError` (enum doc) | parent-summary | parent enum line 304 |
| 305 | `RaylibError::AudioInit` | Template A | `#[from]` variant |
| 308 | `RaylibError::ExportWave` | Template A | `#[from]` variant |
| 311 | `RaylibError::LoadSound` | Template A | `#[from]` variant |
| 314 | `RaylibError::Allocation` | Template A | `#[from]` variant |
| 317 | `RaylibError::Compression` | Template A | `#[from]` variant |
| 320 | `RaylibError::LoadModel` | Template A | `#[from]` variant |
| 323 | `RaylibError::LoadModelAnim` | Template A | `#[from]` variant |
| 326 | `RaylibError::SetMaterial` | Template A | `#[from]` variant |
| 329 | `RaylibError::LoadMaterial` | Template A | `#[from]` variant |
| 332 | `RaylibError::LoadFont` | Template A | `#[from]` variant |
| 335 | `RaylibError::InvalidImage` | Template A | `#[from]` variant |
| 338 | `RaylibError::UpdateTexture` | Template A | `#[from]` variant |
| 341 | `RaylibError::LoadTexture` | Template A | `#[from]` variant |

**Inventory counts for B1:** 18 parent-enum summaries (already
present, keep + add `# Examples`) + 96 variant/field stubs = 114
items requiring touch. Spec §5 quotes "96" — that refers to the
variant+field count; the 18 parent-enum summaries are not "stubs"
per the strict definition (they already exist and serve as headers),
but each one needs a Flavor-3 `# Examples` block added per §4.2's
Template A directive. **Orchestrator triage:** the brief should
clarify whether the 96 count is correct (variants/fields only) or
should be 114 (including parent-enum `# Examples` additions).

**Anomalies / judgment calls:**
- `AllocationError::NullAlloc`, `::IntoUIntFailed`, `::ZeroBytes`
  (lines 91, 94, 98) already had multi-line docs before WS6a — they
  contain intra-doc links to `[MemAlloc]`. Not strictly WS6a stubs;
  may be best left as-is. (Counted in the 96 above for completeness.)
- `UpdateAudioStreamError` (line 59–63) parent-enum doc has a
  multi-paragraph note already; this is "partially enriched" prose
  with a `**Notes**` block. Flag for orchestrator: leave prose or
  rewrite to match Template A's parent-enum summary shape?

---

## raylib/src/ease.rs (Wave-1 dispatch B2)

WS6a-added stubs: **28** (target: Flavor 1 — pure-compute `# Examples`
that execute under `cargo test --doc`).

WS7-enriched items in this file: **none**.

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 105 | `linear_none` | Flavor 1 | NB: existing tests in `mod tests` already cover boundaries — examples can mirror those assertions |
| 109 | `linear_in` | Flavor 1 | |
| 113 | `linear_out` | Flavor 1 | |
| 117 | `linear_in_out` | Flavor 1 | |
| 122 | `sine_in` | Flavor 1 | |
| 126 | `sine_out` | Flavor 1 | |
| 130 | `sine_in_out` | Flavor 1 | |
| 135 | `circ_in` | Flavor 1 | |
| 141 | `circ_out` | Flavor 1 | |
| 147 | `circ_in_out` | Flavor 1 | |
| 158 | `cubic_in` | Flavor 1 | |
| 164 | `cubic_out` | Flavor 1 | |
| 170 | `cubic_in_out` | Flavor 1 | |
| 181 | `quad_in` | Flavor 1 | |
| 187 | `quad_out` | Flavor 1 | spec §4.3 cites this as the canonical Flavor-1 example |
| 193 | `quad_in_out` | Flavor 1 | |
| 203 | `expo_in` | Flavor 1 | |
| 212 | `expo_out` | Flavor 1 | |
| 221 | `expo_in_out` | Flavor 1 | |
| 237 | `back_in` | Flavor 1 | |
| 244 | `back_out` | Flavor 1 | |
| 251 | `back_in_out` | Flavor 1 | known boundary quirk per the existing test comment — examples should test mid-interpolation, not boundary |
| 265 | `bounce_out` | Flavor 1 | |
| 282 | `bounce_in` | Flavor 1 | |
| 287 | `bounce_in_out` | Flavor 1 | |
| 296 | `elastic_in` | Flavor 1 | |
| 314 | `elastic_out` | Flavor 1 | |
| 330 | `elastic_in_out` | Flavor 1 | |

**Anomalies / judgment calls:** none. All 28 are pure-math easing
fns; Flavor 1 is a clean fit. The `EaseFn` type alias (line 26) and
`Tween` struct (line 29) are pre-existing and already documented;
out of scope.

---

## raylib/src/core/drawing.rs (Wave-1 dispatch B3)

WS6a-added stubs: **16** (down from spec's quoted 17; `RaylibDraw`
trait on line 546 was enriched by WS7 with full prose + a `# Examples`
block).

WS7-enriched items in this file: `RaylibDraw` (line 523/546).

Target: Flavor 2 (harness-runnable doctests where the item is a
drawing primitive; otherwise Flavor 3). Mode-guard structs and their
extension traits are documentation-only; their `# Examples` should
demonstrate the `with begin_X / end_X` RAII pattern using
`no_run`-style demos with `# #[cfg(feature = "software_renderer")]`
gates where they exercise pixel state.

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 63 | `RaylibDrawHandle` | Flavor 3 | RAII guard for `begin_drawing`; demo via `no_run` |
| 110 | `RaylibTextureMode` | Flavor 2 or 3 | RAII guard for render-texture; harness-friendly |
| 132 | `RaylibTextureModeExt` | Flavor 3 | trait — example via guard usage |
| 172 | `RaylibVRMode` | Flavor 3 | VR — no harness coverage; `no_run` only |
| 191 | `RaylibVRModeExt` | Flavor 3 | trait — `no_run` |
| 227 | `RaylibMode2D` | Flavor 2 or 3 | 2D camera; harness-friendly |
| 247 | `RaylibMode2DExt` | Flavor 3 | trait — example via guard usage |
| 287 | `RaylibMode3D` | Flavor 3 | 3D camera; window-required |
| 307 | `RaylibMode3DExt` | Flavor 3 | trait — example via guard usage |
| 348 | `RaylibShaderMode` | Flavor 3 | shader — window-required |
| 369 | `RaylibShaderModeExt` | Flavor 3 | trait — example via guard usage |
| 405 | `RaylibBlendMode` | Flavor 2 or 3 | blend mode; harness-friendly |
| 425 | `RaylibBlendModeExt` | Flavor 3 | trait — example via guard usage |
| 461 | `RaylibScissorMode` | Flavor 2 or 3 | scissor; harness-friendly |
| 481 | `RaylibScissorModeExt` | Flavor 3 | trait — example via guard usage |
| 1714 | `RaylibDraw3D` | Flavor 3 | 3D drawing trait; window-required |

**Anomalies / judgment calls:**
- `RaylibDraw` (line 523/546) is WS7-enriched and OUT OF SCOPE. Leave
  untouched.
- Per spec §5, Batch 2 drawing.rs target is "Flavor 2 (harness-
  runnable)", but several items here are window-only (VR, 3D camera,
  shader mode). Implementer should use Flavor 2 where harness-
  exerciseable, Flavor 3 where window-required.

---

## raylib/src/core/camera.rs (Wave-1 dispatch B4)

WS6a-added stubs: **13** (target: mix of Flavor 1 for math accessors
and Flavor 3 for camera-init / mutating ops).

WS7-enriched items in this file: **none**.

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 98 | `Camera3D::camera_type` | Flavor 1 | pure accessor |
| 129 | `Camera3D::forward` | Flavor 1 | math accessor |
| 136 | `Camera3D::up` | Flavor 1 | math accessor |
| 143 | `Camera3D::move_forward` | Flavor 3 | mutating op |
| 149 | `Camera3D::move_up` | Flavor 3 | mutating op |
| 155 | `Camera3D::move_right` | Flavor 3 | mutating op |
| 161 | `Camera3D::move_to_target` | Flavor 3 | mutating op |
| 167 | `Camera3D::yaw` | Flavor 3 | mutating op |
| 173 | `Camera3D::pitch` | Flavor 3 | mutating op |
| 193 | `Camera3D::roll` | Flavor 3 | mutating op |
| 199 | `Camera3D::view_matrix` | Flavor 1 | math accessor |
| 205 | `Camera3D::projection_matrix` | Flavor 1 | math accessor |
| 216 | `Camera3D::update_camera_pro` | Flavor 3 | mutating op |

**Anomalies / judgment calls:**
- `Camera3D::perspective` (line 109), `::orthographic` (line 123),
  and `::update_camera` (line 213) are pre-existing doc-stubs from
  before WS6a (the WS6a diff did not touch them); strictly out of
  scope but the Wave-1 implementer may want to update them in passing
  for consistency. Flag for orchestrator: explicitly allow drive-by
  enrichment on the 3 pre-existing one-liners, or strictly leave
  alone? Recommendation: enrich them too — they're in the same impl
  block and visually adjacent.

---

## raylib/src/core/models.rs (Wave-1 dispatch B5)

WS6a-added stubs: **11** (target: Flavor 3 — model-loading and
mesh/animation ops require a window).

WS7-enriched items in this file: `Model`, `Mesh`, `Material`,
`ModelAnimations` (the top-level types).

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 363 | `RaylibModel` trait | Flavor 3 | extension trait |
| 372 | `RaylibModel::set_transform` | Flavor 3 | trait method |
| 389 | `RaylibModel::meshes_mut` | Flavor 3 | trait method |
| 527 | `RaylibMesh` trait | Flavor 3 | extension trait |
| 879 | `RaylibMaterial` trait | Flavor 3 | extension trait |
| 936 | `FramePoseIter` | Flavor 1 or 3 | iterator type — pure-data, but ModelAnimation behind it is window-loaded |
| 1014 | `FramePoseIterMut` | Flavor 1 or 3 | iterator type |
| 1111 | `RaylibModelAnimation` trait | Flavor 3 | extension trait |
| 1130 | `RaylibModelAnimation::frame_poses_iter` | Flavor 3 | trait method |
| 1160 | `RaylibModelAnimation::frame_poses_iter_mut` | Flavor 3 | trait method |
| 1254 | `MeshBuilder` | Flavor 3 | builder — window required for upload |

**Anomalies / judgment calls:**
- The `RaylibModel`, `RaylibMesh`, `RaylibMaterial`,
  `RaylibModelAnimation` extension traits are conceptual companions
  to the WS7-enriched parent types. Their `# Examples` blocks should
  reference the parent type's example, not duplicate it.
- `FramePoseIter` / `FramePoseIterMut` are pure-data iterators but
  the data they iterate is window-loaded; either flavor works.

---

## raylib/src/core/window.rs (Wave-1 dispatch B6)

WS6a-added stubs: **7** (target: Flavor 3 — window/monitor
inspection requires a window).

WS7-enriched items in this file: **none** (note: `RaylibHandle`,
`RaylibThread`, `RaylibBuilder` were enriched in WS7 but they live
in `core/mod.rs`, not `core/window.rs`).

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 15 | `MonitorInfo::width` | field-doc | struct field; consider expanding `MonitorInfo`'s parent-struct doc with a `# Examples` showing monitor enumeration |
| 17 | `MonitorInfo::height` | field-doc | |
| 19 | `MonitorInfo::physical_width` | field-doc | |
| 21 | `MonitorInfo::physical_height` | field-doc | |
| 23 | `MonitorInfo::name` | field-doc | |
| 25 | `MonitorInfo::position` | field-doc | |
| 29 | `WindowState` | Flavor 3 | bitmask wrapper — example via `get_window_state` + flag inspection |

**Anomalies / judgment calls:**
- All 6 `MonitorInfo` field docs are single-line and adequately
  descriptive; per §4.3 "Skip `# Examples`" guidance for items where
  examples are contrived, these may stay as one-liners but should be
  expanded slightly (e.g., "Monitor width in pixels (virtual/logical)"
  → "Monitor width in **virtual** pixels — distinct from
  `physical_width` which is reported in millimetres."). Orchestrator
  triage: enrich prose-only, no examples?

---

## raylib/src/core/mod.rs (Wave-1 dispatch B7)

WS6a-added stubs: **5** (4 module-level `pub mod` docs + 1 macro
doc).

WS7-enriched items in this file: `RaylibThread` (line 188),
`RaylibHandle` (line 208), `RaylibBuilder` (line 250).

Per spec §5: "Flavor 1 (Color/Rectangle helpers)". **Drift note:**
the WS6a stubs in `mod.rs` are NOT Color/Rectangle helpers — they're
module-level docs + the `rstr!` macro. `Color` and `Rectangle` live
in `raylib-sys` and are re-exported. Spec §5's flavor assignment for
this file is mis-targeted; Template C (module-level doc) is the
correct fit for items 5/7/13/18 and Flavor-skip-with-prose for the
macro.

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 5 | `pub mod automation` | Template C | module-level — expand to "Automation event recording and playback. See the *Automation* chapter of the book." |
| 7 | `pub mod callbacks` | Template C | module-level |
| 13 | `pub mod color` (inline mod) | Template C | inline `pub mod color { pub use crate::ffi::Color; }` |
| 18 | `pub mod data` | Template C | module-level |
| 171 | `rstr!` macro | Skip `# Examples` per §4.3, prose-only | already has descriptive one-liner; expand with usage note inline in prose |

**Anomalies / judgment calls:**
- Spec §5's `Flavor 1 (Color/Rectangle helpers)` for `mod.rs` is
  inaccurate. Orchestrator should clarify when dispatching B7 — use
  Template C for module docs + prose-only for `rstr!`.

---

## raylib/src/core/macros.rs (Wave-1 dispatch B8)

WS6a-added "stubs": **4** (special case — these are
`#[allow(missing_docs)]` attributes inside macro templates, NOT `///`
doc stubs. They suppress the missing-docs lint on macro-generated
struct fields, where attaching a `///` to a `pub(crate) $t`
positional field is awkward.)

WS7-enriched items in this file: **none**.

Per spec §5: "Skip `# Examples`; prose-only". This file is macro
plumbing; the public-API stubs do not actually exist as items
implementers can attach `///` blocks to in a useful way.

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 9 | `make_thin_wrapper` arm-1 `#[allow(missing_docs)]` | special — leave alone or improve macro contract docs | the macro itself already has no doc-comment; consider adding a brief macro-level `///` describing what `make_thin_wrapper!` generates |
| 19 | `make_thin_wrapper` arm-2 `#[allow(missing_docs)]` | same | |
| 35 | `make_thin_wrapper_lifetime` arm-1 `#[allow(missing_docs)]` | same | |
| 43 | `make_thin_wrapper_lifetime` arm-2 `#[allow(missing_docs)]` | same | |
| 141 | `make_rslice` `#[allow(missing_docs)]` | same | NB: WS6a done-note said "4 macros.rs" — this is a 5th `#[allow(missing_docs)]` instance, but the original commit `131a1b0` shows 5 added (4 to `make_thin_wrapper*` + 1 to `make_rslice`). Spec §5's count of 4 is slightly off; actual count is 5 |

**Anomalies / judgment calls:**
- These are not `///` doc-block stubs — they are `#[allow(missing_docs)]`
  attribute insertions inside `macro_rules!` arms. Standard Template B
  prose enrichment does not apply. Per §4.3 "Skip `# Examples`", the
  Wave-1 implementer should add `///` macro-level docs to each of
  `make_thin_wrapper!`, `make_thin_wrapper_lifetime!`,
  `impl_wrapper!`, `gen_from_raw_wrapper!`, `deref_impl_wrapper!`,
  `make_rslice!`, `impl_rslice!` (the publicly-exposed macros) and
  consider whether the `#[allow(missing_docs)]` can be removed if the
  generated struct fields receive a doc-comment via the `$(#[$attrs])*`
  passthrough.
- **Orchestrator triage:** consider rolling this file into a
  small-files batch (B9/B10) — its scope is degenerate and the
  enrichment work is qualitatively different from the rest of B8's
  charter.

---

## Batch 3 — 13 small files (Wave-1 dispatches B9 + B10)

Total WS6a stubs across all Batch 3: **27** (matches WS6a done-note).
Subagent decides flavor per item. Suggested split:

- **B9** (math/data-adjacent, ~14 stubs): `consts.rs`, `core/math.rs`,
  `core/misc.rs`, `core/file.rs`, `core/input.rs`, `lib.rs`,
  `core/automation.rs`.
- **B10** (audio/callbacks/render-adjacent, ~13 stubs):
  `core/audio.rs`, `core/callbacks.rs`,
  `core/callbacks/audio_stream_callback.rs`, `core/shaders.rs`,
  `core/text.rs`, `core/texture.rs`.

### raylib/src/consts.rs (B9, 2 stubs)

WS7-enriched items in this file: **none**.

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 24 | `MAX_MATERIAL_MAPS` | Flavor 1 | pure constant; can show `assert_eq!(MAX_MATERIAL_MAPS, 12)` |
| 26 | `MAX_SHADER_LOCATIONS` | Flavor 1 | pure constant |

### raylib/src/core/audio.rs (B10, 2 stubs)

WS7-enriched items in this file: `RaylibAudio`, `Wave`, `Sound`,
`Music`, `AudioStream` (lines elsewhere; these are out of scope).

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 99 | `WaveSamples` | Flavor 3 | RAII wrapper for `LoadWaveSamples`; window-required to load a Wave |
| 828 | `SoundAlias` | Flavor 3 | borrowed alias to a Sound; window-required |

### raylib/src/core/automation.rs (B9, 1 stub)

WS7-enriched items in this file: **none**.

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 9 | `AutomationEventIter` | Flavor 3 | iterator over loaded automation events; window-required for context |

### raylib/src/core/callbacks.rs (B10, 3 stubs)

WS7-enriched items in this file: **none**.

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 15 | `pub mod audio_stream_callback` | Template C | module-level |
| 127 | `SetLogError<'a>` | Flavor 3 | error returned from callback registration |
| 365 | `attach_audio_stream_processor_to_music` | Flavor 3 | window+audio-device required |

### raylib/src/core/callbacks/audio_stream_callback.rs (B10, 1 stub)

WS7-enriched items in this file: **none**.

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 64 | `unset_audio_stream_callback` | Flavor 3 | window+audio required |

### raylib/src/core/file.rs (B9, 1 stub)

WS7-enriched items in this file: **none**.

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 9 | `FilePathIter` | Flavor 3 | iterator over a `FilePathList`; loaded via window |

### raylib/src/core/input.rs (B9, 1 stub)

WS7-enriched items in this file: **none**.

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 417 | `key_from_i32` | Flavor 1 | pure conversion fn `i32 -> Option<KeyboardKey>` |

### raylib/src/core/math.rs (B9, 3 stubs)

WS7-enriched items in this file: `Vector2`, `Vector3`, `Vector4`,
`Matrix`, `Quaternion` (these are NOT in `core/math.rs` — they live
in `raylib-sys/src/vector_math.rs`; out of scope).

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 130 | `Ray::new` | Flavor 1 | constructor — pure-math |
| 139 | `pub type Rectangle` | Flavor 1 or Template C | type alias; explain why it's re-exported from `ffi` here |
| 155 | `BoundingBox::new` | Flavor 1 | constructor — pure-math |

### raylib/src/core/misc.rs (B9, 3 stubs)

WS7-enriched items in this file: **none**.

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 44 | `RandSeqIterator` | Flavor 1 | pure iterator |
| 115 | `AsF32` trait | Flavor 1 | numeric conversion trait |
| 117 | `AsF32::as_f32` | Flavor 1 | trait method |

### raylib/src/core/shaders.rs (B10, 3 stubs)

WS7-enriched items in this file: `Shader` (the type itself is
enriched in `texture.rs`/`shaders.rs` parent file; verify per WS7
done-note).

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 147 | `ShaderV` trait | Flavor 3 | shader-uniform trait; window+shader required |
| 149 | `ShaderV::UNIFORM_TYPE` const | Flavor 3 | trait const |
| 327 | `RaylibShader` trait | Flavor 3 | shader extension trait |

### raylib/src/core/text.rs (B10, 3 stubs)

WS7-enriched items in this file: `Font`.

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 64 | `RSliceGlyphInfo` | Flavor 3 | RAII slice from `LoadFontData`; window+font required |
| 323 | `RaylibFont` trait | Flavor 3 | font extension trait |
| 409 | `Font::make_weak` | Flavor 3 | converts owned to weak |

### raylib/src/core/texture.rs (B10, 3 stubs)

WS7-enriched items in this file: `Image`, `Texture2D`,
`RenderTexture2D`.

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 229 | `RaylibRenderTexture2D` trait | Flavor 3 | render-texture extension trait |
| 1070 | `Image::gen_image_text` | Flavor 2 | image generation — harness-runnable |
| 1237 | `RaylibTexture2D` trait | Flavor 3 | texture extension trait |

### raylib/src/lib.rs (B9, 1 stub)

WS7-enriched items in this file: crate-level `//!` doc.

| Line | Item | Flavor | Notes |
|------|------|--------|-------|
| 67 | `pub mod core` | Template C | module-level — "Core raylib functionality — window, drawing, input, audio, and other subsystems." Expand with a paragraph and `# See also` referencing the book. |

---

## Summary — orchestrator action items

1. **Total count (207) is ~13% above spec §5's "~183".** Trigger
   the "differs by more than 10%, flag in inventory's intro" rule.
   Spec §5's `~183` is undercount; recommend updating it to ~207
   when reconciling. The drift is because WS7 enriched mostly
   pre-existing-doc items (Color, Image, etc.) rather than WS6a
   one-liners — only **one** WS6a stub (`RaylibDraw`) was enriched
   by WS7.

2. **Spec §5 flavor mis-assignments to clarify in Wave-1 briefs:**
   - **B7 (`core/mod.rs`):** spec says "Flavor 1 (Color/Rectangle
     helpers)". Actual stubs are module-level docs + the `rstr!`
     macro; use Template C + prose-only.
   - **B8 (`core/macros.rs`):** spec says "Skip `# Examples`;
     prose-only" for 4 items. Actual file has 5
     `#[allow(missing_docs)]` annotations inside macro arms (not
     `///` stubs); the Wave-1 implementer should add macro-level
     `///` docs to each public macro AND consider whether the
     `#[allow]`s can be removed.

3. **B1 (`core/error.rs`) item-count nuance:** 96 variant/field
   stubs as stated, **plus** 18 parent-enum summaries that need
   `# Examples` blocks added per Template A. Wave-1 brief should
   make this explicit (96 enrichments + 18 example-block additions
   = 114 touch points; not 96).

4. **B3 (`core/drawing.rs`) headcount:** spec quotes 17; actual is
   16 (`RaylibDraw` enriched by WS7).

5. **Drive-by enrichment of pre-existing one-liners adjacent to
   WS6a stubs (B4 camera.rs, B6 window.rs MonitorInfo fields, B1
   AllocationError variants):** orchestrator should explicitly
   allow or forbid this in each Wave-1 brief to avoid scope drift.
   Recommendation: allow drive-by in B4 and B6 where the item is
   visually adjacent and the enrichment is trivial; forbid in B1
   (the pre-existing `AllocationError` variants already use a
   distinct style with intra-doc links).

6. **B8 (`core/macros.rs`) may belong in a Batch-3 dispatch** given
   its degenerate scope. Orchestrator triage.
