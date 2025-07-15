//! Data manipulation functions. Compress and Decompress with DEFLATE
use std::{
    alloc::Layout,
    ffi::{CString, c_char},
    marker::PhantomData,
    mem::MaybeUninit,
    num::{NonZeroU32, NonZeroUsize},
    ops::{Deref, DerefMut},
    path::Path,
    ptr::NonNull,
};

use crate::{
    error::{AllocationError, CompressionError},
    ffi,
};

/// Calculate the number of bytes needed to allocate `layout`.
#[inline]
fn allocation_layout_size(layout: Layout) -> Result<NonZeroU32, AllocationError> {
    let bytes = layout
        .size()
        .try_into()
        .ok()
        .ok_or(AllocationError::IntoUIntFailed)?;

    NonZeroU32::new(bytes).ok_or(AllocationError::ZeroBytes)
}

/// Calculate the number of bytes needed to allocate `[T; count]`.
#[inline]
fn allocation_array_size<T>(count: usize) -> Result<NonZeroU32, AllocationError> {
    allocation_layout_size(Layout::array::<T>(count).map_err(|_| AllocationError::IntoUIntFailed)?)
}

/// Calculate the number of bytes needed to allocate a copy of `val`
#[inline]
fn allocation_val_size<T: ?Sized>(val: &T) -> Result<NonZeroU32, AllocationError> {
    allocation_layout_size(Layout::for_value(val))
}

mod rl_managed {
    use super::*;

    /// Raylib-managed [`NonNull`].
    #[repr(transparent)]
    #[derive(Debug)]
    pub struct RlManaged<T: ?Sized>(/* unsafe */ NonNull<T>);

