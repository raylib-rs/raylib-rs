//! Data manipulation functions. Compress and Decompress with DEFLATE
use std::{
    alloc::Layout,
    ffi::{CString, c_char},
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    os::raw::c_void,
    path::Path,
    ptr::NonNull,
};

use crate::{
    error::{AllocationError, CompressionError},
    ffi,
};

/// A wrapper acting as an owned buffer for Raylib-allocated memory.
/// Automatically releases the memory with [`ffi::MemFree()`] when dropped.
///
/// # Example
/// ```
/// use raylib::prelude::*;
/// let buf: DataBuf<[u8]> = compress_data(b"11111").unwrap();
/// // Use this how you used to use the return of `compress_data()`.
/// // It will live until `buf` goes out of scope or gets dropped.
/// let data: &[u8] = buf.as_ref();
/// let expected: &[u8] = &[1, 5, 0, 250, 255, 49, 49, 49, 49, 49];
/// assert_eq!(data, expected);
/// ```
///
/// # Safety
///
/// - `buf` must not be dangling.
/// - `buf` must be safe to dereference.
/// - `buf` must be a **unique, owned** pointer (not to static or local memory, and the memory must not be
///   accessible through any pointers/references not derived from the returned [`DataBuf`]).
/// - `buf` must point to valid, intialized data.
/// - `buf` must have been created with `RL_MALLOC`/[`ffi::MemAlloc`] or `RL_REALLOC`/[`ffi::MemRealloc`].
///
/// This structure is only intended for use with pointers given by Raylib with the expectation that you
/// would manually deallocate them with [`ffi::MemFree`]. DO NOT use this structure to hold arbitrary
/// or un-owned pointers.
///
/// If the pointer is expected to be conditionally deallocated by Raylib,
/// (i.e. conditionally passing the buffer to a Raylib function that will certainly deallocatate it)
/// use [`DataBuf::leak`] to prevent [`DataBuf::drop`] from causing a double-free.
#[derive(Debug)]
#[repr(transparent)]
pub struct DataBuf<T: ?Sized> {
    buf: NonNull<T>,
    /// Tell the compiler that this instance logically owns a `T`.
    _marker: PhantomData<T>,
}

impl<T: ?Sized> Drop for DataBuf<T> {
    fn drop(&mut self) {
        // The `T` in the `DataBuf` is dropped by the compiler before the destructor is run

        let ptr = self.buf.as_ptr().cast::<c_void>();

        // SAFETY: Guaranteed by constructor.
        // Caller must have ensured that `buf` is a non-dangling pointer to Raylib-managed memory.
        unsafe {
            ffi::MemFree(ptr);
        }
    }
}

impl<T: ?Sized> Deref for DataBuf<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: Guaranteed by constructor.
        // Caller must have ensured that `buf` is non-null, valid, and unique.
        unsafe { self.buf.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for DataBuf<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Guaranteed by constructor.
        // Caller must have ensured that `buf` is non-null, valid, and unique.
        unsafe { self.buf.as_mut() }
    }
}

impl<T: ?Sized> AsRef<T> for DataBuf<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<T: ?Sized> AsMut<T> for DataBuf<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}

impl<T> DataBuf<MaybeUninit<T>> {
    /// Mark that the data pointed to by `self` is initialized.
    ///
    /// # Safety
    ///
    /// The data pointed to by `self` must actually be initialized.
    pub const unsafe fn assume_init(self) -> DataBuf<T> {
        // SAFETY: `T` and `MaybeUninit<T>` have the same layout.
        unsafe { std::mem::transmute::<DataBuf<MaybeUninit<T>>, DataBuf<T>>(self) }
    }
}

impl<T> DataBuf<[MaybeUninit<T>]> {
    /// Mark that the data pointed to by `self` is initialized.
    ///
    /// # Safety
    ///
    /// The data pointed to by `self` must actually be initialized.
    pub const unsafe fn assume_init(self) -> DataBuf<[T]> {
        // SAFETY: `[T]` and `[MaybeUninit<T>]` have the same layout.
        unsafe { std::mem::transmute::<DataBuf<[MaybeUninit<T>]>, DataBuf<[T]>>(self) }
    }
}

