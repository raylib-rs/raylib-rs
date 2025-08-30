use crate::{databuf::DataBuf, error::CompressionError, ffi};
use std::{
    ffi::{CString, c_char},
    mem::MaybeUninit,
    path::Path,
};

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
        .ok_or(CompressionError::CompressionFailed)
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
        .ok_or(CompressionError::CompressionFailed)
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
            .copied()
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
            .copied()
            .collect();
        b
    } else {
        s.to_vec()
    }
}
