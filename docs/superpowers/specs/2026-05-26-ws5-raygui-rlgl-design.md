# WS5 — raygui (6.0 parity, broad rework) + safe rlgl module — design

**Status:** approved (brainstorm 2026-05-26). Branch `6.0-rc`. Supersedes the one-line WS5 entry in
the roadmap (`2026-05-25-raylib-rs-6.0-roadmap-design.md`, §6 WS5).

**Goal (done criteria):** raygui at raylib-6.0 parity via a broad rework of `raylib/src/rgui/`, plus
a new safe **immediate-mode** rlgl module — both render-verified with the WS4 `test_harness`
(Tier-2 pixel probes), 3-OS CI green. One merge to `raylib-rs/unstable` only at WS8.

This spec covers three pieces, executed in order: **WS5-prep** (shared harness change), **WS5a**
(raygui), **WS5b** (rlgl). `writing-plans` will produce the per-piece implementation plans.

---

## Background

- raygui safe bindings live in `raylib/src/rgui/` behind the `raygui` feature: `mod.rs` +
  `safe.rs` (784 lines). Parity today (per `parity-checklist.md`): **48 / 57** RAYGUIAPI fns
  wrapped, 9 TODO.
- rlgl: `rlgl.h` is already in the bindgen input (`raylib-sys/binding/binding.h`), so **161 `rl*`
  FFI fns exist in `raylib-sys`**. There is **no safe wrapper** in `raylib/src/` today.
- The WS4b headless harness (`raylib/src/test_harness.rs`, feature `software_renderer`) draws into
  an in-memory rlsw framebuffer and reads back via `load_image_from_screen`. That readback is
  **BGRA + Y-inverted** and deterministic on every OS (see
  `notes/ws4b-complete.md` and `[[software-renderer-headless-testing]]`).

### Backlog cherry-picks (with attribution), per `inventory.md`
- **#296** (jgabaut) `gui_load_style_from_memory` — WS5a, *if* the vendored raygui header exposes the
  underlying fn; otherwise defer with a note.
- **#234** safe bindings for OpenGL draw calls — WS5b (rlgl).
- **#179** `rlPushMatrix` family — WS5b (rlgl).

---

## WS5-prep — harness readback normalization

**Problem.** `render_frame` currently returns the raw rlsw readback: BGRA channel order +
Y-inverted. Every render test compensates per-probe (the WS4b status quo). Writing many WS5/showcase
tests on top of that means each one re-derives the quirk.

**Change.** Normalize the readback so callers use natural top-left RGBA coordinates and colors:

- `render_frame(rl, thread, draw) -> Image` returns a **true top-left RGBA** image: swap R↔B and
  flip vertically after `load_image_from_screen`.
- Add `render_frame_raw(rl, thread, draw) -> Image` preserving the current BGRA + Y-inverted
  behavior as a documented escape hatch.
- `assert_pixel` / `pixel_at` continue to compare **R/G/B only** (alpha deliberately ignored — the
  Memory framebuffer reports alpha that need not match an opaque `Color`'s `a`; see
  `notes/ws4b-complete.md` finding 4).
- Update the two existing WS4b tests (`render_shapes.rs`, `render_text.rs`) to natural coords/colors
  and confirm they still pass.

**Implementation note.** The flip + channel swap is done in the harness on the returned `Image`
(e.g. via existing image ops / a direct pixel pass); no `unsafe` beyond what the harness already
composes. Keep `render_frame` `#[cfg(feature = "software_renderer")]` as today.

**Done.** `render_frame` yields natural RGBA top-left; `render_frame_raw` available; both WS4b tests
green under the existing CI feature set.

---

## WS5a — raygui broad rework

### A. String convention (the central change)

raygui's C API takes `const char*` (null-terminated). The 6.0-rc `safe.rs` currently does
`CString::new(text).unwrap()` per call — a fresh heap allocation every frame and a panic on interior
NUL. Research (`docs/research/2026-05-26-A-raygui-string-ergonomics/public.md`) found that imgui-rs,
the closest analog, **removed its `im_str!` macro** (the analog of `rstr!`) and switched to
`impl AsRef<str>` backed by a **single reusable scratch buffer**, achieving ergonomics *and*
amortized zero per-frame allocation.

Adopt that model:

- Per-frame text params take **`impl AsRef<str>`**; nullable-text controls (those that accept a C
  `NULL`, e.g. `GuiPanel`/`GuiLine` titles) take **`Option<impl AsRef<str>>`** (`None` → null ptr).
- Conversion goes through a **reusable thread-local scratch buffer** with helpers:
  - `scratch_txt(s) -> *const c_char`
  - `scratch_txt_opt(Option<s>) -> *const c_char` (`None` → `null`)
  - `scratch_txt_two(a, b) -> (*const c_char, *const c_char)` — two live pointers at different
    offsets in the one buffer, for multi-string controls (e.g. `GuiMessageBox`, `GuiTextInputBox`).
  - Buffer resets (clears) when it grows past a `max_len` cap (imgui-rs `UiBuffer` pattern).