impl<T> DataBuf<T> {
    /// Wrap an already allocated pointer in a [`DataBuf`].
    /// Returns [`None`] if `buf` is null.
    ///
    /// # Safety
    ///
    /// See the [`DataBuf`] safety requirements.
    ///
    /// # Panics
    ///
    /// This method may panic if any of the following are true while `buf` is non-null:
    /// - `buf` is unaligned
    /// - total bytes exceed [`isize::MAX`]
    pub(crate) unsafe fn from_raw(buf: *mut T) -> Option<Self> {
        NonNull::new(buf).map(|buf| {
            // SAFETY: Caller must ensure `count` is initialized if `buf` is non-null.
            assert!(buf.is_aligned(), "DataBuf should be aligned");
            assert!(
                std::mem::size_of::<T>() <= (isize::MAX as usize),
                "total size of DataBuf should not exceed `isize::MAX`"
            );

            Self {
                buf,
                _marker: PhantomData,
            }
        })
    }

    /// Extract the pointer without freeing it, for the purpose of transferring ownership.
    #[inline]
    pub(crate) const fn leak(self) -> NonNull<T> {
        let buf = self.buf.cast::<T>();
        std::mem::forget(self);
        buf
    }

    /// Allocate new memory managed by Raylib.
    ///
    /// # Errors
    ///
    /// - [`InvalidLayout`](AllocationError::InvalidLayout):
    ///   [`Layout::array::<T>(count.get())`](Layout::array) resulted in an error.
    ///
    /// - [`ExceedsUIntMax`](AllocationError::ExceedsUIntMax):
    ///   The size of `[T; count]` in bytes exceeds [`u32::MAX`].
    ///
    /// - [`ExceedsCapacity`](AllocationError::ExceedsCapacity):
    ///   [`ffi::MemAlloc`] returned null.
    ///
    /// # Panics
    ///
    /// This method may panic in debug if the pointer returned by [`ffi::MemAlloc`] is unaligned.
    pub fn alloc() -> Result<DataBuf<MaybeUninit<T>>, AllocationError> {
        let bytes = Layout::array::<T>(1)
            .map_err(AllocationError::InvalidLayout)?
            .size()
            .try_into()
            .map_err(|_| AllocationError::ExceedsUIntMax)?;
        // SAFETY: `bytes` is guaranteed to be non-zero.
        let ptr = unsafe { ffi::MemAlloc(bytes) }.cast::<MaybeUninit<T>>();
        let buf = NonNull::new(ptr).ok_or(AllocationError::ExceedsCapacity)?;
        debug_assert!(
            buf.is_aligned(),
            "allocated buffer should always be aligned"
        );
        Ok(DataBuf {
            buf,
            _marker: PhantomData,
        })
    }
}

impl<T> DataBuf<[T]> {
    /// Wrap an already allocated pointer in a [`DataBuf`].
    /// Returns [`None`] if `buf` is null.
    ///
    /// Takes `count` as a [`MaybeUninit<i32>`] for convenience, as most Raylib functions returning an array
    /// buffer provide the length of the buffer as an [`i32`] out param.
    ///
    /// # Safety
    ///
    /// **In addition** to the [`DataBuf`] safety requirements, this function also requires:
    /// - `count` must be initialized if `buf` is non-null.
    /// - `buf` must point to an array of as many valid, initialized elements as defined by `count`.
    ///
    /// # Panics
    ///
    /// This method may panic if any of the following are true while `buf` is non-null:
    /// - `count` is less than 1
    /// - `buf` is unaligned
    /// - total bytes exceed [`isize::MAX`]
    pub(crate) unsafe fn slice_from_raw(buf: *mut T, count: MaybeUninit<i32>) -> Option<Self> {
        NonNull::new(buf).map(|buf| {
            // SAFETY: Caller must ensure `count` is initialized if `buf` is non-null.
            let len = unsafe { count.assume_init() }.try_into().unwrap();
            assert!(len >= 1, "non-null data should be at least 1 byte");
            assert!(buf.is_aligned(), "DataBuf should be aligned");
            assert!(
                std::mem::size_of::<T>()
                    .checked_mul(len)
                    .is_some_and(|total_size| total_size <= (isize::MAX as usize)),
                "total size of DataBuf should not exceed `isize::MAX`"
            );

            Self {
                buf: NonNull::slice_from_raw_parts(buf, len),
                _marker: PhantomData,
            }
        })
    }

