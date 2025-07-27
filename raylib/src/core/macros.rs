macro_rules! make_thin_wrapper {
    ($(#[$attrs:meta])* $name:ident, $t:ty, $dropfunc:expr) => {
        make_thin_wrapper!($(#[$attrs])* $name, $t, $dropfunc, true);
    };
    ($(#[$attrs:meta])* $name:ident, $t:ty, $dropfunc:expr, false) => {
        $(#[$attrs])*
        #[repr(transparent)]
        #[derive(Debug)]
        pub struct $name(pub(crate) $t);

        impl_wrapper!($name, $t, $dropfunc, 0);
        gen_from_raw_wrapper!($name, $t, $dropfunc, 0);
    };
    ($(#[$attrs:meta])* $name:ident, $t:ty, $dropfunc:expr, true) => {
        $(#[$attrs])*
        #[repr(transparent)]
        #[derive(Debug)]
        pub struct $name(pub(crate) $t);

        impl_wrapper!($name, $t, $dropfunc, 0);
        deref_impl_wrapper!($name, $t, $dropfunc, 0);
        gen_from_raw_wrapper!($name, $t, $dropfunc, 0);
    };
}

macro_rules! make_thin_wrapper_lifetime {
    ($(#[$attrs:meta])* $name:ident, $t1:ty, $t2:ty, $dropfunc:expr) => {
        make_thin_wrapper_lifetime!($(#[$attrs])* $name, $t1, $t2, $dropfunc, true);
    };
    ($(#[$attrs:meta])* $name:ident, $t1:ty, $t2:ty,$dropfunc:expr, false) => {
        $(#[$attrs])*
        #[derive(Debug)]
        pub struct $name<'a>(pub(crate) $t1, &'a $t2);

        impl_wrapper!($name, $t1, $dropfunc, 0);
    };
    ($(#[$attrs:meta])* $name:ident, $t1:ty, $t2:ty, $dropfunc:expr, true) => {
        $(#[$attrs])*
        #[derive(Debug)]
        pub struct $name<'a>(pub(crate) $t1, &'a $t2);

        impl_wrapper!($name<'a>, $t1, $dropfunc, 0);
        deref_impl_wrapper!($name<'a>, $t1, $dropfunc, 0);
    };
}

macro_rules! impl_wrapper {
    ($name:ident$(<$lifetime:tt>)?, $t:ty, $dropfunc:expr, $rawfield:tt) => {
        impl$(<$lifetime>)? $name$(<$lifetime>)? {
            /// Take the raw ffi type.
            ///
            /// # Safety
            ///
            /// Must manually free memory by calling the proper unload function.
            /// Even if `self` implements [`Copy`], exactly one instance should be unloaded to avoid double-free,
            /// and copies must not be used after being unloaded to avoid use-after-free.
            #[must_use]
            pub const unsafe fn unwrap(self) -> $t {
                let inner = self.$rawfield;
                std::mem::forget(self);
                inner
            }
        }

        impl$(<$lifetime>)? Drop for $name$(<$lifetime>)? {
            #[allow(
                unused_unsafe,
                reason = "$dropfunc is not unsafe in every instance of this macro"
            )]
            fn drop(&mut self) {
                // SAFETY: `Self` does not implement `Clone`, and `unwrap` takes ownership of `self`.
                // Taking `self` by mutable reference guarantees it has no aliases.
                unsafe {
                    ($dropfunc)(self.$rawfield);
                }
            }
        }
    };
}

macro_rules! gen_from_raw_wrapper {
    ($name:ident$(<$lifetime:tt>)?, $t:ty, $dropfunc:expr, $rawfield:tt) => {
        impl$(<$lifetime>)? $name$(<$lifetime>)? {
            /// Returns the unwrapped raylib-sys object.
            #[must_use]
            pub const fn to_raw(self) -> $t {
                let raw = self.$rawfield;
                std::mem::forget(self);
                raw
            }

            /// Converts raylib-sys object to a "safe" version.
            ///
            /// # Safety
            ///
            /// Make sure to call this function from the thread the resource was created.
            /// If there are any aliases to `raw`, they must not be used after this function's return drops,
            /// and Rust's aliasing rules must be ensured by the caller.
            #[must_use]
            pub const unsafe fn from_raw(raw: $t) -> Self {
                Self(raw)
            }
        }
    };
}

macro_rules! deref_impl_wrapper {
    ($name:ident$(<$lifetime:tt>)?, $t:ty, $dropfunc:expr, $rawfield:tt) => {
        impl$(<$lifetime>)? std::convert::AsRef<$t> for $name$(<$lifetime>)? {
            fn as_ref(&self) -> &$t {
                self
            }
        }

        impl$(<$lifetime>)? std::convert::AsMut<$t> for $name$(<$lifetime>)? {
            fn as_mut(&mut self) -> &mut $t {
                self
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
    };
}

macro_rules! make_rslice {
    ($(#[$attrs:meta])* $name:ident, $t:ty, $dropfunc:expr) => {
        $(#[$attrs])*
        #[repr(transparent)]
        #[derive(Debug)]
        pub struct $name(pub(crate) std::mem::ManuallyDrop<std::boxed::Box<[$t]>>);

        impl_rslice!($name, std::boxed::Box<[$t]>, $dropfunc, 0);
    };
}

macro_rules! impl_rslice {
    ($name:ident, $t:ty, $dropfunc:expr, $rawfield:tt) => {
        impl Drop for $name {
            #[allow(
                unused_unsafe,
                reason = "$dropfunc is not unsafe in every instance of this macro"
            )]
            fn drop(&mut self) {
                let inner = unsafe { std::mem::ManuallyDrop::take(&mut self.0) };
                unsafe { ($dropfunc)(std::boxed::Box::leak(inner).as_mut_ptr().cast()) };
            }
        }

        impl std::convert::AsRef<$t> for $name {
            fn as_ref(&self) -> &$t {
                &self.$rawfield
            }
        }

        impl std::convert::AsMut<$t> for $name {
            fn as_mut(&mut self) -> &mut $t {
                &mut self.$rawfield
            }
        }

        impl std::ops::Deref for $name {
            type Target = $t;
            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.$rawfield
            }
        }

        impl std::ops::DerefMut for $name {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$rawfield
            }
        }
    };
}
