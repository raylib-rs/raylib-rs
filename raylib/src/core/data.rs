//! Memory compression/decompression

use crate::{databuf::DataBuf, error::CompressionError, ffi};
use std::{
    ffi::{CString, c_char},
    mem::MaybeUninit,
    path::Path,
};

/// Compress data (DEFLATE algorithm)
///
/// # Example
/// ```
/// use raylib::prelude::*;
/// let data = compress_data(b"11111").unwrap();
/// let expected: &[u8] = &[1, 5, 0, 250, 255, 49, 49, 49, 49, 49];
/// assert_eq!(data.as_ref(), expected);
/// ```
///
/// # Errors
///
/// This function returns [`CompressionError::CompressionFailed`] if the pointer returned by [`ffi::CompressData`] is null.
///
/// # Panics
///
/// This function will panic if `data` has a length greater than [`i32::MAX`] bytes.
pub fn compress_data(data: &[u8]) -> Result<DataBuf<[u8]>, CompressionError> {
    let mut out_length = MaybeUninit::uninit();
    // CompressData doesn't actually modify the data, but the header is wrong
    let buffer = {
        unsafe {
            ffi::CompressData(
                data.as_ptr().cast_mut(),
                data.len()
                    .try_into()
                    .expect("data should not exceed i32::MAX bytes"),
                out_length.as_mut_ptr(),
            )
        }
    };
    // SAFETY: `CompressData` returns a unique, owned pointer that is safe to dereference for
    // `out_length` valid, initialized elements if `buffer` is not null. It also guarantees
    // `out_length` is initialized if `buffer` is non-null.
    unsafe { DataBuf::slice_from_raw(buffer, out_length) }
        .ok_or(CompressionError::CompressionFailed)
}

/// Decompress data (DEFLATE algorithm)
///
/// # Example
/// ```
/// use raylib::prelude::*;
/// let input: &[u8] = &[1, 5, 0, 250, 255, 49, 49, 49, 49, 49];
/// let expected: &[u8] = b"11111";
/// let data = decompress_data(input).unwrap();
/// assert_eq!(data.as_ref(), expected);
/// ```
///
/// # Errors
///
/// This function returns [`CompressionError::CompressionFailed`] if the pointer returned by [`ffi::CompressData`] is null.
///
/// # Panics
///
/// This function will panic if `data` has a length greater than [`i32::MAX`] bytes.
pub fn decompress_data(data: &[u8]) -> Result<DataBuf<[u8]>, CompressionError> {
    #[cfg(debug_assertions)]
    println!("{:?}", data.len());

    let mut out_length = MaybeUninit::uninit();
    // CompressData doesn't actually modify the data, but the header is wrong
    let buffer = {
        unsafe {
            ffi::DecompressData(
                data.as_ptr().cast_mut(),
                data.len()
                    .try_into()
                    .expect("data should not exceed i32::MAX bytes"),
                out_length.as_mut_ptr(),
            )
        }
    };
    // SAFETY: `DecompressData` returns a unique, owned pointer that is safe to dereference for
    // `out_length` valid, initialized elements if `buffer` is not null. It also guarantees
    // `out_length` is initialized if `buffer` is non-null.
    unsafe { DataBuf::slice_from_raw(buffer, out_length) }
        .ok_or(CompressionError::CompressionFailed)
}

fn path_to_bytes<P: AsRef<Path>>(path: P) -> Vec<u8> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        path.as_ref().as_os_str().as_bytes().to_vec()
    }
    #[cfg(not(unix))]
    {
        path.as_ref().to_string_lossy().to_string().into_bytes()
    }
}

/// Export data to code (.h), returns true on success
///
/// # Panics
///
/// This function will panic if `file_name` contains an internal 0 byte or if `data` has a length greater than [`i32::MAX`].
#[must_use = "return indicates success or failure of the operation"]
pub fn export_data_as_code(data: &[u8], file_name: impl AsRef<Path>) -> bool {
    let c_str = CString::new(path_to_bytes(file_name))
        .expect("file_name should not have an internal 0 byte");

    unsafe {
        ffi::ExportDataAsCode(
            data.as_ptr(),
            data.len()
                .try_into()
                .expect("data should not exceed i32::MAX bytes"),
            c_str.as_ptr(),
        )
    }
}

/// Encode data to Base64 string
///
/// # Panics
///
/// This function will panic if `data` has a length greater than [`i32::MAX`] or if [`ffi::EncodeDataBase64`] gives a negative output size.
#[must_use]
pub fn encode_data_base64(data: &[u8]) -> Vec<c_char> {
    let mut output_size = 0;
    let bytes = unsafe {
        ffi::EncodeDataBase64(
            data.as_ptr(),
            data.len()
                .try_into()
                .expect("data should not exceed i32::MAX bytes"),
            &raw mut output_size,
        )
    };

    let s = unsafe {
        std::slice::from_raw_parts(
            bytes,
            output_size
                .try_into()
                .expect("output_size should not be negative"),
        )
    };
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
            .copied()
            .collect();
        b
    } else {
        s.to_vec()
    }
}

/// Decode Base64 data
///
/// # Panics
///
/// This function will panic if [`ffi::DecodeDataBase64`] gives a negative output size.
#[must_use]
pub fn decode_data_base64(data: &[u8]) -> Vec<u8> {
    let mut output_size = 0;

    let bytes = unsafe { ffi::DecodeDataBase64(data.as_ptr(), &raw mut output_size) };

    let s = unsafe {
        std::slice::from_raw_parts(
            bytes,
            output_size
                .try_into()
                .expect("output_size should not be negative"),
        )
    };
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
            .copied()
            .collect();
        b
    } else {
        s.to_vec()
    }
}
