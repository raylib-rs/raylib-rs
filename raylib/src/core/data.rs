//! Data manipulation functions. Compress and Decompress with DEFLATE
use std::{
    alloc::Layout, ffi::{c_char, CString}, ops::{Deref, DerefMut}, path::Path, ptr::NonNull
};

use crate::{
    error::{error, Error},
    ffi,
};

/// A wrapper acting as owned buffer for Raylib-allocated memory.
/// Automatically releases the memory with [`ffi::MemFree()`] when dropped.
///
/// Dereference or call `.as_ref()`/`.as_mut()` to access the memory as a `&[u8]` or `&mut [u8]` respectively.
///
/// # Example
/// ```
/// use raylib::prelude::*;
/// let buf: DataBuf = compress_data(b"11111").unwrap();
/// // Use this how you used to use the return of `compress_data()`.
/// // It will live until `buf` goes out of scope or gets dropped.
/// let data: &[u8] = buf.as_ref();
/// let expected: &[u8] = &[1, 5, 0, 250, 255, 49, 49, 49, 49, 49];
/// assert_eq!(data, expected);
/// ```
pub struct DataBuf<T: Copy> {
    buf: NonNull<T>,
    len: usize,
}
impl<T: Copy + std::fmt::Debug> std::fmt::Debug for DataBuf<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataBuf")
            .field("buf", &self.buf)
            .field("len", &self.len)
            .finish()
    }
}
impl<T: Copy> Drop for DataBuf<T> {
    fn drop(&mut self) {
        unsafe {
            ffi::MemFree(self.buf.as_ptr().cast());
        }
    }
}
impl<T: Copy> Deref for DataBuf<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        // This is safe because DataBuf contents are checked everywhere `buf` can be set.
        unsafe { &*std::ptr::slice_from_raw_parts(self.buf.as_ptr(), self.len) }
    }
}
impl<T: Copy> DerefMut for DataBuf<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // This is safe because DataBuf contents are checked everywhere `buf` can be set.
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(self.buf.as_ptr(), self.len) }
    }
}
impl<T: Copy> AsRef<[T]> for DataBuf<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.deref()
    }
}
impl<T: Copy> AsMut<[T]> for DataBuf<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.deref_mut()
    }
}
impl<T: Copy> DataBuf<T> {
    /// Wrap an already allocated pointer in a `DataBuf`
    pub(crate) fn new(buf: *mut T, count: i32) -> Option<Self> {
        NonNull::new(buf).map(|buf| {
            // Ensure DataBuf can always be dereferenced as a slice.
            assert!(count >= 1, "non-null data should be at least 1 byte");
            assert!(buf.is_aligned(), "DataBuf should be aligned");
            assert!(std::mem::size_of::<T>()
                .checked_mul(count as usize)
                .is_some_and(|total_size| total_size <= (isize::MAX as usize)),
                "total size of DataBuf should not exceed `isize::MAX`");

            Self { buf, len: count as usize }
        })
    }

    /// Allocate new memory managed by Raylib
    pub fn alloc(count: i32) -> Result<Self, Error> {
        if count >= 1 {
            let count = count as usize;
            match Layout::array::<T>(count) {
                Err(_e) => Err(error!("memory request does not produce a valid layout")), // I would like to display `e` if possible
                Ok(layout) => {
                    let size = layout.size();
                    if size <= u32::MAX as usize {
                        if let Some(buf) = NonNull::new(unsafe { ffi::MemAlloc(size as u32) }.cast()) {
                            assert!(buf.is_aligned(), "allocated buffer should always be aligned");
                            Ok(Self { buf, len: count })
                        } else { Err(error!("memory request exceeds capacity")) }
                    } else { Err(error!("memory request exceeds unsigned integer maximum")) }
                }
            }
        } else { Err(error!("cannot allocate less than 1 element")) }
    }

    /// Reallocate memory managed by Raylib
    pub fn realloc(&mut self, new_count: i32) -> Result<(), Error> {
        if new_count >= 1 {
            let new_count = new_count as usize;
            match Layout::array::<T>(new_count) {
                Err(_e) => Err(error!("memory request does not produce a valid layout")), // I would like to display `e` if possible
                Ok(layout) => {
                    let size = layout.size();
                    if size <= u32::MAX as usize {
                        if let Some(buf) = NonNull::new(unsafe { ffi::MemRealloc(self.buf.as_ptr().cast(), size as u32) }.cast()) {
                            assert!(buf.is_aligned(), "allocated buffer should always be aligned");
                            self.buf = buf;
                            self.len = new_count;
                            Ok(())
                        } else { Err(error!("memory request exceeds capacity")) }
                    } else { Err(error!("memory request exceeds unsigned integer maximum")) }
                }
            }
        } else { Err(error!("cannot allocate less than 1 element")) }
    }
}

/// Compress data (DEFLATE algorythm)
/// ```rust
/// use raylib::prelude::*;
/// let data = compress_data(b"11111").unwrap();
/// let expected: &[u8] = &[1, 5, 0, 250, 255, 49, 49, 49, 49, 49];
/// assert_eq!(data.as_ref(), expected);
/// ```
pub fn compress_data(data: &[u8]) -> Result<DataBuf<u8>, Error> {
    let mut out_length: i32 = 0;
    // CompressData doesn't actually modify the data, but the header is wrong
    let buffer = {
        unsafe { ffi::CompressData(data.as_ptr() as *mut _, data.len() as i32, &mut out_length) }
    };
    DataBuf::new(buffer, out_length)
        .ok_or_else(|| error!("could not compress data"))
}

/// Decompress data (DEFLATE algorythm)
/// ```rust
/// use raylib::prelude::*;
/// let input: &[u8] = &[1, 5, 0, 250, 255, 49, 49, 49, 49, 49];
/// let expected: &[u8] = b"11111";
/// let data = decompress_data(input).unwrap();
/// assert_eq!(data.as_ref(), expected);
/// ```
pub fn decompress_data(data: &[u8]) -> Result<DataBuf<u8>, Error> {
    #[cfg(debug_assertions)]
    println!("{:?}", data.len());

    let mut out_length: i32 = 0;
    // CompressData doesn't actually modify the data, but the header is wrong
    let buffer = {
        unsafe { ffi::DecompressData(data.as_ptr() as *mut _, data.len() as i32, &mut out_length) }
    };
    DataBuf::new(buffer, out_length)
        .ok_or_else(|| error!("could not compress data"))
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

// Decode Base64 data
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