    /// Extract the pointer without freeing it, for the purpose of transferring ownership.
    ///
    /// **NOTE:** This method eliminates the slice metadata, converting it from a wide pointer
    /// to a thin pointer. This is intentional. Raylib does not use wide pointers, so the thin
    /// pointer will be more applicable.
    /// (and is what was returned by the allocator in the first place, making it safe to free)
    #[inline]
    pub(crate) const fn leak(self) -> NonNull<T> {
        let buf = self.buf.cast::<T>();
        std::mem::forget(self);
        buf
    }

    /// Allocate new memory managed by Raylib.
    ///
    /// # Errors
    ///
    /// - [`InvalidLayout`](AllocationError::InvalidLayout):
    ///   [`Layout::array::<T>(count.get())`](Layout::array) resulted in an error.
    ///
    /// - [`ExceedsUIntMax`](AllocationError::ExceedsUIntMax):
    ///   The size of `[T; count]` in bytes exceeds [`u32::MAX`].
    ///
    /// - [`ExceedsCapacity`](AllocationError::ExceedsCapacity):
    ///   [`ffi::MemAlloc`] returned null.
    ///
    /// # Panics
    ///
    /// This method may panic in debug if the pointer returned by [`ffi::MemAlloc`] is unaligned.
    ///
    /// # Example
    /// ```
    /// # use raylib::prelude::DataBuf;
    /// let mut data_buf = DataBuf::<[i32]>::alloc(5).unwrap();
    /// data_buf[0].write(4);
    /// data_buf[1].write(8);
    /// data_buf[2].write(-23);
    /// data_buf[3].write(9);
    /// data_buf[4].write(0);
    /// // SAFETY: Just initialized all elements
    /// let data_buf = unsafe { data_buf.assume_init() };
    /// assert_eq!(data_buf.as_ref(), &[4, 8, -23, 9, 0]);
    /// ```
    /// (See also: [`DataBuf::alloc_from_copy`])
    pub fn alloc(count: usize) -> Result<DataBuf<[MaybeUninit<T>]>, AllocationError> {
        let bytes = Layout::array::<T>(count)
            .map_err(AllocationError::InvalidLayout)?
            .size()
            .try_into()
            .map_err(|_| AllocationError::ExceedsUIntMax)?;
        // SAFETY: `bytes` is guaranteed to be non-zero.
        let ptr = unsafe { ffi::MemAlloc(bytes) }.cast::<MaybeUninit<T>>();
        let buf = NonNull::new(ptr).ok_or(AllocationError::ExceedsCapacity)?;
        debug_assert!(
            buf.is_aligned(),
            "allocated buffer should always be aligned"
        );
        Ok(DataBuf {
            buf: NonNull::slice_from_raw_parts(buf, count),
            _marker: PhantomData,
        })
    }

