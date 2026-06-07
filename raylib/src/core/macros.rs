/// Escape-hatch mutable access to a wrapper's raw FFI value.
///
/// Implemented by every thin wrapper. For wrappers classified *sound*
/// (see the per-type `// SOUNDNESS:` comments at the `make_thin_wrapper!`
/// call sites) this duplicates `AsMut`; for *readonly* wrappers it is the
/// only mutable access to the raw struct — and it is `unsafe`, because
/// the wrapper's safe API trusts invariants of the raw fields (issue #276).
pub trait AsRawMut<T> {
    /// Mutable access to the wrapped raw FFI value.
    ///
    /// # Safety
    ///
    /// Callers must uphold the wrapper's invariants:
    /// - do **not** corrupt count/dimension/format fields that safe
    ///   accessors use to derive slice lengths or buffer sizes (e.g.
    ///   `Mesh::vertexCount`, `Image::width`/`height`/`format`), and
    /// - do **not** reassign pointer fields that are owned and freed by
    ///   `Drop` (e.g. `Model::meshes`, `Font::glyphs`), unless you take
    ///   over ownership of the previous allocation.
    unsafe fn as_raw_mut(&mut self) -> &mut T;
}

/// Internal helper. Generates a `#[repr(transparent)]` newtype wrapping an
/// FFI value, with `Drop` calling the supplied `Unload*` raylib function.
///
/// Use this when a raylib type is owned by Rust outright (no parent handle
/// keeps it alive). The generated wrapper is `Debug`, has a private
/// positional field, and gets the standard set of impls from
/// `impl_wrapper!`, `gen_from_raw_wrapper!`, and (by default)
/// `deref_impl_wrapper!`. Pass a trailing `false` to suppress the
/// `Deref` / `DerefMut` / `AsRef` / `AsMut` impls when the wrapper should
/// hide its inner FFI value (e.g. when an explicit accessor API is
/// preferred — see `AutomationEventList`).
///
/// The wrapper struct is `!Send` / `!Sync` only insofar as the wrapped FFI
/// type is — most raylib types are non-thread-safe by construction, so in
/// practice these wrappers must stay on the raylib thread.
///
/// Expansion shape (conceptual):
///
/// ```text
/// #[repr(transparent)]
/// #[derive(Debug)]
/// pub struct $name(pub(crate) $t);
///
/// // Always:
/// impl Drop  for $name { fn drop(&mut self) { ($dropfunc)(self.0); } }
/// impl $name { pub unsafe fn unwrap(self) -> $t { ... } }
/// // With the default (`, true`) variant:
/// impl AsRef<$t>  for $name { ... }
/// impl AsMut<$t>  for $name { ... }
/// impl Deref      for $name { type Target = $t; ... }
/// impl DerefMut   for $name { ... }
/// impl $name { pub fn to_raw(self) -> $t { ... } pub unsafe fn from_raw(raw: $t) -> Self { ... } }
/// ```
///
/// Used internally to wrap `Image`, `Texture2D`, `RenderTexture2D`, `Font`,
/// `Mesh`, `Shader`, `Material`, `Model`, `ModelAnimation`, `VrStereoConfig`,
/// `FilePathList`, `AutomationEventList`, `AutomationEvent`, and the
/// borrow-checker-only `Weak*` aliases (which pass a no-op `no_drop`
/// function so they don't free the inner value).
macro_rules! make_thin_wrapper {
    ($(#[$attrs:meta])* $name:ident, $t:ty, $dropfunc:expr) => {
        make_thin_wrapper!($(#[$attrs])* $name, $t, $dropfunc, true);
    };
    ($(#[$attrs:meta])* $name:ident, $t:ty, $dropfunc:expr, false) => {
        $(#[$attrs])*
        #[repr(transparent)]
        #[derive(Debug)]
        #[allow(missing_docs)]
        pub struct $name(pub(crate) $t);

        impl_wrapper!($name, $t, $dropfunc, 0);
        gen_from_raw_wrapper!($name, $t, $dropfunc, 0);
    };
    ($(#[$attrs:meta])* $name:ident, $t:ty, $dropfunc:expr, true) => {
        $(#[$attrs])*
        #[repr(transparent)]
        #[derive(Debug)]
        #[allow(missing_docs)]
        pub struct $name(pub(crate) $t);

        impl_wrapper!($name, $t, $dropfunc, 0);
        deref_impl_wrapper!($name, $t, $dropfunc, 0);
        gen_from_raw_wrapper!($name, $t, $dropfunc, 0);
    };
    ($(#[$attrs:meta])* $name:ident, $t:ty, $dropfunc:expr, readonly) => {
        $(#[$attrs])*
        #[repr(transparent)]
        #[derive(Debug)]
        #[allow(missing_docs)]
        pub struct $name(pub(crate) $t);

        impl_wrapper!($name, $t, $dropfunc, 0);
        readonly_deref_impl_wrapper!($name, $t, $dropfunc, 0);
        gen_from_raw_wrapper!($name, $t, $dropfunc, 0);
    };
}

/// Internal helper. Like [`make_thin_wrapper!`] but with a lifetime
/// parameter that ties the wrapped FFI value to a parent borrow.
///
/// Use this when a raylib resource is logically owned by another handle —
/// e.g. every [`Wave`], [`Sound`], [`Music`], and [`AudioStream`] is bound
/// to the lifetime of the `RaylibAudio` that initialised the audio device.
/// The generated struct holds an extra `&'a $t2` field so the borrow
/// checker prevents the child outliving its parent. As with
/// [`make_thin_wrapper!`], a trailing `false` suppresses the `Deref` /
/// `AsRef` family for wrappers that want a hand-written accessor surface.
///
/// Expansion shape (conceptual):
///
/// ```text
/// #[derive(Debug)]
/// pub struct $name<'a>(pub(crate) $t1, &'a $t2);
///
/// // Always:
/// impl<'a> Drop for $name<'a> { fn drop(&mut self) { ($dropfunc)(self.0); } }
/// impl<'a> $name<'a> { pub unsafe fn unwrap(self) -> $t1 { ... } }
/// // With the default (`, true`) variant:
/// impl<'a> AsRef<$t1> for $name<'a> { ... }
/// impl<'a> AsMut<$t1> for $name<'a> { ... }
/// impl<'a> Deref      for $name<'a> { type Target = $t1; ... }
/// impl<'a> DerefMut   for $name<'a> { ... }
/// ```
///
/// Note: this macro deliberately does **not** invoke
/// `gen_from_raw_wrapper!`, because constructing a lifetime-bound wrapper
/// from a raw FFI value requires a matching borrow of the parent — the
/// owning module supplies its own constructors (e.g.
/// `RaylibAudio::new_sound`) instead.
///
/// [`Wave`]: crate::core::audio::Wave
/// [`Sound`]: crate::core::audio::Sound
/// [`Music`]: crate::core::audio::Music
/// [`AudioStream`]: crate::core::audio::AudioStream
macro_rules! make_thin_wrapper_lifetime {
    ($(#[$attrs:meta])* $name:ident, $t1:ty, $t2:ty, $dropfunc:expr) => {
        make_thin_wrapper_lifetime!($name, $t1, $t2, $dropfunc, true);
    };
    ($(#[$attrs:meta])* $name:ident, $t1:ty, $t2:ty,$dropfunc:expr, false) => {
        $(#[$attrs])*
        #[derive(Debug)]
        #[allow(missing_docs)]
        pub struct $name<'a>(pub(crate) $t1, &'a $t2);

        impl_wrapper!($name, $t1, $dropfunc, 0);
    };
    ($(#[$attrs:meta])* $name:ident, $t1:ty, $t2:ty, $dropfunc:expr, true) => {
        $(#[$attrs])*
        #[derive(Debug)]
        #[allow(missing_docs)]
        pub struct $name<'a>(pub(crate) $t1, &'a $t2);

        impl_wrapper!($name<'a>, $t1, $dropfunc, 0);
        deref_impl_wrapper!($name<'a>, $t1, $dropfunc, 0);
    };
    ($(#[$attrs:meta])* $name:ident, $t1:ty, $t2:ty, $dropfunc:expr, readonly) => {
        $(#[$attrs])*
        #[derive(Debug)]
        #[allow(missing_docs)]
        pub struct $name<'a>(pub(crate) $t1, &'a $t2);

        impl_wrapper!($name<'a>, $t1, $dropfunc, 0);
        readonly_deref_impl_wrapper!($name<'a>, $t1, $dropfunc, 0);
    };
}

/// Internal helper. Emits the always-present pieces of a thin wrapper:
/// the `Drop` impl that calls `$dropfunc`, and an `unsafe fn unwrap`
/// associated function that consumes the wrapper and returns the raw FFI
/// value without running `Drop` (the caller takes over freeing it).
///
/// Invoked from inside [`make_thin_wrapper!`] and
/// [`make_thin_wrapper_lifetime!`] — not intended as a standalone entry
/// point. The optional `$lifetime` token lets the same expansion serve
/// both the non-generic and `<'a>`-generic wrappers.
///
/// Expansion shape (conceptual):
///
/// ```text
/// impl[<'a>] $name[<'a>] {
///     pub unsafe fn unwrap(self) -> $t { /* mem::forget(self) and return inner */ }
/// }
/// impl[<'a>] Drop for $name[<'a>] {
///     fn drop(&mut self) { unsafe { ($dropfunc)(self.$rawfield); } }
/// }
/// ```
macro_rules! impl_wrapper {
    ($name:ident$(<$lifetime:tt>)?, $t:ty, $dropfunc:expr, $rawfield:tt) => {
        impl$(<$lifetime>)? $name$(<$lifetime>)? {
            /// Take the raw ffi type. Must manually free memory by calling the proper unload function
            ///
            /// # Safety
            ///
            /// The caller is responsible for freeing the returned value by calling
            /// the appropriate raylib unload function. Failure to do so will leak resources.
            pub unsafe fn unwrap(self) -> $t {
                let inner = self.$rawfield;
                std::mem::forget(self);
                inner
            }
        }

        impl$(<$lifetime>)? Drop for $name$(<$lifetime>)? {
            #[allow(unused_unsafe)]
            fn drop(&mut self) {
                unsafe {
                    ($dropfunc)(self.$rawfield);
                }
            }
        }


    };
}

/// Internal helper. Emits the raw-FFI conversion surface for wrappers that
/// can be freely constructed from a raylib-sys value:
///
/// * `to_raw(self) -> $t` — non-`unsafe` counterpart of `unwrap`, used when
///   the destination type doesn't need callers to think about ownership
///   ceremony beyond "you now own this raylib resource."
/// * `unsafe fn from_raw(raw: $t) -> Self` — takes ownership of an FFI
///   value loaded elsewhere (e.g. from raylib functions that return by
///   value) and wraps it for RAII cleanup.
///
/// Invoked from [`make_thin_wrapper!`]. Deliberately **omitted** from
/// [`make_thin_wrapper_lifetime!`] expansions because a lifetime-bound
/// child needs an explicit borrow of its parent that this macro can't
/// supply.
///
/// Expansion shape (conceptual):
///
/// ```text
/// impl[<'a>] $name[<'a>] {
///     pub        fn to_raw(self)        -> $t { /* mem::forget + return */ }
///     pub unsafe fn from_raw(raw: $t)   -> Self { Self(raw) }
/// }
/// ```
macro_rules! gen_from_raw_wrapper {
    ($name:ident$(<$lifetime:tt>)?, $t:ty, $dropfunc:expr, $rawfield:tt) => {
        impl$(<$lifetime>)? $name$(<$lifetime>)? {
            /// returns the unwrapped raylib-sys object
            pub fn to_raw(self) -> $t {
                let raw = self.$rawfield;
                std::mem::forget(self);
                raw
            }

            /// converts raylib-sys object to a "safe"
            /// version. Make sure to call this function
            /// from the thread the resource was created.
            ///
            /// # Safety
            ///
            /// The caller must ensure `raw` is a valid, fully initialized raylib object
            /// obtained from a raylib load function. Ownership is transferred to the
            /// returned wrapper, which will call the appropriate unload function on drop.
            pub unsafe fn from_raw(raw: $t) -> Self {
                Self(raw)
            }
        }
    };
}

/// Internal helper. Emits the `AsRef` / `AsMut` / `Deref` / `DerefMut`
/// impls that expose the wrapped FFI value as the wrapper's `Target`.
///
/// Use this when callers should be able to pass `&wrapper` everywhere a
/// `&ffi::T` is expected — that lets the safe wrappers participate in
/// raylib's FFI surface without needing a separate `.as_raw()` call. Some
/// wrappers (e.g. `AutomationEventList`) opt out by passing the trailing
/// `false` to [`make_thin_wrapper!`], because they want to expose a
/// hand-written, type-safe accessor API rather than handing out the raw
/// FFI struct.
///
/// Expansion shape (conceptual):
///
/// ```text
/// impl[<'a>] AsRef<$t>  for $name[<'a>] { fn as_ref(&self) -> &$t       { &self.$rawfield } }
/// impl[<'a>] AsMut<$t>  for $name[<'a>] { fn as_mut(&mut self) -> &mut $t { &mut self.$rawfield } }
/// impl[<'a>] Deref      for $name[<'a>] { type Target = $t; fn deref(&self) -> &$t { ... } }
/// impl[<'a>] DerefMut   for $name[<'a>] { fn deref_mut(&mut self) -> &mut $t { ... } }
/// impl[<'a>] AsRawMut<$t> for $name[<'a>] { unsafe fn as_raw_mut(&mut self) -> &mut $t { ... } }
/// ```
macro_rules! deref_impl_wrapper {
    ($name:ident$(<$lifetime:tt>)?, $t:ty, $dropfunc:expr, $rawfield:tt) => {
        impl$(<$lifetime>)? std::convert::AsRef<$t> for $name$(<$lifetime>)? {
            fn as_ref(&self) -> &$t {
                &self.$rawfield
            }
        }

        impl$(<$lifetime>)? std::convert::AsMut<$t> for $name$(<$lifetime>)? {
            fn as_mut(&mut self) -> &mut $t {
                &mut self.$rawfield
            }
        }

        impl$(<$lifetime>)? std::ops::Deref for $name$(<$lifetime>)? {
            type Target = $t;
            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.$rawfield
            }
        }

        impl$(<$lifetime>)? std::ops::DerefMut for $name$(<$lifetime>)? {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$rawfield
            }
        }

        impl$(<$lifetime>)? crate::core::AsRawMut<$t> for $name$(<$lifetime>)? {
            unsafe fn as_raw_mut(&mut self) -> &mut $t {
                &mut self.$rawfield
            }
        }
    };
}

/// Internal helper. Read half of the deref family only (`Deref` +
/// `AsRef`), plus the `unsafe` [`AsRawMut`] escape hatch. Used by the
/// `readonly` wrapper mode for types whose safe API trusts raw fields
/// (issue #276) — see the `// SOUNDNESS:` comment at each call site.
macro_rules! readonly_deref_impl_wrapper {
    ($name:ident$(<$lifetime:tt>)?, $t:ty, $dropfunc:expr, $rawfield:tt) => {
        impl$(<$lifetime>)? std::convert::AsRef<$t> for $name$(<$lifetime>)? {
            fn as_ref(&self) -> &$t {
                &self.$rawfield
            }
        }

        impl$(<$lifetime>)? std::ops::Deref for $name$(<$lifetime>)? {
            type Target = $t;
            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.$rawfield
            }
        }

        impl$(<$lifetime>)? crate::core::AsRawMut<$t> for $name$(<$lifetime>)? {
            unsafe fn as_raw_mut(&mut self) -> &mut $t {
                &mut self.$rawfield
            }
        }
    };
}

/// Internal helper. Generates a wrapper for a raylib-allocated slice
/// (`ManuallyDrop<Box<[T]>>`) that is freed via the supplied
/// `Unload*` / `MemFree` function instead of the global Rust allocator.
///
/// raylib functions like `LoadImageColors` and `LoadImagePalette` return a
/// pointer plus a length — Rust can borrow that as a `Box<[T]>` for safe
/// indexed access, but **must not** let `Box`'s `Drop` call `free()` on
/// the buffer because raylib may have been built with a custom allocator
/// (see `DECISIONS.md`). The wrapper holds the box inside
/// [`std::mem::ManuallyDrop`] and routes its own `Drop` through
/// `$dropfunc` (typically `ffi::UnloadImageColors`,
/// `ffi::UnloadImagePalette`, or `MemFree`).
///
/// Used internally for `ImagePalette` and `ImageColors`.
///
/// Expansion shape (conceptual):
///
/// ```text
/// #[repr(transparent)]
/// #[derive(Debug)]
/// pub struct $name(pub(crate) std::mem::ManuallyDrop<Box<[$t]>>);
///
/// impl Drop      for $name { /* takes the Box, leaks it back to raw, and calls ($dropfunc)(ptr) */ }
/// impl AsRef<Box<[$t]>>  for $name { ... }
/// impl Deref     for $name { type Target = Box<[$t]>; ... }
/// impl $name { pub fn as_mut_slice(&mut self) -> &mut [$t] { ... } }
/// // No AsMut/DerefMut to the Box: swapping the allocation out (mem::take)
/// // would free the raylib-owned buffer via the global allocator (issue #276).
/// ```
macro_rules! make_rslice {
    ($(#[$attrs:meta])* $name:ident, $t:ty, $dropfunc:expr) => {
        $(#[$attrs])*
        #[repr(transparent)]
        #[derive(Debug)]
        #[allow(missing_docs)]
        pub struct $name(pub(crate) std::mem::ManuallyDrop<std::boxed::Box<[$t]>>);

        impl_rslice!($name, std::boxed::Box<[$t]>, $dropfunc, 0);
    };
}

/// Internal helper. Emits the `Drop` + `AsRef` / `Deref` impls for an
/// rslice wrapper produced by [`make_rslice!`].
///
/// The `Drop` impl is the load-bearing piece: it takes the inner
/// `ManuallyDrop<Box<[T]>>`, calls `Box::leak` to recover the raw pointer
/// without running the global allocator's free, and then hands that
/// pointer to `$dropfunc` (the matching raylib `Unload*` /
/// `MemFree`). This keeps the wrapper correct under raylib builds that
/// use a custom allocator.
///
/// The `AsMut<Box<[T]>>` and `DerefMut` impls are intentionally absent:
/// exposing `&mut Box<[Color]>` would allow `mem::take` to drop the inner
/// `Box` through the global allocator, bypassing the raylib-owned `Unload*`
/// free path — a double-free. Use `as_mut_slice` for element mutation
/// instead (see issue #276).
///
/// Not intended to be called outside of [`make_rslice!`].
///
/// Expansion shape (conceptual):
///
/// ```text
/// impl Drop for $name { /* see above */ }
/// impl AsRef<Box<[T]>> for $name { ... }
/// impl Deref     for $name { type Target = Box<[T]>; ... }
/// impl $name { pub fn as_mut_slice(&mut self) -> &mut [Color] { ... } }
/// ```
macro_rules! impl_rslice {
    ($name:ident, $t:ty, $dropfunc:expr, $rawfield:tt) => {
        impl Drop for $name {
            #[allow(unused_unsafe)]
            fn drop(&mut self) {
                unsafe {
                    let inner = std::mem::ManuallyDrop::take(&mut self.0);
                    ($dropfunc)(std::boxed::Box::leak(inner).as_mut_ptr() as *mut _);
                }
            }
        }

        impl std::convert::AsRef<$t> for $name {
            fn as_ref(&self) -> &$t {
                &self.$rawfield
            }
        }

        impl std::ops::Deref for $name {
            type Target = $t;
            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.$rawfield
            }
        }

        impl $name {
            /// Mutable access to the elements. The `Box` itself is not
            /// exposed (replacing it would free the raylib-owned buffer
            /// through the global allocator — see issue #276).
            #[inline]
            pub fn as_mut_slice(&mut self) -> &mut [crate::ffi::Color] {
                &mut self.$rawfield[..]
            }
        }
    };
}
