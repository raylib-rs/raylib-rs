# Contributing

PRs are always welcome!

We're working on porting all examples from C -> rust.
You can convert C programs in `showcase/original` to rust programs in `showcase/src/example`

## Safety

Please remember to use `unsafe` sparingly--only when absolutely necessary and absolutely certain. If you *must* write `unsafe` code, please uphold the following guidelines:

- Try to document any contributed `unsafe` functions with `/// # Safety` to communicate *why* a function is `unsafe` and how to use it safely.
  
- Try to document any `unsafe` blocks you write with a `// SAFETY:` message to explain *why* an operation is **sound** (unable to result in [Undefined Behavior](https://doc.rust-lang.org/reference/behavior-considered-undefined.html)).
  
A good way to do this is by looking at the `/// # Safety` docs of each `unsafe` function/operation being performed within your `unsafe` block, then listing off *how* your function is ensuring those requirements.

### Examples

- If it is safe "so long as the function is used as intended", then write something like "Caller must uphold safety contract", **mark the function itself as `unsafe`, and write a `/// # Safety` doc describing the exact requirements for "intended" use.**
  ```rs
  /// Assigns an [`i32`] pointer with 5
  ///
  /// # Safety
  ///
  /// `ptr` must be safe to dereference and write to.
  pub unsafe fn set_5(ptr: *mut i32) {
      // SAFETY: Caller must uphold safety contract.
      unsafe { *ptr } = 5;
  }
  ```
  
- If it is safe because the function either branches (`if`/`match`) or exits (`return`/`panic`/`assert`/etc.) in such a way that the `unsafe` block is only reached in a case where its operations are guaranteed to be sound, write something like
  ```rs
  if (x % 2) == 1 {
      // SAFETY: Already checked and x is odd.
      unsafe {
          odd_numbers_only(x);
      }
  }
  ```

- If it is safe because of something inherent that the compiler doesn't know, simply explain it.
  ```rs
  // SAFETY: 5 is not 0.
  unsafe { NonZeroI32::new_unchecked(5) }
  ```
  
- If it is safe because it calls an `ffi` function that is confirmed to be safe, then write something like
  ```rs
  // SAFETY: `Foo` has no preconditions.
  unsafe { ffi::AddOne(x) }
  ```
  ```c
  int AddOne(int x)
  {
      return x + 1; // note that the return can overflow
  }
  ```
  **WARNING: `ffi` functions are `unsafe` because Rust cannot give the same guarantees for them as it could if they were written in Rust. Do not assume an `ffi` function is safe just because it does something that would be safe in Rust. Anything that would cause a panic in rust will cause Undefined Behavior or crash in C.**
  ```rs
  fn sound_incr(n: u8) -> NonZeroU8 {
      let m: u8 = n + 1;
      // SAFETY: `m` is at least 1 (overflow would panic).
      unsafe { NonZeroU8::new_unchecked() };
  }
  ```
  This alternative version looks like it would be the same, except it isn't.
  ```rs
  fn unsound_incr(n: u8) -> NonZeroU8 {
      // SAFETY: `AddOne` has no preconditions
      let m: u8 = unsafe { AddOne(n) }; // `AddOne` just adds 1 to `n`
      // SAFETY: `m` is at least 1 (overflow would panic).
      unsafe { NonZeroU8::new_unchecked() };
  }
  ```
  In `unsound_incr`, `AddOne` has no *preconditions*, but does not give the same protections as Rust would. If `n` is 255, `sound_incr` would panic, but `unsound_incr` would return **an invalid `NonZeroU8` containing 0**.

## Working on the book

The user guide lives in `book/` and is built with [mdBook](https://rust-lang.github.io/mdBook/). Install once with `cargo install mdbook` (or use the official release binary), then from the repo root:

- `mdbook serve book` — live preview at <http://localhost:3000>.
- `mdbook build book` — write the static HTML to `book/book/`.
- `mdbook test book -L target/debug/deps` — run the book's Rust code blocks as doctests (requires `cargo build -p raylib --features full` first).

CI builds and tests the book on every push (`book.yml`). The public Pages deploy ships with the WS9 showcase.
