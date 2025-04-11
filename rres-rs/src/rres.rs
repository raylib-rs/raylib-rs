#![allow(non_camel_case_types)]

use std::{
    error::Error,
    ffi::{CStr, CString},
    fmt::Display,
    ops::{Deref, DerefMut},
    sync::Mutex,
};

use crate::is_zero;

#[repr(u32)]
pub enum ResourceDataType {
    DATA_NULL = rres_sys::rresResourceDataType::RRES_DATA_NULL as u32,
    DATA_RAW = rres_sys::rresResourceDataType::RRES_DATA_RAW as u32,

    DATA_TEXT = rres_sys::rresResourceDataType::RRES_DATA_TEXT as u32,

    DATA_IMAGE = rres_sys::rresResourceDataType::RRES_DATA_IMAGE as u32,

    DATA_WAVE = rres_sys::rresResourceDataType::RRES_DATA_WAVE as u32,

    DATA_VERTEX = rres_sys::rresResourceDataType::RRES_DATA_VERTEX as u32,

    DATA_FONT_GLYPHS = rres_sys::rresResourceDataType::RRES_DATA_FONT_GLYPHS as u32,

    DATA_LINK = rres_sys::rresResourceDataType::RRES_DATA_LINK as u32,

    DATA_DIRECTORY = rres_sys::rresResourceDataType::RRES_DATA_DIRECTORY as u32,
}

impl ResourceDataType {
    /// Get rresResourceDataType from FourCC code
    pub fn get_data_type(four_cc: [u8; 4]) -> Self {
        unsafe { std::mem::transmute(rres_sys::rresGetDataType(four_cc.as_ptr())) }
    }
}
#[repr(u32)]
pub enum CompressionType {
    COMP_NONE = rres_sys::rresCompressionType::RRES_COMP_NONE as u32,
    COMP_RLE = rres_sys::rresCompressionType::RRES_COMP_RLE as u32,
    COMP_DEFLATE = rres_sys::rresCompressionType::RRES_COMP_DEFLATE as u32,
    COMP_LZ4 = rres_sys::rresCompressionType::RRES_COMP_LZ4 as u32,
    COMP_LZMA2 = rres_sys::rresCompressionType::RRES_COMP_LZMA2 as u32,
    COMP_QOI = rres_sys::rresCompressionType::RRES_COMP_QOI as u32,
}
#[repr(u32)]
pub enum EncryptionType {
    CIPHER_NONE = rres_sys::rresEncryptionType::RRES_CIPHER_NONE as u32,
    CIPHER_XOR = rres_sys::rresEncryptionType::RRES_CIPHER_XOR as u32,
    CIPHER_DES = rres_sys::rresEncryptionType::RRES_CIPHER_DES as u32,
    CIPHER_TDES = rres_sys::rresEncryptionType::RRES_CIPHER_TDES as u32,
    CIPHER_IDEA = rres_sys::rresEncryptionType::RRES_CIPHER_IDEA as u32,
    CIPHER_AES = rres_sys::rresEncryptionType::RRES_CIPHER_AES as u32,
    CIPHER_AES_GCM = rres_sys::rresEncryptionType::RRES_CIPHER_AES_GCM as u32,
    CIPHER_XTEA = rres_sys::rresEncryptionType::RRES_CIPHER_XTEA as u32,
    CIPHER_BLOWFISH = rres_sys::rresEncryptionType::RRES_CIPHER_BLOWFISH as u32,
    CIPHER_RSA = rres_sys::rresEncryptionType::RRES_CIPHER_RSA as u32,
    CIPHER_SALSA20 = rres_sys::rresEncryptionType::RRES_CIPHER_SALSA20 as u32,
    CIPHER_CHACHA20 = rres_sys::rresEncryptionType::RRES_CIPHER_CHACHA20 as u32,
    CIPHER_XCHACHA20 = rres_sys::rresEncryptionType::RRES_CIPHER_XCHACHA20 as u32,
    CIPHER_XCHACHA20_POLY1305 = rres_sys::rresEncryptionType::RRES_CIPHER_XCHACHA20_POLY1305 as u32,
}
#[repr(u32)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorType {
    SUCCESS = rres_sys::rresErrorType::RRES_SUCCESS as u32,
    ERROR_FILE_NOT_FOUND = rres_sys::rresErrorType::RRES_ERROR_FILE_NOT_FOUND as u32,
    ERROR_FILE_FORMAT = rres_sys::rresErrorType::RRES_ERROR_FILE_FORMAT as u32,
    ERROR_MEMORY_ALLOC = rres_sys::rresErrorType::RRES_ERROR_MEMORY_ALLOC as u32,
}

