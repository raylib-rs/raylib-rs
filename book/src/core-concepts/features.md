# Features and platforms

raylib-rs uses Cargo features to gate platform back-ends, optional math integrations, and the
software renderer. The default feature set mirrors raylib 6.0's `config.h` `SUPPORT_*` configuration:
everything a desktop OpenGL 3.3 application needs, and nothing it doesn't.

Optional integrations — `glam`, `mint`, `serde` — are strictly opt-in. A default build carries zero
math-crate dependencies. The `full` feature alias is the canonical "everything except mutually-
exclusive back-ends" target used for CI linting and testing.

> **Important:** `--all-features` is **invalid** for `raylib-sys`. Always use `--features full`
> when you want maximum capability in a single build invocation.

## API surface

Feature flags (headline set):

- `opengl_33` — OpenGL 3.3 core; this is the default back-end on desktop.
- `opengl_21` — OpenGL 2.1 compatibility; older hardware and some embedded targets.
- `opengl_es_20` — OpenGL ES 2.0; mobile / DRM back-end. Use with `drm`.
- `drm` — DRM/KMS tty rendering; pair with `opengl_es_20`.
- `wayland` — Wayland display server support (Linux).
- `software_renderer` — CPU software renderer (`PLATFORM=Memory`, rlsw). Mutually exclusive with
  `opengl_*`. Enables the `raylib::test_harness` headless render-test module.
- `glam` — `From`/`Into` conversions between raylib's `Vector*`/`Matrix`/`Quaternion` and
  `glam::Vec*`/`glam::Mat4`/`glam::Quat`.
- `mint` — `From`/`Into` conversions with `mint::Vector*`/`mint::ColumnMatrix*`.
- `serde` — `Serialize`/`Deserialize` derives on math and color types.
- `nobuild` — skip building the vendored raylib C source (you supply the link target).
- `nobindgen` — skip re-generating `raylib-sys` bindings (read from `RAYLIB_BINDGEN_LOCATION`).
- `full` — curated maximum-capability alias (excludes mutually-exclusive back-ends and escape
  hatches); use this for `cargo test --features full` and CI.

## Example

Three common `Cargo.toml` stanzas:

```text
# 1. Default desktop build (OpenGL 3.3, no optional adapters)
[dependencies]
raylib = "5.7"

# 2. Headless / CI build using the software renderer (no GPU required)
[dependencies]
raylib = { version = "5.7", features = ["software_renderer"] }

# 3. Game with glam math and serde serialization
[dependencies]
raylib = { version = "5.7", features = ["glam", "serde"] }
```

## Gotchas

- **`opengl_*` features are mutually exclusive.** The build script panics if more than one is
  selected. Do not enable two OpenGL features in the same build.
- **`software_renderer` is mutually exclusive with `opengl_*`.** The two back-ends cannot be
  compiled into the same binary. It is also currently incompatible with
  `wasm32-unknown-emscripten` — tracked-deferred; see `notes/ws6b-complete.md`.
- **`--all-features` is invalid for `raylib-sys`** because it would try to enable multiple
  mutually-exclusive back-ends simultaneously. Use `--features full` instead.
- **`SUPPORT_*` flags** — individual raylib capability flags (e.g., `SUPPORT_IMAGE_GENERATION`,
  `SUPPORT_FILEFORMAT_PNG`) map directly to Cargo features of the same name. They are included in
  `full` and usually do not need to be set explicitly.

## See also

- [Install on Windows](../getting-started/install-windows.md),
  [macOS](../getting-started/install-macos.md),
  [Linux](../getting-started/install-linux.md),
  [Web](../getting-started/install-web.md) — platform-specific dependency setup.
- [`modules/software-renderer.md`](../modules/software-renderer.md) — the headless render-test
  harness.
- [`ecosystem/glam-mint-serde.md`](../ecosystem/glam-mint-serde.md) — the math-ecosystem
  integration features.
