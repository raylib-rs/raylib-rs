# Safety

raylib-rs is a safe Rust wrapper over a C library. The safe API surface — everything in
`raylib::prelude` — never requires `unsafe` from the caller. Internally, `unsafe` is used sparingly
and always justified.

The project follows two conventions enforced by code review:

- Every `unsafe fn` has a `/// # Safety` doc that states the preconditions the caller must uphold.
- Every `unsafe { ... }` block has a `// SAFETY:` comment explaining why the specific call is
  sound at that site.

These conventions make it easy to audit the unsafety budget during reviews and when upgrading
raylib's C version.

## API surface

**Conventions, not API items** — this chapter describes the disciplines baked into the codebase.
Key structural choices that influence safety:

- **RAII wrappers** (`Image`, `Texture2D`, `Font`, …) ensure resources are freed exactly once.
  No raw pointers are exposed to callers.
- **`RaylibThread: !Send + !Sync`** — the main-thread token is not sendable, so thread-sensitive
  raylib calls can only be made from the thread that called `init()`.
- **No `Send`/`Sync` for pointer-owning wrappers** — types that own opaque C pointers do not
  implement `AsRef`/`AsMut` in ways that could allow aliased mutable access (partial WS6 fix for
  #277).
- **Audio lifetimes** — `Sound`, `Music`, and `AudioStream` carry lifetime parameters tying them
  to the `RaylibAudio` that created them; the borrow checker prevents use-after-free.

## Example

The excerpt below is taken from `raylib/src/rlgl/mod.rs` and shows both conventions in action:

```text
/// Push the current matrix onto the stack; the returned guard pops it on drop.
/// Apply transforms (rl_translatef/rl_rotatef/rl_scalef) through the guard.
#[inline]
fn rl_push_matrix(&mut self) -> RlMatrix<'_, Self> {
    // SAFETY: the returned RlMatrix's Drop calls the matching rlPopMatrix.
    unsafe { ffi::rlPushMatrix() };
    RlMatrix::new(self)
}

/// Reset the current matrix to identity.
#[inline]
fn rl_load_identity(&mut self) {
    // SAFETY: rlLoadIdentity is an unconditional state mutation; no preconditions.
    unsafe { ffi::rlLoadIdentity() }
}
```

`rl_push_matrix` is itself a safe function — callers need no `unsafe`. The `// SAFETY:` comment
explains the invariant: the `RlMatrix` guard's `Drop` impl guarantees the matching `rlPopMatrix`
call, so the push/pop pair is always balanced.

A full `unsafe fn` with a `# Safety` doc follows the same pattern — see
`raylib/src/core/databuf.rs` for the `RlManaged` type, where each constructor documents its
preconditions on pointer provenance and allocator compatibility.

## Gotchas

- **Don't assume FFI calls are safe because the C looks innocuous.** raylib's C functions carry
  implicit preconditions (e.g., window must be initialised, texture must be uploaded, pointer
  must not be null). The binding encodes these into the Rust type system where possible; where it
  cannot (raw FFI escape hatches), `# Safety` docs spell them out.
- **`AsRef`/`AsMut` unsoundness** — pointer-owning wrappers do not implement `AsRef`/`AsMut` in
  ways that could permit aliased mutation of the same C object from multiple Rust references.
  The WS6-prep partial fix (#277) removed the most dangerous impls; a full refactor is
  tracked-deferred.
- **Skeletal-animation Drop** — the pre-6.0 code called `UnloadModelAnimations` via a raw
  `*mut` pointer in a way that could double-free. The WS3 redesign introduced `ModelAnimations`
  as a proper RAII struct with a safe `Drop` impl. If you are upgrading from 5.x, replace any
  manual animation teardown with the new type.

## See also

- [`CONTRIBUTE.md`](https://github.com/raylib-rs/raylib-rs/blob/unstable/CONTRIBUTE.md) — the full coding conventions including the `unsafe`
  discipline.
- [`core-concepts/raii-and-resources.md`](./raii-and-resources.md) — how RAII enforces resource
  safety.
- [docs.rs unsafe functions](https://docs.rs/raylib/latest/raylib/) — every `unsafe fn` in the
  public API lists its `# Safety` requirements.