impl Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorType::SUCCESS => f.write_str("Success"),
            ErrorType::ERROR_FILE_NOT_FOUND => f.write_str("File not found"),
            ErrorType::ERROR_FILE_FORMAT => f.write_str("File format error"),
            ErrorType::ERROR_MEMORY_ALLOC => f.write_str("Memory allocation error"),
        }
    }
}

impl Error for ErrorType {}

#[repr(u32)]
pub enum TextEncoding {
    TEXT_ENCODING_UNDEFINED = rres_sys::rresTextEncoding::RRES_TEXT_ENCODING_UNDEFINED as u32,
    TEXT_ENCODING_UTF8 = rres_sys::rresTextEncoding::RRES_TEXT_ENCODING_UTF8 as u32,
    TEXT_ENCODING_UTF8_BOM = rres_sys::rresTextEncoding::RRES_TEXT_ENCODING_UTF8_BOM as u32,
    TEXT_ENCODING_UTF16_LE = rres_sys::rresTextEncoding::RRES_TEXT_ENCODING_UTF16_LE as u32,
    TEXT_ENCODING_UTF16_BE = rres_sys::rresTextEncoding::RRES_TEXT_ENCODING_UTF16_BE as u32,
}
#[repr(u32)]
pub enum CodeLang {
    CODE_LANG_UNDEFINED = rres_sys::rresCodeLang::RRES_CODE_LANG_UNDEFINED as u32,
    CODE_LANG_C = rres_sys::rresCodeLang::RRES_CODE_LANG_C as u32,
    CODE_LANG_CPP = rres_sys::rresCodeLang::RRES_CODE_LANG_CPP as u32,
    CODE_LANG_CS = rres_sys::rresCodeLang::RRES_CODE_LANG_CS as u32,
    CODE_LANG_LUA = rres_sys::rresCodeLang::RRES_CODE_LANG_LUA as u32,
    CODE_LANG_JS = rres_sys::rresCodeLang::RRES_CODE_LANG_JS as u32,
    CODE_LANG_PYTHON = rres_sys::rresCodeLang::RRES_CODE_LANG_PYTHON as u32,
    CODE_LANG_RUST = rres_sys::rresCodeLang::RRES_CODE_LANG_RUST as u32,
    CODE_LANG_ZIG = rres_sys::rresCodeLang::RRES_CODE_LANG_ZIG as u32,
    CODE_LANG_ODIN = rres_sys::rresCodeLang::RRES_CODE_LANG_ODIN as u32,
    CODE_LANG_JAI = rres_sys::rresCodeLang::RRES_CODE_LANG_JAI as u32,
    CODE_LANG_GDSCRIPT = rres_sys::rresCodeLang::RRES_CODE_LANG_GDSCRIPT as u32,
    CODE_LANG_GLSL = rres_sys::rresCodeLang::RRES_CODE_LANG_GLSL as u32,
}
#[repr(u32)]
pub enum PixelFormat {
    PIXELFORMAT_UNDEFINED = rres_sys::rresPixelFormat::RRES_PIXELFORMAT_UNDEFINED as u32,
    PIXELFORMAT_UNCOMP_GRAYSCALE =
        rres_sys::rresPixelFormat::RRES_PIXELFORMAT_UNCOMP_GRAYSCALE as u32,
    PIXELFORMAT_UNCOMP_GRAY_ALPHA =
        rres_sys::rresPixelFormat::RRES_PIXELFORMAT_UNCOMP_GRAY_ALPHA as u32,
    PIXELFORMAT_UNCOMP_R5G6B5 = rres_sys::rresPixelFormat::RRES_PIXELFORMAT_UNCOMP_R5G6B5 as u32,
    PIXELFORMAT_UNCOMP_R8G8B8 = rres_sys::rresPixelFormat::RRES_PIXELFORMAT_UNCOMP_R8G8B8 as u32,
    PIXELFORMAT_UNCOMP_R5G5B5A1 =
        rres_sys::rresPixelFormat::RRES_PIXELFORMAT_UNCOMP_R5G5B5A1 as u32,
    PIXELFORMAT_UNCOMP_R4G4B4A4 =
        rres_sys::rresPixelFormat::RRES_PIXELFORMAT_UNCOMP_R4G4B4A4 as u32,
    PIXELFORMAT_UNCOMP_R8G8B8A8 =
        rres_sys::rresPixelFormat::RRES_PIXELFORMAT_UNCOMP_R8G8B8A8 as u32,
    PIXELFORMAT_UNCOMP_R32 = rres_sys::rresPixelFormat::RRES_PIXELFORMAT_UNCOMP_R32 as u32,
    PIXELFORMAT_UNCOMP_R32G32B32 =
        rres_sys::rresPixelFormat::RRES_PIXELFORMAT_UNCOMP_R32G32B32 as u32,
    PIXELFORMAT_UNCOMP_R32G32B32A32 =
        rres_sys::rresPixelFormat::RRES_PIXELFORMAT_UNCOMP_R32G32B32A32 as u32,
    PIXELFORMAT_COMP_DXT1_RGB = rres_sys::rresPixelFormat::RRES_PIXELFORMAT_COMP_DXT1_RGB as u32,
    PIXELFORMAT_COMP_DXT1_RGBA = rres_sys::rresPixelFormat::RRES_PIXELFORMAT_COMP_DXT1_RGBA as u32,
    PIXELFORMAT_COMP_DXT3_RGBA = rres_sys::rresPixelFormat::RRES_PIXELFORMAT_COMP_DXT3_RGBA as u32,
    PIXELFORMAT_COMP_DXT5_RGBA = rres_sys::rresPixelFormat::RRES_PIXELFORMAT_COMP_DXT5_RGBA as u32,
    PIXELFORMAT_COMP_ETC1_RGB = rres_sys::rresPixelFormat::RRES_PIXELFORMAT_COMP_ETC1_RGB as u32,
    PIXELFORMAT_COMP_ETC2_RGB = rres_sys::rresPixelFormat::RRES_PIXELFORMAT_COMP_ETC2_RGB as u32,
    PIXELFORMAT_COMP_ETC2_EAC_RGBA =
        rres_sys::rresPixelFormat::RRES_PIXELFORMAT_COMP_ETC2_EAC_RGBA as u32,
    PIXELFORMAT_COMP_PVRT_RGB = rres_sys::rresPixelFormat::RRES_PIXELFORMAT_COMP_PVRT_RGB as u32,
    PIXELFORMAT_COMP_PVRT_RGBA = rres_sys::rresPixelFormat::RRES_PIXELFORMAT_COMP_PVRT_RGBA as u32,
    PIXELFORMAT_COMP_ASTC_4x4_RGBA =
        rres_sys::rresPixelFormat::RRES_PIXELFORMAT_COMP_ASTC_4x4_RGBA as u32,
    PIXELFORMAT_COMP_ASTC_8x8_RGBA =
        rres_sys::rresPixelFormat::RRES_PIXELFORMAT_COMP_ASTC_8x8_RGBA as u32,
}
#[repr(u32)]
pub enum VertexAttribute {
    VERTEX_ATTRIBUTE_POSITION =
        rres_sys::rresVertexAttribute::RRES_VERTEX_ATTRIBUTE_POSITION as u32,
    VERTEX_ATTRIBUTE_TEXCOORD1 =
        rres_sys::rresVertexAttribute::RRES_VERTEX_ATTRIBUTE_TEXCOORD1 as u32,
    VERTEX_ATTRIBUTE_TEXCOORD2 =
        rres_sys::rresVertexAttribute::RRES_VERTEX_ATTRIBUTE_TEXCOORD2 as u32,
    VERTEX_ATTRIBUTE_TEXCOORD3 =
        rres_sys::rresVertexAttribute::RRES_VERTEX_ATTRIBUTE_TEXCOORD3 as u32,
    VERTEX_ATTRIBUTE_TEXCOORD4 =
        rres_sys::rresVertexAttribute::RRES_VERTEX_ATTRIBUTE_TEXCOORD4 as u32,
    VERTEX_ATTRIBUTE_NORMAL = rres_sys::rresVertexAttribute::RRES_VERTEX_ATTRIBUTE_NORMAL as u32,
    VERTEX_ATTRIBUTE_TANGENT = rres_sys::rresVertexAttribute::RRES_VERTEX_ATTRIBUTE_TANGENT as u32,
    VERTEX_ATTRIBUTE_COLOR = rres_sys::rresVertexAttribute::RRES_VERTEX_ATTRIBUTE_COLOR as u32,
    VERTEX_ATTRIBUTE_INDEX = rres_sys::rresVertexAttribute::RRES_VERTEX_ATTRIBUTE_INDEX as u32,
}
#[repr(u32)]
pub enum VertexFormat {
    VERTEX_FORMAT_UBYTE = rres_sys::rresVertexFormat::RRES_VERTEX_FORMAT_UBYTE as u32,
    VERTEX_FORMAT_BYTE = rres_sys::rresVertexFormat::RRES_VERTEX_FORMAT_BYTE as u32,
    VERTEX_FORMAT_USHORT = rres_sys::rresVertexFormat::RRES_VERTEX_FORMAT_USHORT as u32,
    VERTEX_FORMAT_SHORT = rres_sys::rresVertexFormat::RRES_VERTEX_FORMAT_SHORT as u32,
    VERTEX_FORMAT_UINT = rres_sys::rresVertexFormat::RRES_VERTEX_FORMAT_UINT as u32,
    VERTEX_FORMAT_INT = rres_sys::rresVertexFormat::RRES_VERTEX_FORMAT_INT as u32,
    VERTEX_FORMAT_HFLOAT = rres_sys::rresVertexFormat::RRES_VERTEX_FORMAT_HFLOAT as u32,
    VERTEX_FORMAT_FLOAT = rres_sys::rresVertexFormat::RRES_VERTEX_FORMAT_FLOAT as u32,
}
#[repr(u32)]
pub enum FontStyle {
    FONT_STYLE_UNDEFINED = rres_sys::rresFontStyle::RRES_FONT_STYLE_UNDEFINED as u32,
    FONT_STYLE_REGULAR = rres_sys::rresFontStyle::RRES_FONT_STYLE_REGULAR as u32,
    FONT_STYLE_BOLD = rres_sys::rresFontStyle::RRES_FONT_STYLE_BOLD as u32,
    FONT_STYLE_ITALIC = rres_sys::rresFontStyle::RRES_FONT_STYLE_ITALIC as u32,
}

