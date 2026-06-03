# WS7a — mdBook bootstrap + `book.yml` + getting-started — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land a buildable mdBook skeleton at `book/`, a green `.github/workflows/book.yml` CI workflow, and the seven entry-level chapters (introduction + quickstart + 4 install guides + deferred-targets note) that don't depend on the safe-API surface.

**Architecture:** mdBook source under `book/src/`, stock `navy` theme, `mdbook test` wired to `target/debug/deps` so ``` ```rust ``` code blocks compile against the actual `raylib` crate. The `book.yml` workflow builds the book and runs `mdbook test` on every push/PR — build-only, **no Pages deploy (WS9 finale)**.

**Tech Stack:** mdBook 0.4.x, `peaceiris/actions-mdbook@v2`, `dtolnay/rust-toolchain@stable`, the existing `raylib` workspace.

**Spec:** `docs/superpowers/specs/2026-05-27-ws7-docs-and-book-design.md` §3 (WS7a), §4 (book layout + book.toml + book.yml).

**Prerequisite:** WS6 complete (`check`/`test`/`web`/`sanitizers` green; `deny(missing_docs)` enforced). **Working model:** branch `6.0-rc`; push to `fork`; watch with `gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status` last.

---

## File structure

| File | Responsibility | Tasks |
|------|----------------|-------|
| `book/book.toml` (new) | mdBook config: title, src dir, theme, edition | 1 |
| `book/src/SUMMARY.md` (new) | Full table of contents (placeholders for WS7b/WS7c chapters) | 1 |
| `book/src/introduction.md` (new) | What raylib-rs 6.0 is, audience, scope | 2 |
| `book/src/getting-started/quickstart.md` (new) | Minimum hello-window in <30 lines; doubles as the doctest-wiring integration test | 3 |
| `.github/workflows/book.yml` (new) | mdBook build + `mdbook test` on push/PR, Ubuntu, no deploy | 4 |
| `book/src/getting-started/install-windows.md` (new) | Step-by-step Windows install | 5 |
| `book/src/getting-started/install-macos.md` (new) | Step-by-step macOS install | 6 |
| `book/src/getting-started/install-linux.md` (new) | Step-by-step Linux install | 7 |
| `book/src/getting-started/install-web.md` (new) | wasm32-unknown-emscripten path | 8 |
| `book/src/getting-started/deferred-targets.md` (new) | Known-unsupported: aarch64, cross-compile, NixOS X11 | 9 |
| `CONTRIBUTE.md` (modify) | Add a one-liner for `mdbook serve book` | 10 |

---

## Task 1: `book.toml` + `SUMMARY.md` skeleton

**Files:**
- Create: `book/book.toml`
- Create: `book/src/SUMMARY.md`

The book builds with empty placeholder chapters so the CI can be wired and verified before any prose is written. Placeholder chapters use mdBook's "draft chapter" feature (a SUMMARY entry with no path) so they show in the TOC as unwritten without breaking the build.

- [ ] **Step 1: Create `book/book.toml`**

```toml
[book]
title = "raylib-rs 6.0"
authors = ["raylib-rs contributors"]
language = "en"
src = "src"

[output.html]
git-repository-url = "https://github.com/raylib-rs/raylib-rs"
edit-url-template = "https://github.com/raylib-rs/raylib-rs/edit/unstable/book/{path}"
default-theme = "navy"
preferred-dark-theme = "navy"

[rust]
edition = "2024"
```

- [ ] **Step 2: Create `book/src/SUMMARY.md` with full TOC; later-WS chapters are draft entries**

Draft chapters (no file path, indented) build without erroring. They render in the TOC italicized.

```markdown
# Summary

[Introduction](./introduction.md)

# Getting Started

- [Quickstart](./getting-started/quickstart.md)
- [Install on Windows](./getting-started/install-windows.md)
- [Install on macOS](./getting-started/install-macos.md)
- [Install on Linux](./getting-started/install-linux.md)
- [Install for the Web](./getting-started/install-web.md)
- [Deferred targets](./getting-started/deferred-targets.md)

# Core Concepts

- [Handle and thread]()
- [RAII and resources]()
- [Strings and allocations]()
- [Safety]()
- [Features and platforms]()

# Modules

- [Window and drawing]()
- [Input]()
- [Shapes]()
- [Textures and images]()
- [Text and fonts]()
- [3D models]()
- [Audio]()
- [raymath]()
- [Collision]()
- [raygui]()
- [rlgl]()
- [Software renderer]()
- [Callbacks and logging]()
- [Error handling]()

# Ecosystem