- A **thread-local** buffer fits raylib's single-threaded, `!Send` draw model (`RaylibThread` is
  `!Send`); no per-call signature plumbing. (Implementation may instead place the buffer on the
  handle if the plan finds a concrete reason; thread-local is the default.)
- Interior NUL: copy bytes verbatim into the buffer + a trailing `\0`; an interior NUL truncates the
  C view (no panic). Document this; it matches imgui-rs behavior.

Net caller experience: `d.gui_button(rect, "OK")`, `d.gui_label(rect, &format!("Score: {score}"))`,
`d.gui_panel(rect, None::<&str>)`. `rstr!` is no longer used for gui (showcase `Some(rstr!("x"))` →
`"x"` / `Some("x")`, done in WS9).

### B. Module restructure

Split the 784-line `safe.rs` into grouped files under `raylib/src/rgui/`:

- `scratch.rs` — the thread-local scratch buffer + `scratch_txt*` helpers (internal).
- `state.rs` — global state / lock / alpha / font / style / styles-loading / tooltips.
- `containers.rs` — WindowBox, GroupBox, Line, Panel, ScrollPanel, TabBar.
- `controls.rs` — basic controls (Label, Button, Toggle*, CheckBox, ComboBox, DropdownBox, Spinner,
  ValueBox(+Float), TextBox, Slider*, ProgressBar, StatusBar, DummyRec, Grid).
- `advanced.rs` — ListView(Ex), MessageBox, TextInputBox, ColorPicker(HSV), ColorPanel(HSV),
  ColorBar*.
- `icons.rs` — IconText, SetIconScale, GetIcons, LoadIcons, DrawIcon.
- `mod.rs` — feature gate, re-exports, the prelude wiring.

**Trait layout.** Today control fns are default methods on one `RaylibDrawGui` trait,
blanket-`impl`'d for `D: RaylibDraw` (so they're callable on every draw-handle type). Keep that
pattern but split into **grouped sub-traits**, each blanket-impl'd for `D: RaylibDraw` and re-exported
together (so `use raylib::prelude::*` still brings them all in):
`RaylibGuiState`, `RaylibGuiContainers`, `RaylibGuiControls`, `RaylibGuiAdvanced`, `RaylibGuiIcons`.
(A single umbrella `RaylibDrawGui` may be retained as a supertrait/alias for source compatibility if
cheap.)

**Consolidate duplication.** Global-state fns are currently defined twice — on `impl RaylibHandle`
*and* on the `RaylibDrawGui` trait. Resolve to a single home: control-drawing fns on the
draw-context sub-traits; global config/state setters available where callers need them without
duplicate definitions. The plan settles the exact placement; the requirement is **no duplicated
wrapper bodies**.

### C. Soundness

- Replace `std::mem::transmute(GuiGetState())` (and any sibling enum transmutes) with a checked
  `i32 → enum` conversion (`TryFrom`/match), per the crate's `unsafe`/SAFETY conventions.

### D. Parity completion

- Wrap the **9 TODO fns**: `GuiTabBar`, `GuiValueBoxFloat`, `GuiColorPanel`, `GuiColorPickerHSV`,
  `GuiColorPanelHSV`, `GuiSetIconScale`, `GuiGetIcons`, `GuiLoadIcons`, `GuiDrawIcon`.
- **Audit** the 48 existing wrappers against the vendored raylib-6.0 `raygui.h` for signature/return
  changes; adjust wrappers as needed.
- Fold in **#296** (`gui_load_style_from_memory`) if the vendored header supports it.
- Update `parity-checklist.md` raygui section to reflect completion.

### Done (WS5a)
All 57 raygui fns wrapped or consciously deferred; the new `impl AsRef<str>` + scratch-buffer
convention applied module-wide; module split into grouped files/sub-traits; no duplicated bodies;
enum-transmute soundness fixed; #296 folded if applicable; Tier-2 render tests cover a sample of
controls; `deny(missing_docs)` clean.

---

## WS5b — safe rlgl module (immediate-mode focus)

A new safe **top-level** module `raylib/src/rlgl/` — a sibling of `core/` and `rgui/`, not a `core/`
submodule. rlgl wraps a separate header (`rlgl.h`) and is the GL-abstraction *layer* that raylib's
own domain modules sit on top of, so it mirrors how `rgui/` (wrapping `raygui.h`) is placed rather
than joining the domain submodules in `core/`. It covers the immediate-mode drawing surface the rlgl
showcase examples actually use
(`shapes_rlgl_triangle`, `shapes_rlgl_color_wheel`, `models_rlgl_solar_system`; `rlgl_standalone`
stays a raw-ffi example because it bypasses `RaylibHandle`). Scope is deliberately **not** the full
161-fn surface — GL-object lifecycle stays with the existing Texture/Shader/RenderTexture RAII.

