# Public: How do other engines / immediate-mode GUI libraries balance string ergonomics vs. per-frame C-string allocation?

Context: raygui (C) takes `const char*` (null-terminated). The safe binding must turn a Rust
string into a null-terminated pointer **every frame, for every control**. The 6.0-rc `safe.rs`
currently does `CString::new(text).unwrap()` per call (fresh heap alloc + panic on interior NUL).
CLAUDE.md's stated convention is `&CStr` + the `rstr!` macro (zero-alloc but macro-bound). Which
is the right balance?

## Findings

**Dear ImGui (C++) — the reference immediate-mode design.** Takes `const char*` (null-terminated).
The full UI is rebuilt every frame, so string params are hot. C++ callers typically pass literals
or `printf`-style formatting into an internal/stack buffer (`ImGui::Text("%d", x)`), avoiding heap
churn. There's a long-standing proposal to accept length-delimited `ImStr`/`string_view` so callers
needn't null-terminate ([ocornut/imgui #494](https://github.com/ocornut/imgui/issues/494)), but
null-terminated `const char*` remains the default. Lesson: the C layer *requires* null termination;
the binding owns the conversion cost.

**imgui-rs (Rust binding — the closest analog to raylib-rs + raygui).** This is the most directly
relevant precedent, and it *iterated away* from exactly the `rstr!`-style approach raylib-rs
documents:
- v0.8 removed `ImStr`/`ImString`; v0.9 removed the **`im_str!` macro** — the direct analog of
  raylib-rs's `rstr!` ([CHANGELOG](https://github.com/imgui-rs/imgui-rs/blob/main/CHANGELOG.md),
  [v0.8.0 release](https://github.com/imgui-rs/imgui-rs/releases/tag/v0.8.0)).
- All widget functions now take **`impl AsRef<str>`**. Migration guide: `im_str!("button")` →
  `"button"`; `&im_str!("age {}", 100)` → `format!("age {}", 100)`.
- Null-termination is handled by a **single reusable scratch buffer** held on the `Ui` context
  (`UiBuffer { buffer: Vec<u8>, max_len }`). The `scratch_txt` helper appends the string bytes + a
  `\0` and returns a pointer into that buffer; the buffer is cleared only when it grows past
  `max_len`. So it is **amortized zero-allocation** in steady state while keeping a fully ergonomic
  `&str`/`String`/`format!` API ([string.rs source](https://raw.githubusercontent.com/imgui-rs/imgui-rs/main/imgui/src/string.rs)).
  Variants like `scratch_txt_two` place multiple live strings at different offsets in the one
  buffer for multi-string calls.
- Their own design issue captured the tension explicitly: "I'm not yet sure if avoiding dynamic
  allocation is an unnecessary micro-optimization or an important design goal"
  ([issue #7](https://github.com/imgui-rs/imgui-rs/issues/7)). They resolved it with the scratch
  buffer — keeping ergonomics *and* avoiding per-call heap allocation.

**egui (pure-Rust immediate mode).** Takes `impl Into<String>` / `impl Into<WidgetText>` and
**allocates freely** (owns `String`s). Philosophy: a frame is ~1–2 ms and that's fine; if you are
allocation-bound, swap in a custom allocator (mimalloc/talc) for ~20% — they don't contort the API
to avoid allocation ([egui README](https://github.com/emilk/egui), [docs.rs/egui](https://docs.rs/egui/latest/egui/)).
Lesson: for typical UIs, per-frame string allocation is not actually a measured bottleneck.

**Unreal Engine (C++, retained Slate UI).** Three semantic string types
([Epic docs](https://dev.epicgames.com/documentation/en-us/unreal-engine/string-handling-in-unreal-engine)):
`FString` (mutable, dynamic, heap), `FName` (immutable, **interned/hashed** — one stored copy per
unique string, for identifiers), and `FText` (localized, **user-facing UI text**). UI uses `FText`.
Because Slate is retained-mode, per-frame allocation isn't the axis; the transferable lessons are
(a) **interning** repeated identifiers (`FName`) and (b) a dedicated user-facing text type. The
interning idea maps loosely to caching the C-string for repeated literal labels.

## Synthesis for raygui

The closest precedent (imgui-rs) deliberately abandoned the `rstr!`-equivalent macro in favor of
`impl AsRef<str>` backed by a **reusable per-frame scratch buffer**, achieving both ergonomics and
amortized zero per-frame heap allocation. egui shows that even naive per-frame allocation is
usually not a real bottleneck. Unreal contributes the interning idea for repeated identifiers.

That yields three viable conventions for the safe raygui layer, in rough order of
ergonomics-vs-allocation balance:

1. **`impl AsRef<str>` + reusable scratch buffer (imgui-rs model).** Ergonomic (`"x"`,
   `format!(...)`, `String`), amortized zero-alloc, no NUL panic. Needs buffer infra
   (`scratch_txt` / `_opt` / `_two`) and a place to hold the buffer (thread-local fits raylib's
   single-threaded `!Send` draw model, or on the handle since gui methods take `&mut self`).
   Optional/nullable text → `Option<impl AsRef<str>>`.
2. **`Option<&CStr>` + `rstr!` (current canonical raylib-rs / showcase).** Zero-alloc for literals,
   but macro-bound and awkward for dynamic strings.
3. **`&str` + per-call `CString::new().unwrap()` (current 6.0-rc).** Simplest/ergonomic but
   per-frame heap alloc + NUL panic; contradicts CLAUDE.md.

## Sources

- [imgui-rs CHANGELOG](https://github.com/imgui-rs/imgui-rs/blob/main/CHANGELOG.md) — ImStr/im_str! removal, AsRef<str> migration
- [imgui-rs v0.8.0 release](https://github.com/imgui-rs/imgui-rs/releases/tag/v0.8.0) — deprecation + migration guide
- [imgui-rs string.rs](https://raw.githubusercontent.com/imgui-rs/imgui-rs/main/imgui/src/string.rs) — UiBuffer / scratch_txt reusable-buffer mechanism
- [imgui-rs issue #7](https://github.com/imgui-rs/imgui-rs/issues/7) — API design debate on string allocation
- [ocornut/imgui issue #494](https://github.com/ocornut/imgui/issues/494) — ImStr/string_view length-delimited proposal
- [egui README](https://github.com/emilk/egui) / [docs.rs/egui](https://docs.rs/egui/latest/egui/) — Into<String>, allocate-freely philosophy, allocator note
- [Unreal: String Handling](https://dev.epicgames.com/documentation/en-us/unreal-engine/string-handling-in-unreal-engine) — FString/FName/FText semantics
- [FString vs FName vs Text (Suneet Agrawal)](https://agrawalsuneet.github.io/blogs/fstring-vs-fname-vs-text-in-unreal/) — when-to-use summary
