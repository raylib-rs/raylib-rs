#[macro_export]
macro_rules! impl_wrapper {
    ($name:ident, $t:ty, $dropfunc:expr, $rawfield:tt) => {
        impl $name {
            /// Take the raw ffi type. Must manually free memory by calling the proper unload function
            pub unsafe fn unwrap(self) -> $t {
                let inner = self.$rawfield;
                std::mem::forget(self);
                inner
            }

            /// converts raylib-sys object to a "safe"
            /// version. Make sure to call this function
            /// from the thread the resource was created.
            pub unsafe fn from_raw(raw: $t) -> Self {
                Self(raw)
            }
        }

        impl Drop for $name {
            #[allow(unused_unsafe)]
            fn drop(&mut self) {
                unsafe {
                    ($dropfunc)(self.$rawfield);
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

        impl $name {
            pub fn as_raw(&self) -> &$t {
                &self.$rawfield
            }
        }
    };
}

#[macro_export]
macro_rules! impl_wrapper_bounded {
    ($name:ident, $t:ty, $dropfunc:expr, $rawfield:tt) => {
        impl<'bind> $name<'bind> {
            /// Take the raw ffi type. Must manually free memory by calling the proper unload function
            pub unsafe fn unwrap(self) -> $t {
                let inner = self.$rawfield;
                std::mem::forget(self);
                inner
            }

            /// converts raylib-sys object to a "safe"
            /// version. Make sure to call this function
            /// from the thread the resource was created.
            pub unsafe fn from_raw(raw: $t) -> Self {
                Self(raw, std::marker::PhantomData)
            }
        }

        impl<'bind> Drop for $name<'bind> {
            #[allow(unused_unsafe)]
            fn drop(&mut self) {
                unsafe {
                    ($dropfunc)(self.$rawfield);
                }
            }
        }

        impl<'bind> std::convert::AsRef<$t> for $name<'bind> {
            fn as_ref(&self) -> &$t {
                &self.$rawfield
            }
        }

        impl<'bind> std::convert::AsMut<$t> for $name<'bind> {
            fn as_mut(&mut self) -> &mut $t {
                &mut self.$rawfield
            }
        }

        impl<'bind> $name<'bind> {
            pub fn as_raw(&self) -> &$t {
                &self.$rawfield
            }
        }
    };
}

#[macro_export]
macro_rules! make_thin_wrapper {
    ($name:ident, $t:ty, $dropfunc:expr) => {
        #[repr(transparent)]
        #[derive(Debug)]
        pub struct $name(pub(crate) $t);

        $crate::impl_wrapper!($name, $t, $dropfunc, 0);
    };
}

#[macro_export]
macro_rules! make_bound_thin_wrapper {
    ($name:ident, $t:ty, $dropfunc:expr, $binding:ty) => {
        #[repr(transparent)]
        #[derive(Debug)]
        pub struct $name<'bind>(
            pub(crate) $t,
            pub(crate) std::marker::PhantomData<&'bind $binding>,
        );

        $crate::impl_wrapper_bounded!($name, $t, $dropfunc, 0);
    };
}