pub struct ResourceChunk(pub(crate) rres_sys::rresResourceChunk);
pub struct ResourceMulti(pub(crate) rres_sys::rresResourceMulti);
pub struct ResourceChunkInfo(pub(crate) rres_sys::rresResourceChunkInfo);
pub struct CentralDir(pub(crate) rres_sys::rresCentralDir);
pub struct ResourceChunkData(pub(crate) rres_sys::rresResourceChunkData);
pub type FontGlyphInfo = rres_sys::rresFontGlyphInfo;

pub struct DirEntry(pub(crate) rres_sys::rresDirEntry);

pub type FileHeader = rres_sys::rresFileHeader;

impl DirEntry {
    pub unsafe fn from_raw(inner: rres_sys::rresDirEntry) -> Self {
        Self(inner)
    }
    /// Resource id
    pub fn id(&self) -> u32 {
        self.0.id
    }
    /// Resource global offset in file
    pub fn offset(&self) -> u32 {
        self.0.offset
    }

    /// Resource fileName size (NULL terminator and 4-byte alignment padding considered)
    pub fn file_name_size(&self) -> u32 {
        self.0.fileNameSize
    }

    /// Resource original fileName (NULL terminated and padded to 4-byte alignment)
    pub fn filename(&self) -> String {
        let r = self
            .0
            .fileName
            .iter()
            .map(|f| *f as u8)
            .collect::<Vec<u8>>();
        CStr::from_bytes_until_nul(&r)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
}

impl ResourceChunk {
    pub unsafe fn from_raw(inner: rres_sys::rresResourceChunk) -> Self {
        Self(inner)
    }