- [glam, mint, serde]()
- [What next?]()
```

- [ ] **Step 3: Local sanity — `mdbook build`**

If `mdbook` is not installed locally, skip the local verify and rely on CI (Task 4). To install: `cargo install mdbook --version 0.4.43` or use the official mdBook release binary.

Run: `mdbook build book`
Expected: builds clean to `book/book/index.html`; warnings about empty chapters are OK (draft chapters are allowed).

- [ ] **Step 4: Commit the skeleton**

```bash
git add book/book.toml book/src/SUMMARY.md
git commit -m "docs(ws7a): bootstrap mdBook skeleton at book/

Empty draft chapters for Core Concepts/Modules/Ecosystem (filled in WS7b).
Getting Started chapters land in subsequent commits.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 2: `introduction.md` — the front door

**Files:**
- Create: `book/src/introduction.md`

**Contract for the prose:**
- 200–400 words.
- Opens with a one-sentence definition: "raylib-rs is a safe, idiomatic Rust binding to raylib 6.0."
- Audience: Rust developers writing 2D/3D games, simulations, or visualizations who want raylib's simplicity without dropping to `unsafe` everywhere.
- Lists what's in scope: window/input/drawing, 2D/3D, raymath, audio, raygui, rlgl, software-renderer headless harness.
- Lists what's out of scope (or covered elsewhere): the C raylib reference (links to upstream raylib site); the docs.rs rustdoc API reference; the showcase examples (links to the future Pages site).
- Closes with a "How to read this book" paragraph: Getting Started for setup; Core Concepts for the conventions that show up everywhere (RAII, the handle/thread, safety); Modules for per-area narrative; Ecosystem for opt-in integrations.
- One or two cross-reference links: to `getting-started/quickstart.md` and to upstream raylib at `https://www.raylib.com/`.

- [ ] **Step 1: Write `book/src/introduction.md` per the contract above**

- [ ] **Step 2: Build check**

Run: `mdbook build book`
Expected: clean build; the introduction renders at the root.

- [ ] **Step 3: Commit**

```bash
git add book/src/introduction.md
git commit -m "docs(ws7a): introduction chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 3: `quickstart.md` — the doctest-wiring integration test

**Files:**
- Create: `book/src/getting-started/quickstart.md`

The quickstart is **both prose AND the integration test for `mdbook test` wiring** — it must contain at least one ``` ```rust,no_run ``` block that uses the real `raylib` crate. Window-opening so it's `no_run`, but it must compile.

