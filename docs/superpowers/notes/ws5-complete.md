# WS5 complete — raygui 6.0 parity (broad rework) + safe immediate-mode rlgl

**Status:** DONE. Branch `6.0-rc`, pushed to `fork`. raygui at 6.0 parity via a broad rework, plus a safe immediate-mode `rlgl` module; both render-verified with the WS4 headless harness; 3-OS CI green.

Spec: `docs/superpowers/specs/2026-05-26-ws5-raygui-rlgl-design.md`. Plans: `plans/2026-05-26-ws5-prep-harness-normalization.md`, `…-ws5a-raygui-rework.md`, `…-ws5b-rlgl-immediate-mode.md`. String-convention research: `docs/research/2026-05-26-A-raygui-string-ergonomics/public.md`.

## WS5-prep — harness readback normalization
- `test_harness::render_frame` now returns **true top-left RGBA** (flips Y + swaps R↔B internally); `render_frame_raw` keeps the raw rlsw readback (BGRA + Y-inverted). New render tests use natural draw coords/colors. `assert_pixel` still ignores alpha. (`raylib/src/test_harness.rs`.)
- **MSVC link fix (found here, root-caused in WS5a):** the safe crate exposed the whole `gen_image_*` family unconditionally while `build.rs` only compiles those C symbols under `SUPPORT_IMAGE_GENERATION`, so `--no-default-features` MSVC builds failed `LNK2019` on clean runners (WS4b only passed via a CI build-cache artifact). WS5a Task 8 cfg-gated the family to fix it.

## WS5a — raygui broad rework
- **String convention (imgui-rs model):** controls take `impl AsRef<str>` (nullable → `Option<impl AsRef<str>>`) converted through a reusable **thread-local scratch buffer** (`rgui/scratch.rs`: `scratch_txt`/`_opt`/`_two`/`_three`/`_slice`). Amortized zero per-frame alloc, no NUL panic. Editable-buffer controls keep `&mut String` via the shared `gui_edit_string` helper.
- **Module split:** the 784-line `safe.rs` → `state.rs` / `containers.rs` / `controls.rs` / `advanced.rs` / `icons.rs` / `scratch.rs`, each a sub-trait (`RaylibGuiState`/`Containers`/`Controls`/`Advanced`/`Icons`) blanket-impl'd for `D: RaylibDraw`; `RaylibGuiState` also impl'd for `RaylibHandle`. Umbrella `RaylibDrawGui` supertrait retained for source compat. Duplication (old `impl RaylibHandle` + trait) eliminated.
- **Parity:** 57/57 RAYGUIAPI fns wrapped (audited against `binding/raygui.h`). 9 new: `GuiTabBar`, `GuiValueBoxFloat`, `GuiColorPanel`, `GuiColorPickerHSV`, `GuiColorPanelHSV`, `GuiSetIconScale`, `GuiDrawIcon`, and `GuiGetIcons`/`GuiLoadIcons` as raw `unsafe` + `# Safety` (safe abstraction deferred). **#296 deferred** — `GuiLoadStyleFromMemory` is absent from the vendored header.
- **Soundness fixes (from code review, verified against raygui C source):**
  1. `gui_set_tooltip` — `GuiSetTooltip` stores the pointer and derefs it on later control draws; now retains the string in a thread-local `CString` (the old scratch/stack pointer was a UAF — pre-existing).
  2. `gui_value_box_float` — `GuiValueBoxFloat` takes no size and writes up to `RAYGUI_VALUEBOX_MAX_CHARS+1` (33) bytes; now requires/reserves ≥33 bytes (small buffers overran the heap).
  3. `gui_edit_string` — viewing uninitialized spare capacity as `&[u8]` was UB; now zero-inits the full capacity before scanning.
  4. `gui_get_state` — replaced `transmute(i32→enum)` with a checked `match`.

## WS5b — safe immediate-mode rlgl
- New **top-level** `raylib/src/rlgl/` module (sibling of `core/`/`rgui/`; wraps `rlgl.h`). Entry via `RaylibRlgl` (blanket-impl'd for `D: RaylibDraw`).
- **Matrix stack:** `rl_push_matrix()` → `RlMatrix` RAII guard (auto-`rlPopMatrix`, derefs to the handle); `rl_translatef`/`rl_rotatef`/`rl_scalef`/`rl_load_identity`/`rl_mult_matrixf`/`rl_matrix_mode`/`rl_ortho`/`rl_set_matrix_projection`/`rl_set_matrix_modelview`. (`Matrix` is a re-export of `ffi::Matrix`, so set-matrix passes by value; `rl_mult_matrixf` casts `&Matrix as *const f32` — repr(C), 16 column-major f32s.)
- **Immediate mode:** `rl_begin(DrawMode)` → `RlImmediate` RAII guard (auto-`rlEnd`) and closure form `rl_draw`. Vertex methods (`vertex2f/3f`, `color4ub/3f/4f`, `texcoord2f`, `normal3f`) live on the guard, so they can't be called outside a block. `DrawMode` (Lines/Triangles/Quads) + `MatrixMode` enums from `ffi::RL_*`.
- **Render-state toggles:** depth test, backface culling.
- **Bind-safe-handle:** `rl_set_texture`/`rl_enable_texture`/`rl_disable_texture`/`rl_active_texture_slot`/`rl_enable_shader`/`rl_set_shader` take `&Texture2D`/`&Shader` and read `.id`/`.locs` (with a textured doctest). GL-object lifecycle left to existing RAII + raw `ffi`. Closes #234 + #179.
- **rlsw finding (documented in the test):** immediate-mode vertices **must** emit `texcoord2f`; without it the default (0,0) coord samples a transparent texel in the shapes texture under rlsw → 0 pixels. Bind the shapes texture + emit texcoords per vertex.

## Render tests (Tier-2, via `software_renderer`)
`render_shapes`, `render_text` (WS4b, now natural coords), `render_gui` (button/label), `render_rlgl` (immediate-mode quad + matrix translate + red/blue fidelity). CI feature set: `software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION[,raygui]`. All run on ubuntu/macOS/windows in `.github/workflows/baseline.yml` (`software-render` job).

## Tracked-deferred (carried to WS6)
- WS3 idiom PRs (#272/#268/#266) + soundness PRs (#277/#257/#256/#118).
- The `custom_audio_stream_callback` deprecation warning (blocks WS6 `-Dwarnings`).
- Safe abstraction for `GuiGetIcons`/`GuiLoadIcons` (currently raw `unsafe`).
- PR #296 (`GuiLoadStyleFromMemory`) — revisit when the vendored raygui advances.
- The raylib new-fn tail in `parity-checklist.md` (~non-gui TODOs).
- Model-animation RAII sanitizer test.