    impl<T: ?Sized> std::ops::Deref for RlManaged<T> {
        type Target = NonNull<T>;

        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T: ?Sized> std::ops::DerefMut for RlManaged<T> {
        #[inline]
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl<T: ?Sized> RlManaged<T> {
        /// Mark memory as Raylib-managed.
        ///
        /// # Safety
        ///
        /// `data` must be unique, not dangling, and allocated with `RL_ALLOC`/[`ffi::MemAlloc`] or `RL_REALLOC`/[`ffi::MemRealloc`].
        #[inline]
        pub(crate) const unsafe fn new(data: NonNull<T>) -> Self {
            Self(data)
        }
    }

    impl<T> RlManaged<[T]> {
        /// Create a Raylib-managed slice from a thin pointer and a length.
        ///
        /// The `len` argument is the number of **elements**, not the number of bytes.
        ///
        /// This function is safe, but dereferencing the return value is unsafe.
        /// See the documentation of [`std::slice::from_raw_parts`] for slice safety requirements.
        #[inline]
        pub(crate) const fn slice_from_raw_parts(data: RlManaged<T>, len: usize) -> Self {
            Self(NonNull::slice_from_raw_parts(data.0, len))
        }

        /// Access the pointer of this allocation.
        ///
        /// **NOTE:** This method eliminates the slice metadata, converting it from a wide pointer
        /// to a thin pointer. This is intentional. Raylib does not use wide pointers, so the thin
        /// pointer will be more applicable.
        /// (and is what was returned by the allocator in the first place, making it safe to free)
        #[inline]
        pub const fn as_ptr(self) -> *mut T {
            self.0.cast().as_ptr()
        }
    }

    /// Allocate `size` bytes of memory aligned to `T` using [`ffi::MemAlloc`].
    ///
    /// Returns [`None`] if [`ffi::MemAlloc`] returned null.
    #[inline]
    pub fn mem_alloc<T>(size: NonZeroU32) -> Option<RlManaged<MaybeUninit<T>>> {
        // SAFETY: `size` is not zero.
        let ptr = unsafe { ffi::MemAlloc(size.get()) }.cast();
        NonNull::new(ptr).map(RlManaged)
    }

    impl<T: ?Sized> RlManaged<T> {
        /// Reallocate `ptr` to have `size` bytes of memory aligned to `U` using [`ffi::MemRealloc`].
        ///
        /// Any elements that were initialized prior to calling will still be initialized after reallocating.
        /// Any elements that were not previously in the allocation are uninitialized.
        ///
        /// Returns the original memory block if [`ffi::MemRealloc`] returned null.
        #[inline]
        pub fn mem_realloc<U>(self, size: NonZeroU32) -> Result<RlManaged<MaybeUninit<U>>, Self> {
            // SAFETY: `self` is non-null and is Raylib-allocated, and `size` is not zero.
            let new_ptr = unsafe { ffi::MemRealloc(self.0.as_ptr().cast(), size.get()) }.cast();
            NonNull::new(new_ptr).map(RlManaged).ok_or_else(|| self)
        }

        /// Free `ptr` using [`ffi::MemFree`].
        #[inline]
        pub fn mem_free(self) {
            // SAFETY: `self` is non-null and Raylib-allocated.
            unsafe {
                ffi::MemFree(self.0.as_ptr().cast());
            }
        }
    }
}
pub use rl_managed::*;

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
/// - `buf` must point to [valid](https://doc.rust-lang.org/std/ptr/index.html#safety), intialized data.
/// - `buf` must be [convertible to a reference](std::ptr#pointer-to-reference-conversion).
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
    buf: RlManaged<T>,
    /// Tell the compiler that this instance logically owns a `T`.
    _marker: PhantomData<T>,
}

impl<T: ?Sized> Drop for DataBuf<T> {
    #[inline]
    fn drop(&mut self) {
        let mut ptr = MaybeUninit::uninit();
        // SAFETY: Both `self.buf` and `ptr` are non-null and valid for 1 element.
        unsafe {
            std::ptr::copy_nonoverlapping(std::ptr::from_ref(&self.buf), ptr.as_mut_ptr(), 1)
        };
        // SAFETY: Just written to with a valid value
        let ptr = unsafe { ptr.assume_init() };
        // SAFETY: `RlManaged` is guaranteed to be unique, non-null, valid, and not dangling
        unsafe {
            ptr.drop_in_place();
        }
        ptr.mem_free(); // `ptr` will not be observed after free
    }
}

impl<T: ?Sized> Deref for DataBuf<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: `buf` is non-null, unique, & valid, and guaranteed convertible to a reference.
        unsafe { self.buf.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for DataBuf<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: `buf` is non-null, unique, & valid, and guaranteed convertible to a reference.
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

impl<T> DataBuf<[MaybeUninit<T>]> {
    /// Mark that the data pointed to by `self` is initialized.
    ///
    /// # Safety
    ///
    /// The data pointed to by `self` must actually be initialized.
    #[inline]
    pub const unsafe fn assume_init(self) -> DataBuf<[T]> {
        // SAFETY: `DataBuf<[MaybeUninit<T>]>` and `DataBuf<[T]>` have the same layout
        unsafe { std::mem::transmute::<DataBuf<[MaybeUninit<T>]>, DataBuf<[T]>>(self) }
    }
}

impl<T: ?Sized> DataBuf<T> {
    /// Wrap an already allocated, non-null, Raylib-managed pointer in a [`DataBuf`].
    #[inline]
    pub(crate) fn from_rlmanaged(buf: RlManaged<T>) -> Self {
        Self {
            buf,
            _marker: PhantomData,
        }
    }

    /// Wrap an already allocated, non-null pointer in a [`DataBuf`].
    ///
    /// # Safety
    ///
    /// See the [`DataBuf`] safety requirements.
    #[inline]
    pub(crate) unsafe fn from_nonnull(data: NonNull<T>) -> Self {
        // SAFETY: Caller must ensure `data` is Raylib-managed.
        let buf = unsafe { RlManaged::new(data) };
        Self::from_rlmanaged(buf)
    }

    /// Wrap an already allocated pointer in a [`DataBuf`].
    /// Returns [`None`] if `buf` is null.
    ///
    /// # Safety
    ///
    /// See the [`DataBuf`] safety requirements.
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: *mut T) -> Option<Self> {
        NonNull::new(ptr).map(|buf|
            // SAFETY: Caller must uphold safety contract.
            unsafe { Self::from_nonnull(buf) })
    }

    /// Extract the pointer without freeing it, for the purpose of transferring ownership.
    ///
    /// **WARNING:** The returned pointer must be unloaded manually to avoid a memory leak.
    #[inline]
    pub const fn leak(self) -> RlManaged<T> {
        let mut buf = MaybeUninit::uninit();
        // SAFETY: Both `self.buf` and `ptr` are non-null and valid for 1 element.
        unsafe {
            std::ptr::copy_nonoverlapping(std::ptr::from_ref(&self.buf), buf.as_mut_ptr(), 1);
        }
        // SAFETY: Just written to with a valid value
        let buf = unsafe { buf.assume_init() };
        std::mem::forget(self); // Prevent `self` from causing double-free
        buf
    }
}

impl<T> DataBuf<[T]> {
    /// Wrap an already allocated pointer in a [`DataBuf`].
    ///
    /// # Safety
    ///
    /// **In addition** to the [`DataBuf`] and [`from_raw_parts_mut`](std::slice::from_raw_parts_mut)
    /// safety requirements, this function also requires:
    /// - `buf` must point to an array of as many valid, initialized elements as defined by `count`.
    ///
    /// # Panics
    ///
    /// This method may panic if `buf` is both non-null and unaligned.
    #[inline]
    pub(crate) unsafe fn slice_from_nonnull(buf: NonNull<T>, len: NonZeroUsize) -> Self {
        assert!(buf.is_aligned(), "DataBuf should be aligned");
        // SAFETY: Caller must uphold `from_raw_parts_mut` safety contract
        let slice = unsafe { std::slice::from_raw_parts_mut(buf.as_ptr(), len.get()) };
        // SAFETY: A mutable reference cannot be null.
        let buf = unsafe { NonNull::new_unchecked(slice) };
        // SAFETY: Calller must uphold `DataBuf` safety contract
        unsafe { Self::from_nonnull(buf) }
    }

