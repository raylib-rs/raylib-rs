//! Useful functions that don't fit anywhere else

use crate::{
    core::{RaylibHandle, RaylibThread, texture::Image},
    ffi,
};
use std::{
    ffi::CString,
    ops::{Deref, DerefMut, Range},
    ptr::NonNull,
};

/// Struct for holding the result of [`RaylibHandle::load_random_sequence`].
/// This is a thin wrapper for an array of [`i32`]. The reason it exists is because Raylib expects you
/// to unload the sequence it creates manually, and this struct does it for you.
pub struct RandomSequence(NonNull<[i32]>);

impl Deref for RandomSequence {
    type Target = [i32];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl DerefMut for RandomSequence {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

impl Drop for RandomSequence {
    fn drop(&mut self) {
        unsafe { ffi::UnloadRandomSequence(self.0.as_ptr().cast()) }
    }
}

impl IntoIterator for RandomSequence {
    type Item = i32;
    type IntoIter = RandSeqIterator;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        RandSeqIterator(self, 0)
    }
}
/// Iterator over [`RandomSequence`] elements.
pub struct RandSeqIterator(RandomSequence, usize);

impl Iterator for RandSeqIterator {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        self.1
            .checked_add(1)
            .filter(|n| *n < self.0.len())
            .map(|n| self.0[std::mem::replace(&mut self.1, n)])
    }
}

impl ExactSizeIterator for RandSeqIterator {
    fn len(&self) -> usize {
        self.0.len() - self.1
    }
}

impl std::iter::FusedIterator for RandSeqIterator {}

/// Open URL with default system browser (if available)
///
/// # Example
/// ```ignore
/// use raylib::*;
/// fn main() {
///     open_url("https://google.com");
/// }
/// ```
///
/// # Panics
///
/// This function will panic if `url` contains an internal 0 byte
pub fn open_url(url: &str) {
    let s = CString::new(url).expect("url should not contain an internal 0 byte");
    unsafe {
        ffi::OpenURL(s.as_ptr());
    }
}

impl RaylibHandle {
    /// Load random values sequence, no values repeated, min and max included
    ///
    /// # Panics
    ///
    /// This method will panic if [`ffi::LoadRandomSequence`] errors or if `count` is greater than [`usize::MAX`].
    // This function should definitely return `Option<RandomSequence>` and `num` should be a `RangeInclusive<i32>`
    #[must_use]
    pub fn load_random_sequence(&self, num: Range<i32>, count: u32) -> RandomSequence {
        let ptr = unsafe { ffi::LoadRandomSequence(count, num.start, num.end) };
        RandomSequence(NonNull::slice_from_raw_parts(
            NonNull::new(ptr).expect("error occurred in ffi::LoadRandomSequence"),
            count
                .try_into()
                .expect("count should not exceed usize::MAX"),
        ))
    }

    /// Load pixels from the screen into a CPU image
    #[inline]
    #[must_use]
    pub fn load_image_from_screen(&self, _: &RaylibThread) -> Image {
        unsafe { Image(ffi::LoadImageFromScreen()) }
    }

    /// Takes a screenshot of current screen (saved a .png)
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte
    pub fn take_screenshot(&mut self, _: &RaylibThread, filename: &str) {
        let c_filename =
            CString::new(filename).expect("filename should not contain an internal 0 byte");
        unsafe {
            ffi::TakeScreenshot(c_filename.as_ptr());
        }
    }

    /// Returns a random value between min and max (both included)
    /// ```ignore
    /// use raylib::*;
    /// fn main() {
    ///     let (mut rl, thread) = ...;
    ///     let r = rl.get_random_value(0, 10);
    ///     println!("random value: {}", r);
    /// }
    #[inline]
    #[must_use]
    pub fn get_random_value<T: From<i32>>(&self, num: Range<i32>) -> T {
        unsafe { (ffi::GetRandomValue(num.start, num.end) as i32).into() }
    }

    /// Set the seed for random number generation
    pub fn set_random_seed(&mut self, seed: u32) {
        unsafe {
            ffi::SetRandomSeed(seed);
        }
    }
}

/// Lossy conversion to an [`f32`]
pub trait AsF32: Copy {
    /// Convert `self` to [`f32`] using `as`.
    fn as_f32(self) -> f32;
}

macro_rules! impl_as_f32 {
    ($($Ty:ty),* $(,)?) => {$(
        impl AsF32 for $Ty {
            #[allow(
                clippy::cast_lossless,
                clippy::cast_precision_loss,
                reason = "AsF32 is meant to replicate the `as` keyword"
            )]
            fn as_f32(self) -> f32 {
                self as f32
            }
        }
    )*};
}

impl_as_f32!(u8, u16, u32, i8, i16, i32, f32);
