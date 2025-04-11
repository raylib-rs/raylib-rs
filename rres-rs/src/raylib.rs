use std::{
    error::Error,
    ffi::{CStr, CString},
    fmt::{Debug, Display},
};

use crate::{is_zero, ResourceChunk, ResourceMulti};
use raylib::{
    audio::{RaylibAudio, Wave},
    models::Mesh,
    text::Font,
    texture::Image,
    RaylibHandle,
};

pub struct UnpackError(i32);
impl Debug for UnpackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            0 => f.write_str("No error, decompression/decryption successful"),
            1 => f.write_str("Encryption algorithm not supported"),
            2 => f.write_str("Invalid password on decryption"),
            3 => f.write_str("Compression algorithm not supported"),
            4 => f.write_str("Error on data decompression"),
            _ => f.write_str("Unknown error"),
        }
    }
}
impl Display for UnpackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            0 => f.write_str("No error, decompression/decryption successful"),
            1 => f.write_str("Encryption algorithm not supported"),
            2 => f.write_str("Invalid password on decryption"),
            3 => f.write_str("Compression algorithm not supported"),
            4 => f.write_str("Error on data decompression"),
            _ => f.write_str("Unknown error"),
        }
    }
}
impl Error for UnpackError {}

pub trait RRESImpl {
    /// Load raw data from rres resource
    fn load_data_from_resource<'a>(&self, chunk: ResourceChunk) -> &'a [u8] {
        let mut i: u32 = 0;
        let d = unsafe { rres_sys::LoadDataFromResource(chunk.0, &mut i) };

        unsafe { std::slice::from_raw_parts_mut(d as *mut u8, i as usize) }
    }

    /// Load text data from rres resource
    ///
    /// NOTE: Text must be NULL terminated
    fn load_text_from_resource<'a>(&self, chunk: ResourceChunk) -> &'a str {
        let d = unsafe { rres_sys::LoadTextFromResource(chunk.0) };
        unsafe { CStr::from_ptr(d).to_str().unwrap() }
    }

    /// Load Image data from rres resource
    fn load_image_from_resource(&self, chunk: ResourceChunk) -> Option<Image> {
        let d = unsafe { rres_sys::LoadImageFromResource(chunk.0) };
        if is_zero(&d) {
            return None;
        } else {
            return Some(unsafe { Image::from_raw(d) });
        }
    }

    /// Load Font data from rres resource. Returns None on failure.
    fn load_font_from_resource(&self, multi: ResourceMulti) -> Option<Font> {
        let d = unsafe { rres_sys::LoadFontFromResource(multi.0) };
        if is_zero(&d) {
            return None;
        } else {
            return Some(unsafe { Font::from_raw(d) });
        }
    }

    /// Load Mesh data from rres resource
    ///
    /// NOTE: We try to load vertex data following raylib structure constraints, in case data does not fit raylib Mesh structure, it is not loaded
    fn load_mesh_from_resource(&self, multi: ResourceMulti) -> Option<Mesh> {
        let d = unsafe { rres_sys::LoadMeshFromResource(multi.0) };
        if is_zero(&d) {
            return None;
        } else {
            return Some(unsafe { Mesh::from_raw(d) });
        }
    }

    /// Unpack compressed/encrypted data from resource chunk.
    ///
    /// In case data could not be processed, it is just copied in chunk.data.raw for processing here.
    ///
    /// NOTE: Data corruption CRC32 check has already been performed by rresLoadResourceMulti() on rres.h
    fn unpack_resource_chunk(&self, chunk: &mut ResourceChunk) -> Result<(), UnpackError> {
        let j = unsafe { rres_sys::UnpackResourceChunk(&mut chunk.0) };
        if j == 0 {
            return Ok(());
        } else {
            return Err(UnpackError(j));
        }
    }

    /// Set base directory for externally linked data
    ///
    /// NOTE: When resource chunk contains an external link (FourCC: LINK, Type: RRES_DATA_LINK), a base directory is required to be prepended to link path. If not provided, the application path is prepended to link by default
    fn set_base_directory(&self, base_dir: &str) {
        unsafe {
            let s = CString::new(base_dir).unwrap();
            rres_sys::SetBaseDirectory(s.as_ptr())
        }
    }
}

impl<'a> RRESImpl for RaylibHandle {}

pub trait RRESAudio<'a> {
    /// Load Wave data from rres resource
    fn load_wave_from_resource(&self, chunk: ResourceChunk) -> Option<Wave<'a>>;
}

// In order to construct a Wave from outside the crate, we have our own identical type that we construct
// and transmute.
#[allow(dead_code)]
struct IdenticalWave<'a>(raylib::ffi::Wave, &'a RaylibAudio);

impl<'a> RRESAudio<'a> for RaylibAudio {
    /// Load Wave data from rres resource
    fn load_wave_from_resource(&self, chunk: ResourceChunk) -> Option<Wave<'a>> {
        let d = unsafe { rres_sys::LoadWaveFromResource(chunk.0) };

        if is_zero(&d) {
            return None;
        } else {
            return Some(unsafe {
                std::mem::transmute(IdenticalWave(rres_sys::LoadWaveFromResource(chunk.0), self))
            });
        }
    }
}
