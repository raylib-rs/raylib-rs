# Strings and allocations

raylib is a C API, so strings cross an FFI boundary on almost every call. raylib-rs handles the
two different use cases separately:

- **Most API functions** accept `&str` and return owned `String`. The binding converts to a
  null-terminated `CString` internally. This is ergonomic and safe; the small allocation is
  acceptable for infrequent calls like loading a file or setting a window title.
- **Per-frame GUI draw functions** take `&CStr` to avoid allocations inside the render loop.
  The `rstr!` macro constructs a `&CStr` at compile time from a string literal — zero allocation,
  zero runtime cost.

The WS5 raygui binding goes further: raygui's string parameters use `impl AsRef<str>` signatures
backed by a thread-local scratch buffer. This means you can pass `&str` or `String` directly to
GUI calls without wrapping them in `rstr!`.

## API surface

- [`rstr!`](https://docs.rs/raylib/latest/raylib/macro.rstr.html) — constructs a `&'static CStr`
  from a string literal at compile time, or a `CString` from a format string.
- `&CStr` parameters — the low-level string type used by per-frame GUI draw functions.
- `&str` parameters — the ergonomic string type used by most load/config functions.
- `impl AsRef<str>` — the raygui parameter convention (WS5); accepts `&str`, `String`, and
  anything that derefs to `str`.

## Example

`rstr!` is the idiomatic way to create a `&CStr` literal without a heap allocation:

```rust
# extern crate raylib;
use std::ffi::CStr;

// rstr! is a macro exported from the raylib crate root.
let label: &CStr = raylib::rstr!("hello");
assert_eq!(label.to_bytes(), b"hello");
```

The compile-time form (`rstr!("literal")`) embeds a null byte and wraps the bytes as `&CStr`
without any runtime allocation. The runtime form (`rstr!("{} items", count)`) allocates a
`CString` and is suitable for dynamic strings outside the hot path.

## Gotchas

- **Per-frame GUI calls use `&CStr`** — use `rstr!` for compile-time labels. For dynamic strings
  inside a frame, prefer the `impl AsRef<str>` raygui API (WS5) which uses a thread-local buffer
  and avoids a heap allocation on every call.
- **Embedded nul bytes** — backlog #220 notes that `OsStr::to_string_lossy` does not replace
  embedded nul bytes. raylib-rs validates its string inputs, but be aware that C raylib will
  truncate at the first nul if a nul somehow reaches the C side.
- **raylib-allocated buffers** — some functions return raylib-managed byte slices wrapped as
  `ManuallyDrop<Box<[T]>>`. These must be freed via the matching `Unload*` or `MemFree`, not
  `libc::free`. See `DECISIONS.md`.

## See also

- [`core-concepts/safety.md`](./safety.md) — why FFI calls require careful handling.
- [`modules/raygui.md`](../modules/raygui.md) — the `impl AsRef<str>` raygui convention.
- [`modules/text-and-fonts.md`](../modules/text-and-fonts.md) — text rendering and font loading.
