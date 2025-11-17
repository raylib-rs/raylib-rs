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
        make_thin_wrapper_lifetime!($name, $t1, $t2, $dropfunc, true);
    };
    ($(#[$attrs:meta])* $name:ident, $t1:ty, $t2:ty,$dropfunc:expr, false) => {
        #[derive(Debug)]
        pub struct $name<'a>(pub(crate) $t1, &'a $t2);

        impl_wrapper!($name, $t1, $dropfunc, 0);
    };
    ($(#[$attrs:meta])* $name:ident, $t1:ty, $t2:ty, $dropfunc:expr, true) => {
        #[derive(Debug)]
        pub struct $name<'a>(pub(crate) $t1, &'a $t2);

        impl_wrapper!($name<'a>, $t1, $dropfunc, 0);
        deref_impl_wrapper!($name<'a>, $t1, $dropfunc, 0);
    };
}

macro_rules! impl_wrapper {
    ($name:ident$(<$lifetime:tt>)?, $t:ty, $dropfunc:expr, $rawfield:tt) => {
        impl$(<$lifetime>)? $name$(<$lifetime>)? {
            /// Take the raw ffi type. Must manually free memory by calling the proper unload function
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
            pub unsafe fn from_raw(raw: $t) -> Self {
                Self(raw)
            }
        }
    };
}

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

macro_rules! make_thick_wrapper {
    (
        $(#[$attrs:meta])*
        $vis:vis struct $WrapperTy:ident {
            $(
                $(#[$field_attrs:meta])*
                $field_vis:vis $field:ident: $FieldTy:ty
            ),* $(,)?
        }
        $(weak = $(#[$weak_attrs:meta])* $WeakTy:ident,)?
        raw = $RawTy:ty,
        drop = $dropfunc:expr $(,)?
    ) => {
        $(#[$attrs])*
        #[derive(Debug)]
        #[repr(C)]
        #[allow(non_snake_case)]
        pub struct $WrapperTy {
            $(
                $(#[$field_attrs])*
                $field_vis $field: $FieldTy
            ),*
        }

        #[allow(clippy::unnecessary_operation, clippy::identity_op)]
        const _: () = {
            [concat!("Size of ", stringify!($WrapperTy))][::std::mem::size_of::<$WrapperTy>() - ::std::mem::size_of::<$RawTy>()];
            [concat!("Alignment of ", stringify!($WrapperTy))][::std::mem::align_of::<$WrapperTy>() - ::std::mem::align_of::<$RawTy>()];
            $([concat!("Offset of field: ", stringify!($WrapperTy::$field))][::std::mem::offset_of!($WrapperTy, $field) - ::std::mem::offset_of!($RawTy, $field)];)*
        };

        impl Drop for $WrapperTy {
            fn drop(&mut self) {
                unsafe {
                    $dropfunc(self.clone_raw());
                }
            }
        }

    $(
        $(#[$weak_attrs])*
        #[derive(Debug)]
        #[repr(transparent)]
        pub struct $WeakTy(ManuallyDrop<$WrapperTy>);

        // Weak things can be clone
        impl Clone for $WeakTy {
            #[inline]
            fn clone(&self) -> Self {
                Self(ManuallyDrop::new(unsafe { $WrapperTy::from_raw_unchecked(self.0.clone_raw()) }))
            }
        }

        impl std::ops::Deref for $WeakTy {
            type Target = $WrapperTy;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl std::ops::DerefMut for $WeakTy {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
        impl AsRef<$WrapperTy> for $WeakTy {
            #[inline]
            fn as_ref(&self) -> &$WrapperTy {
                self
            }
        }
        impl AsMut<$WrapperTy> for $WeakTy {
            #[inline]
            fn as_mut(&mut self) -> &mut $WrapperTy {
                self
            }
        }
    )?

        impl $WrapperTy {
        $(
            /// # Safety
            /// Do not break Rust's aliasing rules.
            #[inline]
            #[must_use = "weak resources must be manually unloaded"]
            pub unsafe fn make_weak(self) -> $WeakTy {
                $WeakTy(ManuallyDrop::new(self))
            }
            /// # Safety
            /// - Do not break Rust's aliasing rules.
            #[doc = concat!(" - Other weak instances of this mesh must not be accessed after the [`", stringify!($WrapperTy), "`] is dropped.")]
            #[inline]
            pub unsafe fn from_weak(weak: $WeakTy) -> $WrapperTy {
                ManuallyDrop::into_inner(weak.0)
            }
        )?

            /// # Safety
            /// - Do not break Rust's aliasing rules.
            #[doc = concat!(" - Other raw instances of this mesh must not be accessed after the [`", stringify!($WrapperTy), "`] is dropped.")]
            #[inline]
            pub unsafe fn from_raw_unchecked(raw: $RawTy) -> $WrapperTy {
                unsafe { std::mem::transmute(raw) }
            }
            /// # Safety
            /// Do not break Rust's aliasing rules.
            #[inline]
            pub unsafe fn to_raw(self) -> $RawTy {
                unsafe { std::mem::transmute(self) }
            }

            #[doc = concat!(" This is safe as long as it isn't made public, because that would allow unsafe fields of [`", stringify!($WrapperTy), "`] to be misused.")]
            ///
            /// This may seem safer than [`Self::as_raw_mut`], but mutable pointers can be copied and their contents mutated from behind a shared reference.
            #[inline]
            pub(crate) fn clone_raw(&self) -> $RawTy {
                unsafe { std::mem::transmute_copy(self) }
            }

            #[doc = concat!(" This is safe as long as it isn't made public, because that would allow unsafe fields of [`", stringify!($WrapperTy), "`] to be misused.")]
            ///
            /// This may seem safer than [`Self::as_raw_mut`], but mutable pointers can be copied and their contents mutated from behind a shared reference.
            #[inline]
            pub(crate) fn as_raw_ref(&self) -> &$RawTy {
                unsafe { &*std::ptr::from_ref(self).cast() }
            }

            #[doc = concat!(" This is safe as long as it isn't made public, because that would allow unsafe fields of [`", stringify!($WrapperTy), "`] to be misused.")]
            #[inline]
            pub(crate) fn as_raw_mut(&mut self) -> &mut $RawTy {
                unsafe { &mut *std::ptr::from_mut(self).cast() }
            }
        }
    };
}