    /// Allocate memory managed by Raylib and initialize by copying.
    ///
    /// # Errors
    ///
    /// - [`ExceedsUIntMax`](AllocationError::ExceedsUIntMax):
    ///   The size of `[T; count]` in bytes exceeds [`u32::MAX`].
    ///
    /// - [`ExceedsCapacity`](AllocationError::ExceedsCapacity):
    ///   [`ffi::MemAlloc`] returned null.
    ///
    /// # Panics
    ///
    /// This method may panic in debug if the pointer returned by [`ffi::MemAlloc`] is unaligned.
    ///
    /// # Example
    /// ```
    /// # use raylib::prelude::DataBuf;
    /// let src = [4, 8, -23, 9, 0];
    /// let mut data_buf = DataBuf::<[i32]>::alloc_from_copy(&src).unwrap();
    /// assert_eq!(data_buf.as_ref(), &src);
    /// ```
    pub fn alloc_from_copy(src: &[T]) -> Result<Self, AllocationError>
    where
        T: Copy,
    {
        let bytes = Layout::for_value(src)
            .size()
            .try_into()
            .map_err(|_| AllocationError::ExceedsUIntMax)?;
        // SAFETY: `bytes` is guaranteed to be non-zero.
        let ptr = unsafe { ffi::MemAlloc(bytes) }.cast::<MaybeUninit<T>>();
        let buf = NonNull::new(ptr).ok_or(AllocationError::ExceedsCapacity)?;
        debug_assert!(
            buf.is_aligned(),
            "allocated buffer should always be aligned"
        );
        let mut buf = NonNull::slice_from_raw_parts(buf, src.len());
        // SAFETY: `&[T]` and `&[MaybeUninit<T>]` have the same layout
        let uninit_src = unsafe { std::mem::transmute::<&[T], &[MaybeUninit<T>]>(src) };
        // SAFETY: `MemAlloc` has not returned NULL and `MaybeUninit` removes the requirement for
        // the data to be initialized, so `buf` is convertible to a reference.
        unsafe { buf.as_mut() }.copy_from_slice(uninit_src);
        // SAFETY: Valid elements have just been copied into `self` so it is initialized,
        // and `NonNull<[MaybeUninit<T>]>` and `NonNull<[T]>` have the same layout
        let buf = unsafe { std::mem::transmute::<NonNull<[MaybeUninit<T>]>, NonNull<[T]>>(buf) };
        Ok(DataBuf {
            buf,
            _marker: PhantomData,
        })
    }

    /// Reallocate memory already managed by Raylib.
    ///
    /// # Errors
    ///
    /// - [`InvalidLayout`](AllocationError::InvalidLayout):
    ///   [`Layout::array::<T>(count.get())`](Layout::array) resulted in an error.
    ///
    /// - [`ExceedsUIntMax`](AllocationError::ExceedsUIntMax):
    ///   The size of `[T; count]` in bytes exceeds [`u32::MAX`].
    ///
    /// - [`ExceedsCapacity`](AllocationError::ExceedsCapacity):
    ///   [`ffi::MemAlloc`] returned null.
    ///
    /// # Panics
    ///
    /// This method may panic in debug if the pointer returned by [`ffi::MemAlloc`] is unaligned.
    pub fn realloc(self, new_count: usize) -> Result<DataBuf<[MaybeUninit<T>]>, AllocationError> {
        let bytes = Layout::array::<T>(new_count)
            .map_err(AllocationError::InvalidLayout)?
            .size()
            .try_into()
            .map_err(|_| AllocationError::ExceedsUIntMax)?;
        let old_ptr = self.leak().cast::<c_void>().as_ptr();
        // SAFETY: `bytes` is guaranteed to be non-zero and `self.buf` is guaranteed to be
        // both non-null and raylib-managed.
        let new_ptr = unsafe { ffi::MemRealloc(old_ptr, bytes) }.cast::<MaybeUninit<T>>();
        let buf = NonNull::new(new_ptr).ok_or(AllocationError::ExceedsCapacity)?;
        debug_assert!(
            buf.is_aligned(),
            "allocated buffer should always be aligned"
        );

        Ok(DataBuf {
            buf: NonNull::slice_from_raw_parts(buf, new_count),
            _marker: PhantomData,
        })
    }
}