### Surface

- **Matrix stack** (RAII): a guard that `rlPushMatrix` on entry / `rlPopMatrix` on drop, plus
  `translate(x,y,z)`, `rotatef(angle, x,y,z)`, `scalef(x,y,z)`, `mult_matrixf`, `matrix_mode`,
  `load_identity`, `ortho`, `frustum`, `viewport`, `set_matrix_modelview`, `set_matrix_projection`.
- **Immediate mode**: a vertex-stream builder scoped to `rlBegin(mode)…rlEnd()` (RAII or
  closure-scoped), exposing `vertex2f/2i/3f`, `color3f/4f/4ub`, `texcoord2f`, `normal3f`.
- **Render-state toggles** used by the examples: `enable/disable_backface_culling`,
  `enable/disable_depth_test`. (Add the few neighbors only if an example needs them.)
- **Bind-safe-handle interop** (forward-looking; no showcase example exercises it): ergonomic
  methods that accept the crate's safe wrappers and read `.id` (and `locs`) internally —
  `rl_set_texture(&Texture2D)`, `rl_enable_texture(&Texture2D)`, `rl_active_texture_slot(i32)`,
  `rl_enable_shader(&Shader)`, `rl_set_shader(&Shader)`. These are sound because the safe wrapper
  guarantees a live id. Include a **doctest** showing textured immediate-mode drawing.
- **GL-object lifecycle stays raw `ffi`** (create/destroy of textures/shaders/framebuffers/VBOs,
  render batch internals): document that users obtain resources from the existing safe API
  (`load_texture`, `load_shader`) and pass them in, and that the raw `ffi::rl*` calls remain
  available as a power-user escape hatch (note `Texture2D`/`Shader` `Deref` to their `ffi` struct,
  so `texture.id` already reads through today).

### Safety
Every `unsafe fn` gets a `/// # Safety` doc; every `unsafe { }` block a `// SAFETY:` comment. The
RAII guards must guarantee balanced push/pop and begin/end even on early return/panic within scope.

### rlsw / software_renderer behavior
Immediate-mode drawing works under `software_renderer`/rlsw (it is exactly how the WS4 harness
renders). Document any rlgl fns that are no-ops or behave differently under the Memory platform as
encountered; the safe module does not change shape under `software_renderer`.

### Cherry-picks
Fold #234 and #179 in with attribution as the relevant areas are implemented.

### Done (WS5b)
Safe immediate-mode rlgl module: matrix-stack RAII + vertex-stream builder + render-state toggles +
bind-safe-handle methods, all documented and `unsafe`-justified; lifecycle left to existing RAII;
Tier-2 render tests cover the matrix-stack + immediate-mode paths (triangle / color-wheel);
`deny(missing_docs)` clean.

---

## Testing (both WS5a + WS5b)

Tier-2 headless pixel-probe tests via `raylib::test_harness` using the **normalized** `render_frame`:

- raygui: render a sample of controls (e.g. button, label, checkbox, slider, panel) and probe
  representative pixels (control body / text presence), in the spirit of the WS4b shape/text tests.
- rlgl: render a matrix-stack-transformed immediate-mode triangle and a color-wheel sector; probe
  expected colored pixels.

CI feature set is unchanged from WS4b (all five `SUPPORT_MODULE_*` + `software_renderer`) **plus the
`raygui` feature** for the gui legs. When watching CI, make `gh run watch <id> --exit-status` the
LAST command in the call.

---

## Risks

1. **raygui header parity drift** — vendored `raygui.h` may differ from the parity-checklist
   assumptions. *Mitigation:* the WS5a audit step diff-checks every wrapper against the header.
2. **Scratch-buffer aliasing on multi-string calls** — two live `*const c_char` into one buffer must
   not overlap. *Mitigation:* `scratch_txt_two` places them at distinct offsets and returns both
   before any further push; covered by a unit test.
3. **rlgl under rlsw gaps** — some rlgl state fns may be inert on the Memory platform.
   *Mitigation:* scope WS5b render tests to immediate-mode + matrix stack (known-working in WS4) and
   document inert fns rather than testing them.
4. **API break from the string-convention change** — `&str` callers must migrate to `impl AsRef<str>`
   / `Option<impl AsRef<str>>`. *Mitigation:* the migration is in the ergonomic direction
   (`Some(rstr!("x"))` → `"x"`); showcase updates land in WS9; note in `CHANGELOG.md`.

## Tracked-deferred (carried from WS3/WS4b into WS6)
WS3 idiom PRs (#272/#268/#266), soundness PRs (#277/#257/#256/#118), the broader raylib new-fn tail
in `parity-checklist.md`, the `custom_audio_stream_callback` deprecation warning (blocks WS6
`-Dwarnings`), and the model-animation RAII sanitizer test remain folded into WS6 unless a WS5 task
naturally touches them.