**Contract for the prose:**
- 150–300 words.
- Three sections: "Add the dependency", "Open a window", "What's next".
- "Add the dependency" — Cargo.toml snippet (text block, not Rust):
  ```toml
  [dependencies]
  raylib = "5.7"
  ```
  (Note: version stays at 5.7 in the book until WS8 publishes 6.0; the book lives on `6.0-rc` but `Cargo.toml` doesn't bump until WS8. Footnote this if it feels needed.)
- "Open a window" — a ``` ```rust,no_run ``` block, the canonical hello-window:

  ```rust,no_run
  use raylib::prelude::*;

  fn main() {
      let (mut rl, thread) = raylib::init()
          .size(640, 480)
          .title("Hello, raylib-rs")
          .build();

      while !rl.window_should_close() {
          let mut d = rl.begin_drawing(&thread);
          d.clear_background(Color::WHITE);
          d.draw_text("Hello, raylib-rs!", 20, 20, 20, Color::BLACK);
      }
  }
  ```

- "What's next" — link to the install guides and to the Modules chapters.

- [ ] **Step 1: Write `book/src/getting-started/quickstart.md` per the contract**

- [ ] **Step 2: Verify the Rust block compiles via `mdbook test`**

Run (from repo root):
```bash
cargo build -p raylib --features full
mdbook test book -L target/debug/deps
```

Expected: zero failures. If the Rust code block fails (missing imports, renamed API, etc.), fix the snippet — the quickstart is the canonical "hello, raylib-rs" example and must always compile.

- [ ] **Step 3: Commit**

```bash
git add book/src/getting-started/quickstart.md
git commit -m "docs(ws7a): quickstart chapter (verifies mdbook test wiring)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 4: `book.yml` — mdBook build + `mdbook test` in CI

**Files:**
- Create: `.github/workflows/book.yml`

Build-only on Ubuntu. The `mdbook test` step requires the `raylib` crate compiled first so the doctest linker can find it under `target/debug/deps`.

- [ ] **Step 1: Write `.github/workflows/book.yml`**

```yaml
name: book

on:
  push:
    branches: [6.0-rc, unstable, master]
  pull_request:
  workflow_dispatch:

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive

      - name: Install Linux build deps
        run: |
          sudo apt-get update
          sudo apt-get install --no-install-recommends -y \
            cmake \
            libglfw3-dev \
            libwayland-dev \
            libxkbcommon-dev \
            libegl1-mesa-dev \
            libgles2-mesa-dev \
            libxinerama-dev \
            libxcursor-dev \
            libxrandr-dev \
            libxi-dev \
            libgl1-mesa-dev

      - uses: dtolnay/rust-toolchain@stable

      - name: Set up mdBook
        uses: peaceiris/actions-mdbook@v2
        with:
          mdbook-version: 'latest'   # pin to a specific tag (e.g. '0.4.43') after first green run

      - name: Build raylib (debug; needed by mdbook test for doctest linking)
        run: cargo build -p raylib --features full

      - name: Build the book
        run: mdbook build book

      - name: Run book doctests
        run: mdbook test book -L target/debug/deps
```

- [ ] **Step 2: Commit + push + watch**

```bash
git add .github/workflows/book.yml
git commit -m "ci(ws7a): add book.yml — mdBook build + mdbook test (no deploy)

Build-only on Ubuntu; public Pages deploy lands in WS9.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
git push fork 6.0-rc
# capture the run id from the push output or 'gh run list -R Dacode45/ms-raylib-rs --limit 1'
gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status
```

Expected: green. If `mdbook test` fails with linker errors looking for `libraylib`, check that the `cargo build -p raylib --features full` step succeeded and produced `target/debug/deps/libraylib-*.rlib`. If linker errors mention native libraries (`libglfw`, `libGL`), revisit the apt-get list.

- [ ] **Step 3: Pin `mdbook-version` to the resolved version after first green run**

```bash
# Look at the workflow log for the 'Set up mdBook' step — it prints the version. e.g., 0.4.43.
```

Edit `book.yml`:
```yaml
          mdbook-version: '0.4.43'   # pinned after first green run (was 'latest')
```

Commit:
```bash
git add .github/workflows/book.yml
git commit -m "ci(ws7a): pin mdbook-version after first green run

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 5: `install-windows.md`

**Files:**
- Create: `book/src/getting-started/install-windows.md`

**Contract:**
- 200–400 words.
- Prerequisites: Rust 1.85+ via rustup, Visual Studio Build Tools 2019+ (the MSVC toolchain), CMake.
- One canonical install path: install rustup, run `rustup install stable`, install VS Build Tools with "Desktop development with C++" workload, install CMake from cmake.org.
- Quick verify: `cargo new my-raylib-game && cd my-raylib-game && cargo add raylib && cargo build` should succeed.
- Troubleshooting bullet for "cannot find libclang" → link to bindgen docs and to backlog issue #254 (or whichever resolves).
- Note on the windowing subsystem: GLFW + OpenGL 3.3 by default; opt-in `wayland`/`drm` features are Linux-only.

- [ ] **Step 1: Write `book/src/getting-started/install-windows.md` per the contract**

- [ ] **Step 2: Build check**

Run: `mdbook build book`
Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add book/src/getting-started/install-windows.md
git commit -m "docs(ws7a): Windows install guide

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 6: `install-macos.md`

**Files:**
- Create: `book/src/getting-started/install-macos.md`

**Contract:**
- 200–400 words.
- Prerequisites: Rust 1.85+ via rustup, Xcode Command Line Tools, CMake (via Homebrew or cmake.org).
- Apple Silicon vs Intel: both work; Apple Silicon uses the system Metal-via-OpenGL compatibility path.
- Quick verify: `cargo new my-raylib-game && cd my-raylib-game && cargo add raylib && cargo build` should succeed.
- Troubleshooting bullet: if linker fails with framework errors, ensure Xcode CLI Tools are current (`xcode-select --install`).

- [ ] **Step 1: Write `book/src/getting-started/install-macos.md` per the contract**

- [ ] **Step 2: Build check**

Run: `mdbook build book`
Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add book/src/getting-started/install-macos.md
git commit -m "docs(ws7a): macOS install guide

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 7: `install-linux.md`

**Files:**
- Create: `book/src/getting-started/install-linux.md`

**Contract:**
- 250–450 words (Linux has the most variability).
- Prerequisites: Rust 1.85+ via rustup; CMake; the X11/Wayland dev headers (full apt-get list mirrors what `book.yml` installs — copy it verbatim).
- Ubuntu/Debian: `apt-get install` line.
- Fedora/RHEL: `dnf install` line (mention `cmake gcc-c++ libX11-devel libXcursor-devel libXinerama-devel libXrandr-devel libXi-devel mesa-libGL-devel`).
- Arch: `pacman -S` line.
- Quick verify: `cargo new my-raylib-game && cd my-raylib-game && cargo add raylib && cargo build` should succeed.
- Note: Wayland support is opt-in via `--features wayland`; DRM/tty rendering via `--features drm,opengl_es_20`.
- Troubleshooting: missing libclang → install `clang` / `libclang-dev`.

- [ ] **Step 1: Write `book/src/getting-started/install-linux.md` per the contract**

- [ ] **Step 2: Build check**

Run: `mdbook build book`
Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add book/src/getting-started/install-linux.md
git commit -m "docs(ws7a): Linux install guide (Ubuntu/Debian/Fedora/Arch)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 8: `install-web.md`

**Files:**
- Create: `book/src/getting-started/install-web.md`

**Contract:**
- 250–450 words.
- Target: `wasm32-unknown-emscripten` (build-verified continuously in `web.yml`).
- Prerequisites: Rust 1.85+, the wasm32 target (`rustup target add wasm32-unknown-emscripten`), emsdk 3.1.64+ (link to upstream emsdk install docs).
- Required `EMCC_CFLAGS` env var — copy the exact line from `web.yml`:
  ```
  -O3 -sUSE_GLFW=3 -sASSERTIONS=1 -sWASM=1 -sASYNCIFY -sGL_ENABLE_GET_PROC_ADDRESS=1
  ```
- Build command: `cargo build -p raylib --target wasm32-unknown-emscripten`.
- Note: CI build-verifies the target; running in a browser locally needs an HTML shell and a web server — point at the future WS9 showcase site for working examples.
- Known limitation: the `software_renderer` feature is **not** currently supported on `wasm32-unknown-emscripten` (link to `notes/ws6b-complete.md` tracked-deferred item #5).

- [ ] **Step 1: Write `book/src/getting-started/install-web.md` per the contract**

- [ ] **Step 2: Build check**

Run: `mdbook build book`
Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add book/src/getting-started/install-web.md
git commit -m "docs(ws7a): web (wasm32-unknown-emscripten) install guide

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 9: `deferred-targets.md` — known-unsupported

**Files:**
- Create: `book/src/getting-started/deferred-targets.md`

**Contract:**
- 100–200 words. Short and explicit.
- Header: "Known-unsupported / future work" (or similar).
- One bullet per deferred target, each with: target name, current status, link to the backlog item.
  - **aarch64 Linux** — backlog issue #227. Tracked; build currently broken.
  - **Cross-compile Linux → Windows (debug mode)** — backlog issue #181.
  - **NixOS X11** — backlog PR #289 + issue #288. Companion in WS6 platform matrix.
  - **`software_renderer` on `wasm32`** — tracked-deferred in `notes/ws6b-complete.md` (item 5).
- Closing line: "These are tracked in the project backlog; community PRs welcome."

- [ ] **Step 1: Write `book/src/getting-started/deferred-targets.md` per the contract**

- [ ] **Step 2: Build check**

Run: `mdbook build book`
Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add book/src/getting-started/deferred-targets.md
git commit -m "docs(ws7a): deferred-targets note (aarch64, cross-compile, nix, rlsw-on-web)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 10: `CONTRIBUTE.md` — `mdbook serve` one-liner

**Files:**
- Modify: `CONTRIBUTE.md`

**Contract:** add one short "Working on the book" section near the bottom (or after the existing build-instructions section, whichever fits the file's flow).

- [ ] **Step 1: Read `CONTRIBUTE.md` and locate the right insertion point**

Run: read `CONTRIBUTE.md` to find an appropriate section (likely after build/test instructions; before any contribution-flow section).

- [ ] **Step 2: Add the section**

Append (adjust placement to match the file's existing structure):

```markdown
## Working on the book

The user guide lives in `book/` and is built with [mdBook](https://rust-lang.github.io/mdBook/). Install once with `cargo install mdbook` (or use the official release binary), then from the repo root:

- `mdbook serve book` — live preview at <http://localhost:3000>.
- `mdbook build book` — write the static HTML to `book/book/`.
- `mdbook test book -L target/debug/deps` — run the book's Rust code blocks as doctests (requires `cargo build -p raylib --features full` first).

CI builds and tests the book on every push (`book.yml`). The public Pages deploy ships with the WS9 showcase.
```

- [ ] **Step 3: Commit**

```bash
git add CONTRIBUTE.md
git commit -m "docs(ws7a): CONTRIBUTE.md — mdbook serve/build/test one-liners

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 11: Final push + watch — WS7a sign-off

- [ ] **Step 1: Push the accumulated commits**

```bash
git push fork 6.0-rc
```

- [ ] **Step 2: Watch `book.yml` go green on the full final state**

```bash
gh run list -R Dacode45/ms-raylib-rs --limit 5
# pick the latest book run id
gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status
```

Expected: green. Both `mdbook build` and `mdbook test` pass.

- [ ] **Step 3: WS7a sign-off**

WS7a is done when:
- `book/` exists with `book.toml`, `src/SUMMARY.md`, and the 7 getting-started markdown files.
- `book.yml` is green on the fork.
- `mdbook serve book` locally renders the introduction, quickstart, and 5 install/deferred-targets chapters.
- `CONTRIBUTE.md` has the `mdbook serve` one-liner.

Next: WS7b (`2026-05-27-ws7b-book-chapters.md`) fills the 21 placeholder chapters.
