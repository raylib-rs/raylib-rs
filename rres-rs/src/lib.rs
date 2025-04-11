mod rres;

#[cfg(feature = "raylib")]
mod raylib;

pub use rres::*;

#[cfg(feature = "raylib")]
pub use raylib::*;

/// Check if a struct is zeroed out
pub(crate) fn is_zero<T: Sized>(p: &T) -> bool {
    unsafe {
        ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())[0]
            == 0
    }
}

pub use rres_sys as ffi;