    /// Load one resource chunk for provided id
    pub fn load(file_name: &str, id: i32) -> Option<Self> {
        let r = unsafe {
            let cstr = CString::new(file_name).unwrap();
            rres_sys::rresLoadResourceChunk(cstr.as_ptr(), id)
        };

        if is_zero(&r) {
            None
        } else {
            Some(Self(r))
        }
    }
}

impl Drop for ResourceChunk {
    fn drop(&mut self) {
        unsafe { rres_sys::rresUnloadResourceChunk(self.0) }
    }
}

impl ResourceChunkData {
    pub unsafe fn from_raw(inner: rres_sys::rresResourceChunkData) -> Self {
        Self(inner)
    }
    pub fn prop_count(&self) -> u32 {
        self.0.propCount
    }
    pub fn props(&self) -> &[u32] {
        assert!(!self.0.props.is_null());
        assert!(self.0.props.is_aligned());
        unsafe { std::slice::from_raw_parts(self.0.props, self.0.propCount as usize) }
    }
    pub fn props_mut(&mut self) -> &mut [u32] {
        assert!(!self.0.props.is_null());
        assert!(self.0.props.is_aligned());
        unsafe { std::slice::from_raw_parts_mut(self.0.props, self.0.propCount as usize) }
    }
}

impl ResourceMulti {
    pub unsafe fn from_raw(inner: rres_sys::rresResourceMulti) -> Self {
        Self(inner)
    }
    /// Load resource for provided id (multiple resource chunks)
    pub fn load(file_name: &str, id: i32) -> Option<Self> {
        let r = unsafe {
            let cstr = CString::new(file_name).unwrap();
            rres_sys::rresLoadResourceMulti(cstr.as_ptr(), id)
        };

        if is_zero(&r) {
            None
        } else {
            Some(Self(r))
        }
    }