/// Compress data (DEFLATE algorithm)
/// ```rust
/// use raylib::prelude::*;
/// let data = compress_data(b"11111").unwrap();
/// let expected: &[u8] = &[1, 5, 0, 250, 255, 49, 49, 49, 49, 49];
/// assert_eq!(data.as_ref(), expected);
/// ```
pub fn compress_data(data: &[u8]) -> Result<DataBuf<[u8]>, CompressionError> {
    let mut out_length = MaybeUninit::uninit();
    // CompressData doesn't actually modify the data, but the header is wrong
    let buffer = {
        unsafe {
            ffi::CompressData(
                data.as_ptr() as *mut _,
                data.len() as i32,
                out_length.as_mut_ptr(),
            )
        }
    };
    // SAFETY: `CompressData` returns a unique, owned pointer that is safe to dereference for
    // `out_length` valid, initialized elements if `buffer` is not null. It also guarantees
    // `out_length` is initialized if `buffer` is non-null.
    unsafe { DataBuf::slice_from_raw(buffer, out_length) }
        .ok_or_else(|| CompressionError::CompressionFailed)
}

/// Decompress data (DEFLATE algorithm)
/// ```rust
/// use raylib::prelude::*;
/// let input: &[u8] = &[1, 5, 0, 250, 255, 49, 49, 49, 49, 49];
/// let expected: &[u8] = b"11111";
/// let data = decompress_data(input).unwrap();
/// assert_eq!(data.as_ref(), expected);
/// ```
pub fn decompress_data(data: &[u8]) -> Result<DataBuf<[u8]>, CompressionError> {
    #[cfg(debug_assertions)]
    println!("{:?}", data.len());

    let mut out_length = MaybeUninit::uninit();
    // CompressData doesn't actually modify the data, but the header is wrong
    let buffer = {
        unsafe {
            ffi::DecompressData(
                data.as_ptr() as *mut _,
                data.len() as i32,
                out_length.as_mut_ptr(),
            )
        }
    };
    // SAFETY: `DecompressData` returns a unique, owned pointer that is safe to dereference for
    // `out_length` valid, initialized elements if `buffer` is not null. It also guarantees
    // `out_length` is initialized if `buffer` is non-null.
    unsafe { DataBuf::slice_from_raw(buffer, out_length) }
        .ok_or_else(|| CompressionError::CompressionFailed)
}

#[cfg(unix)]
fn path_to_bytes<P: AsRef<Path>>(path: P) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    path.as_ref().as_os_str().as_bytes().to_vec()
}

#[cfg(not(unix))]
fn path_to_bytes<P: AsRef<Path>>(path: P) -> Vec<u8> {
    path.as_ref().to_string_lossy().to_string().into_bytes()
}

/// Export data to code (.h), returns true on success
pub fn export_data_as_code(data: &[u8], file_name: impl AsRef<Path>) -> bool {
    let c_str = CString::new(path_to_bytes(file_name)).unwrap();

    unsafe { ffi::ExportDataAsCode(data.as_ptr(), data.len() as i32, c_str.as_ptr()) }
}

/// Encode data to Base64 string
pub fn encode_data_base64(data: &[u8]) -> Vec<c_char> {
    let mut output_size = 0;
    let bytes =
        unsafe { ffi::EncodeDataBase64(data.as_ptr(), data.len() as i32, &mut output_size) };

    let s = unsafe { std::slice::from_raw_parts(bytes, output_size as usize) };
    if s.contains(&0) {
        // Work around a bug in Rust's from_raw_parts function
        let mut keep = true;
        let b: Vec<c_char> = s
            .iter()
            .filter(|f| {
                if **f == 0 {
                    keep = false;
                }
                keep
            })
            .map(|f| *f)
            .collect();
        b
    } else {
        s.to_vec()
    }
}

/// Decode Base64 data
pub fn decode_data_base64(data: &[u8]) -> Vec<u8> {
    let mut output_size = 0;

    let bytes = unsafe { ffi::DecodeDataBase64(data.as_ptr(), &mut output_size) };

    let s = unsafe { std::slice::from_raw_parts(bytes, output_size as usize) };
    if s.contains(&0) {
        // Work around a bug in Rust's from_raw_parts function
        let mut keep = true;
        let b: Vec<u8> = s
            .iter()
            .filter(|f| {
                if **f == 0 {
                    keep = false;
                }
                keep
            })
            .map(|f| *f)
            .collect();
        b
    } else {
        s.to_vec()
    }
}