    /// Wrap an already allocated pointer in a [`DataBuf`].
    /// Returns [`None`] if `buf` is null.
    ///
    /// Takes `count` as a [`MaybeUninit<i32>`] for convenience, as most Raylib functions returning an array
    /// buffer provide the length of the buffer as an [`i32`] out param.
    ///
    /// # Safety
    ///
    /// **In addition** to the [`DataBuf`] and [`from_raw_parts_mut`](std::slice::from_raw_parts_mut) safety requirements,
    /// this function also requires:
    /// - `count` must be initialized if `buf` is non-null.
    /// - `buf` must point to an array of as many valid, initialized elements as defined by `count`.
    ///
    /// # Panics
    ///
    /// This method may panic if `count` is less than 1 while `buf` is non-null.
    #[inline]
    pub(crate) unsafe fn slice_from_raw(ptr: *mut T, count: MaybeUninit<i32>) -> Option<Self> {
        NonNull::new(ptr).map(|buf| {
            // SAFETY: Caller must ensure `count` is initialized if `buf` is non-null.
            let len = unsafe { count.assume_init() }
                .try_into()
                .ok()
                .and_then(NonZeroUsize::new)
                .unwrap();
            // SAFETY: Caller must uphold `DataBuf` and `slice` safety contracts
            unsafe { Self::slice_from_nonnull(buf, len) }
        })
    }

    /// Allocate new memory managed by Raylib.
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
    #[inline]
    pub fn alloc(count: usize) -> Result<DataBuf<[MaybeUninit<T>]>, AllocationError> {
        let bytes = allocation_array_size::<T>(count)?;
        let buf = mem_alloc::<T>(bytes).ok_or(AllocationError::NullAlloc)?;
        Ok(DataBuf {
            buf: RlManaged::slice_from_raw_parts(buf, count),
            _marker: PhantomData,
        })
    }

    /// Allocate memory managed by Raylib and initialize by copying.
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
        let bytes = allocation_val_size(src)?;
        let buf = mem_alloc::<T>(bytes).ok_or(AllocationError::NullAlloc)?;
        let buf = RlManaged::slice_from_raw_parts(buf, src.len());
        // SAFETY: `&[T]` and `&[MaybeUninit<T>]` have the same layout.
        let uninit_src = unsafe { std::mem::transmute::<&[T], &[MaybeUninit<T>]>(src) };
        let mut buf = DataBuf::from_rlmanaged(buf);
        buf.copy_from_slice(uninit_src);
        // SAFETY: Valid elements have just been copied into `self` so it is initialized.
        Ok(unsafe { buf.assume_init() })
    }

    /// Reallocate memory already managed by Raylib.
    ///
    /// # Panics
    ///
    /// This method may panic in debug if the pointer returned by [`ffi::MemAlloc`] is unaligned.
    pub fn realloc(self, new_count: usize) -> Result<DataBuf<[MaybeUninit<T>]>, AllocationError> {
        let bytes = allocation_array_size::<T>(new_count)?;
        let old_ptr = self.leak();
        let new_buf = old_ptr.mem_realloc(bytes).map_err(|old_ptr| {
            old_ptr.mem_free();
            AllocationError::NullAlloc
        })?;
        let new_buf = RlManaged::slice_from_raw_parts(new_buf, new_count);
        Ok(DataBuf::from_rlmanaged(new_buf))
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