    pub fn count(&self) -> u32 {
        self.0.count
    }

    pub fn chunks(&self) -> Vec<ResourceChunk> {
        assert!(!self.0.chunks.is_null());
        assert!(self.0.chunks.is_aligned());
        unsafe { std::slice::from_raw_parts(self.0.chunks, self.0.count as usize) }
            .iter()
            .map(|f| ResourceChunk(*f))
            .collect::<Vec<_>>()
    }
}

impl Drop for ResourceMulti {
    fn drop(&mut self) {
        unsafe { rres_sys::rresUnloadResourceMulti(self.0) }
    }
}
impl ResourceChunkInfo {
    pub unsafe fn from_raw(inner: rres_sys::rresResourceChunkInfo) -> Self {
        Self(inner)
    }
    /// Get the type of the chunk
    pub fn get_type(&self) -> ResourceDataType {
        ResourceDataType::get_data_type(self.type_)
    }
    /// Load resource chunk info for provided id
    pub fn load(file_name: &str, id: i32) -> Option<Self> {
        let r = unsafe {
            let cstr = CString::new(file_name).unwrap();
            rres_sys::rresLoadResourceChunkInfo(cstr.as_ptr(), id)
        };
        if is_zero(&r) {
            None
        } else {
            Some(Self(r))
        }
    }
    /// Load all resource chunks info
    pub fn load_all(file_name: &str) -> Vec<Self> {
        let mut i: u32 = 0;
        let r = unsafe {
            let cstr = CString::new(file_name).unwrap();
            rres_sys::rresLoadResourceChunkInfoAll(cstr.as_ptr(), &mut i)
        };
        if !r.is_null() {
            assert!(!r.is_null());
            assert!(r.is_aligned());
            unsafe {
                std::slice::from_raw_parts_mut(r, i as usize)
                    .iter()
                    .map(|f| Self(*f))
                    .collect::<Vec<_>>()
            }
        } else {
            return Vec::new();
        }
    }
}
impl Deref for ResourceChunkInfo {
    type Target = rres_sys::rresResourceChunkInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ResourceChunkInfo {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl CentralDir {
    pub unsafe fn from_raw(inner: rres_sys::rresCentralDir) -> Self {
        Self(inner)
    }
    /// Load central directory resource chunk from file
    pub fn load(file_name: &str) -> Option<Self> {
        let r = unsafe {
            let cstr = CString::new(file_name).unwrap();
            rres_sys::rresLoadCentralDirectory(cstr.as_ptr())
        };

        if is_zero(&r) {
            None
        } else {
            Some(Self(r))
        }
    }

    /// Get resource identifier from filename
    ///
    /// WARNING: It requires the central directory previously loaded
    pub fn get_resource_id(&self, file_name: &str) -> Option<i32> {
        unsafe {
            let cstr = CString::new(file_name).unwrap();
            let id = rres_sys::rresGetResourceId(self.0, cstr.as_ptr());
            if id == 0 {
                None
            } else {
                Some(id)
            }
        }
    }

    pub fn count(&self) -> u32 {
        self.0.count
    }

    pub fn entries(&self) -> Vec<DirEntry> {
        assert!(!self.0.entries.is_null());
        assert!(self.0.entries.is_aligned());
        unsafe { std::slice::from_raw_parts(self.0.entries, self.0.count as usize) }
            .iter()
            .map(|f| DirEntry(*f))
            .collect::<Vec<_>>()
    }
}

impl Drop for CentralDir {
    fn drop(&mut self) {
        unsafe { rres_sys::rresUnloadCentralDirectory(self.0) }
    }
}

/// Compute CRC32 hash
///
/// NOTE: CRC32 is used as rres id, generated from original filename
pub fn compute_crc32(data: &[u8]) -> u32 {
    unsafe { rres_sys::rresComputeCRC32(data.as_ptr(), data.len() as i32) }
}

static CIPHER_MUTEX: Mutex<()> = Mutex::new(());

///
/// Get password to be used on data decryption.
///
/// Rust note: this function is made thread safe thanks to an internal Mutex.
pub fn get_cipher_password() -> &'static str {
    let _lock = CIPHER_MUTEX.lock().unwrap();
    unsafe {
        CStr::from_ptr(rres_sys::rresGetCipherPassword())
            .to_str()
            .unwrap()
    }
}

///
/// Set password to be used on data decryption.
///
/// Rust note: this function is made thread safe thanks to an internal Mutex.
pub fn set_cipher_password(pass: &str) {
    let _lock = CIPHER_MUTEX.lock().unwrap();
    unsafe {
        let cstr = CString::new(pass).unwrap();
        rres_sys::rresSetCipherPassword(cstr.as_ptr())
    };
}
