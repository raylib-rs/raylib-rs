//! Data manipulation functions. Compress and Decompress with DEFLATE
use std::{
    ffi::{c_char, CString},
    ops::{Deref, DerefMut},
    path::Path,
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
#[derive(Debug)]
pub struct DataBuf {
    buf: *mut u8,
    len: usize,
}
impl Drop for DataBuf {
    fn drop(&mut self) {
        unsafe {
            ffi::MemFree(self.buf.cast());
        }
    }
}
impl Deref for DataBuf {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        // TODO: Consider frontloading `from_raw_parts` checks into `DataBuf::new()` and using `&*std::ptr::slice_from_raw_parts(self.buf, self.len)` here instead
        unsafe {
            std::slice::from_raw_parts(self.buf, self.len)
        }
    }
}
impl DerefMut for DataBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // TODO: Consider frontloading `from_raw_parts_mut` checks into `DataBuf::new()` and using `&mut *std::ptr::slice_from_raw_parts_mut(self.buf, self.len)` here instead
        unsafe {
            std::slice::from_raw_parts_mut(self.buf, self.len)
        }
    }
}
impl AsRef<[u8]> for DataBuf {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.deref()
    }
}
impl AsMut<[u8]> for DataBuf {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.deref_mut()
    }
}
impl DataBuf {
    pub(crate) fn new(buf: *mut u8, byte_count: i32) -> Option<DataBuf> {
        if buf.is_null() {
            None
        } else {
            assert!(byte_count >= 1, "non-null data should be at least 1 byte");
            Some(DataBuf { buf, len: byte_count as usize })
        }
    }
}

/// Compress data (DEFLATE algorythm)
/// ```rust
/// use raylib::prelude::*;
/// let data = compress_data(b"11111").unwrap();
/// let expected: &[u8] = &[1, 5, 0, 250, 255, 49, 49, 49, 49, 49];
/// assert_eq!(data.as_ref(), expected);
/// ```
pub fn compress_data(data: &[u8]) -> Result<DataBuf, Error> {
    let mut out_length: i32 = 0;
    // CompressData doesn't actually modify the data, but the header is wrong
    let buffer = {
        unsafe { ffi::CompressData(data.as_ptr() as *mut _, data.len() as i32, &mut out_length) }
    };
    if let Some(buffer) = DataBuf::new(buffer, out_length) {
        Ok(buffer)
    } else {
        Err(error!("could not compress data"))
    }
}

/// Decompress data (DEFLATE algorythm)
/// ```rust
/// use raylib::prelude::*;
/// let input: &[u8] = &[1, 5, 0, 250, 255, 49, 49, 49, 49, 49];
/// let expected: &[u8] = b"11111";
/// let data = decompress_data(input).unwrap();
/// assert_eq!(data.as_ref(), expected);
/// ```
pub fn decompress_data(data: &[u8]) -> Result<DataBuf, Error> {
    #[cfg(debug_assertions)]
    println!("{:?}", data.len());

    let mut out_length: i32 = 0;
    // CompressData doesn't actually modify the data, but the header is wrong
    let buffer = {
        unsafe { ffi::DecompressData(data.as_ptr() as *mut _, data.len() as i32, &mut out_length) }
    };
    if let Some(buffer) = DataBuf::new(buffer, out_length) {
        Ok(buffer)
    } else {
        Err(error!("could not compress data"))
    }
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
