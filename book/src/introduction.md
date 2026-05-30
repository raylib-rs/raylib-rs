# Introduction

raylib-rs is a safe, idiomatic Rust binding to raylib 6.0 — the simple and easy-to-use game programming library. It lets you write 2D/3D games, simulations, and visualizations in Rust with the same low-ceremony feel as the original C library, without requiring `unsafe` in user code.

## Who this is for

This book is for Rust developers who want to build interactive graphical applications — games, simulations, visualizations, or tools — using raylib's clean API. You should be comfortable with basic Rust (ownership, traits, closures), but you do not need prior game development experience. No C/C++ knowledge is required.

## What's in scope

raylib-rs 6.0 covers the full raylib 6.0 surface:

- **Window management and drawing** — 2D and 3D rendering, shapes, textures, render targets.
- **Input** — keyboard, mouse, gamepad, touch.
- **Text and fonts** — system fonts, custom fonts, SDF rendering.
- **3D models** — meshes, materials, skeletal animation via `ModelAnimations`.
- **Audio** — sounds, music streams, `AudioStream` for custom mixing.
- **raymath** — vectors, matrices, quaternions, easing functions. Math methods come via a C shim so there is no extra dependency by default.
- **raygui** — immediate-mode GUI panels, buttons, sliders, text boxes.
- **rlgl** — low-level OpenGL abstraction for custom rendering.
- **Software renderer** — a headless `PLATFORM=Memory` backend for pixel-probe testing without a GPU or window.

## What's out of scope here

- **The C raylib reference** — see [raylib.com](https://www.raylib.com/) for the upstream API docs.
- **The rustdoc API reference** — generated from doc comments; published to docs.rs with each crate release.
- **The showcase examples** — full ports of the raylib C example set, shipping as a GitHub Pages site in WS9.

## How to read this book

**[Getting Started](./getting-started/quickstart.md)** — set up the dependency and open your first window in under 30 lines. Then follow the install guide for your platform (Windows, macOS, Linux, or Web/Wasm).

**Core Concepts** — the conventions that show up everywhere: the `RaylibHandle`/`RaylibThread` ownership model; RAII resources and how they are cleaned up; strings and allocations; safety rules; feature flags and platform support.

**Modules** — per-area narrative chapters: window and drawing, input, shapes, textures, text, 3D, audio, raymath, collision, raygui, rlgl, the software renderer, callbacks, and error handling.

**Ecosystem** — opt-in integrations: `glam` and `mint` for interop with other math libraries, `serde` for serialization, and pointers to what comes next.
