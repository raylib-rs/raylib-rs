# Showcase raygui source viewer — design

**Date:** 2026-06-06
**Status:** approved
**Scope:** `showcase/` (viewer, Cargo.toml), `.github/workflows/showcase.yml`

## Problem

The F1 source-viewer overlay (`showcase/src/viewer.rs`) is a hand-rolled
full-screen black panel: `draw_rectangle`/`draw_text` tabs, keyboard-only
scrolling, and a "Source on GitHub: \<url\>" footer line. Two issues:

1. The GitHub deep-links don't belong in the canvas overlay. PR #317 already
   put real `<a href>` C/Rust links on every synthesized example page — the
   in-overlay URL footer is redundant and unclickable.
2. The hand-rolled chrome duplicates (badly) what raygui already provides:
   window chrome, tab toggles, draggable scrollbars, buttons, and clipboard
   integration.

A factual constraint discovered during design: raygui's `GuiTextBox` is
**single-line only** (`multiline = false; // TODO` in the vendored
`raygui.h:2638`) and supports paste but not copy. raygui has no
text-selection control. Copy therefore means a whole-file **Copy button**
via `SetClipboardText`, not selection-based copy.

## Decisions

- **D1 — raygui always-on for showcase.** The viewer is compiled into all
  229 examples, so the showcase's `raylib` dependency gains the `raygui`
  feature unconditionally. The showcase `raygui` feature and all 32
  `required-features = ["raygui"]` entries are removed. No cfg-gated dual
  overlay implementation.
- **D2 — Copy is a whole-file button.** A `GuiButton("Copy")` copies the
  active tab's full source. No Ctrl+C binding, no selection.
- **D3 — footer URLs deleted everywhere** (desktop included). The HTML page
  owns the GitHub links. `SourcePair.c_url`/`rust_url` stay — the gallery
  index and `xtask_build_pages` still consume them.

## Viewer UI (`showcase/src/viewer.rs`)

Replace `draw_overlay` with native-raygui chrome (default light style,
matching the raygui examples):

- **`GuiWindowBox`** inset ~24 px from the canvas edges, titled with the
  example name. Its built-in **×** closes the viewer; F1 still toggles.
- **Header strip** inside the window: `GuiToggleGroup` `[C | Rust]` for tab
  switching (Tab key remains a shortcut), and a right-aligned
  `GuiButton("Copy")`.
- **Body:** `GuiScrollPanel`. Content rect =
  `(max_line_width, line_count × line_height)`; visible lines drawn inside a
  scissor guard over the returned `view` rect. Mouse wheel + draggable
  scrollbars come free from raygui (manual wheel handling is removed);
  PgUp/PgDn/Home/End keep working by nudging the stored scroll vector in
  `update`.
- The "F1: view source" hint badge stays hand-rolled (it's a passive badge,
  not a control).

### Mechanics

- `draw` changes `&self` → `&mut self`: raygui is immediate-mode, clicks are
  detected during draw. Close (× or window-box result) and tab clicks mutate
  state directly. The Copy click sets a `pending_copy` flag serviced on the
  next `update` via `RaylibHandle::set_clipboard_text` (clipboard needs the
  handle, which `draw` doesn't have). All 229 call sites already hold a
  `mut` viewer, so **no example files change**.
- `max_line_width` is measured (`measure_text`) once per tab in `update`
  and cached; scroll state lives in the struct as a `Vector2`.
- The safe rgui blanket impl (`impl<D: RaylibDraw> RaylibDrawGui for D`)
  means the existing `draw<D: RaylibDraw>` bound already provides every gui
  method — no signature rework beyond mutability.
- Delete `FOOTER_FONT_SIZE`, `FOOTER_TOP_GAP`, the footer draw call, and the
  footer reservation in `max_scroll` (scroll math re-derived from the
  scroll-panel view rect instead).

## Feature plumbing (`showcase/Cargo.toml`)

- `raylib = { version = ..., path = "../raylib", features = ["raygui"] }`
- Remove `raygui = ["raylib/raygui"]` from `[features]`.
- Remove all 32 `required-features = ["raygui"]` entries; the
  `SUPPORT_CUSTOM_FRAME_CONTROL` gate stays.
- `xtask_wasm_build`'s bucket logic is data-driven from `Cargo.toml`
  required-features — the raygui bucket disappears automatically, no xtask
  change.

## CI (`.github/workflows/showcase.yml`)

- Merge the two build steps (with/without raygui) into one
  `cargo build -p raylib-showcase --examples`.
- Drop `raygui` from the clippy feature list (keep
  `SUPPORT_CUSTOM_FRAME_CONTROL`).
- **Job names stay unchanged** — renaming/deleting a job orphans its
  required-check context in the `unstable` branch ruleset.
- `pages.yml` untouched: `gen_thumbnails` already runs with
  raygui + software_renderer (WS5 proved the combo).

## Known caveats

- **Wasm clipboard:** whether `SetClipboardText` reaches the browser
  clipboard under emscripten is verified during implementation. If it's a
  no-op on web, the button is harmless and web users have the page's GitHub
  links.
- **raygui-on-raygui examples:** with the overlay open over an example that
  draws its own gui controls, clicks can also reach controls underneath.
  Pre-existing behavior with the current overlay; out of scope.

## Testing & verification

- Existing viewer unit tests (`example_name_from_pathname`) stay.
- `cargo build -p raylib-showcase --examples` (single bucket now).
- `cargo clippy -p raylib-showcase --examples --features SUPPORT_CUSTOM_FRAME_CONTROL -- -D warnings`.
- Manual desktop pass on one example: F1 open, tab toggle (mouse + Tab key),
  scrollbar drag, wheel, PgUp/PgDn/Home/End, Copy → paste into an editor,
  close via × and via F1.
- Web covered by the existing Playwright pages smoke test.
