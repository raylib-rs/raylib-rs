//! Text and Font related functions
//!
//! Text manipulation functions are super unsafe, so use Rust's [`String`] functions

use crate::core::math::Vector2;
use crate::core::texture::{Image, Texture2D};
use crate::core::{RaylibHandle, RaylibThread};
use crate::databuf::DataBuf;
use crate::error::{AllocationError, LoadFontError};
use crate::ffi;
use crate::ffi::Rectangle;

use std::convert::{AsMut, AsRef, TryInto};
use std::ffi::{CString, OsString};
use std::mem::ManuallyDrop;

fn no_drop<T>(_thing: T) {}
make_thin_wrapper!(
    /// Font, font texture and [`GlyphInfo`] array data
    Font,
    ffi::Font,
    ffi::UnloadFont
);
make_thin_wrapper!(
    /// Unowned version of [`Font`] that does not free the resource when dropped
    WeakFont,
    ffi::Font,
    no_drop
);
make_thin_wrapper!(
    /// [`GlyphInfo`], font characters glyphs info
    GlyphInfo,
    ffi::GlyphInfo,
    no_drop
);

/// An owned slice of [`GlyphInfo`]s that will be freed by Raylib.
#[repr(transparent)]
#[derive(Debug)]
pub struct RSliceGlyphInfo(pub(crate) std::mem::ManuallyDrop<Box<[GlyphInfo]>>);

impl Drop for RSliceGlyphInfo {
    fn drop(&mut self) {
        let inner = unsafe { std::mem::ManuallyDrop::take(&mut self.0) };
        let len = inner.len();
        unsafe {
            ffi::UnloadFontData(
                Box::leak(inner).as_mut_ptr().cast(),
                len.try_into().expect("len should not exceed i32::MAX"),
            );
        }
    }
}

impl AsRef<Box<[GlyphInfo]>> for RSliceGlyphInfo {
    #[inline]
    fn as_ref(&self) -> &Box<[GlyphInfo]> {
        &self.0
    }
}

impl AsMut<Box<[GlyphInfo]>> for RSliceGlyphInfo {
    #[inline]
    fn as_mut(&mut self) -> &mut Box<[GlyphInfo]> {
        &mut self.0
    }
}

impl std::ops::Deref for RSliceGlyphInfo {
    type Target = Box<[GlyphInfo]>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for RSliceGlyphInfo {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// #[cfg(feature = "nightly")]
// impl !Send for Font {}
// #[cfg(feature = "nightly")]
// unsafe impl Sync for Font {}
// #[cfg(feature = "nightly")]
// impl !Send for WeakFont {}
// #[cfg(feature = "nightly")]
// unsafe impl Sync for WeakFont {}

impl AsRef<ffi::Texture2D> for Font {
    #[inline]
    fn as_ref(&self) -> &ffi::Texture2D {
        &self.0.texture
    }
}

impl AsRef<ffi::Texture2D> for WeakFont {
    #[inline]
    fn as_ref(&self) -> &ffi::Texture2D {
        &self.0.texture
    }
}

pub(crate) struct Codepoints(pub(crate) ManuallyDrop<Box<[i32]>>);

impl Drop for Codepoints {
    fn drop(&mut self) {
        unsafe {
            ffi::UnloadCodepoints(self.0.as_mut_ptr());
        }
    }
}

const TOO_MANY_CODEPOINTS: &str = "fonts exceeding 2,147,483,647 characters are not supported; at most 1,112,064 characters can possibly be encoded in utf-8.";

impl RaylibHandle {
    /// Load all codepoints from a UTF-8 text string, codepoints count returned by parameter
    ///
    /// # Panics
    ///
    /// This method will panic if `text` contains an internal 0 byte, or if [`ffi::LoadCodepoints`] assigns a negative len.
    #[must_use]
    pub(crate) fn load_codepoints(text: &str) -> Codepoints {
        let ptr = CString::new(text).expect("text should not contain an internal 0 byte");
        let mut len = 0;
        // SAFETY: `ptr` is a valid c-string, and `len` is non-null and safe to dereference for writing.
        // `LoadCodepoints` is pure and does not require Raylib to be initialized.
        let u = unsafe { ffi::LoadCodepoints(ptr.as_ptr(), &raw mut len) };

        let data = unsafe {
            std::slice::from_raw_parts_mut(
                u,
                len.try_into()
                    .expect("codepoint count should never be negative"),
            )
        };
        Codepoints(std::mem::ManuallyDrop::new(unsafe { Box::from_raw(data) }))
    }

    /// Get total number of codepoints in a UTF-8 encoded string
    ///
    /// # Panics
    ///
    /// This method will panic if `text` contains an internal 0 byte.
    #[must_use]
    #[inline]
    pub fn get_codepoint_count(text: &str) -> i32 {
        let ptr = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe { ffi::GetCodepointCount(ptr.as_ptr()) }
    }

    /// Unload font from GPU memory (VRAM)
    ///
    /// Weak fonts will leak memory if they are not unlaoded
    ///
    /// # Safety
    ///
    /// This method frees the resource associated with `font`.
    /// The caller must ensure that `font` has not yet been unloaded, and that no copies
    /// of `font` are accessed or unloaded after this method returns.
    #[inline]
    pub unsafe fn unload_font(&mut self, font: WeakFont) {
        unsafe { ffi::UnloadFont(font.to_raw()) };
    }

    /// Loads font from file into GPU memory (VRAM).
    ///
    /// # Errors
    ///
    /// This method returns [`LoadFontError::LoadFromFileFailed`] if [`ffi::LoadFont`] returns
    /// a font whose glyphs are null or texture id is 0.
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte.
    #[inline]
    pub fn load_font(&mut self, _: &RaylibThread, filename: &str) -> Result<Font, LoadFontError> {
        let c_filename =
            CString::new(filename).expect("filename should not contain an internal 0 byte");
        let f = unsafe { ffi::LoadFont(c_filename.as_ptr()) };
        if f.glyphs.is_null() || f.texture.id == 0 {
            return Err(LoadFontError::LoadFromFileFailed {
                path: filename.into(),
            });
        }
        Ok(Font(f))
    }

    /// Loads font from file with extended parameters.
    ///
    /// Supplying [`None`] for chars loads the entire character set.
    ///
    /// # Errors
    ///
    /// This method returns [`LoadFontError::LoadFromFileFailed`] if [`ffi::LoadFontEx`] returns
    /// a font whose glyphs are null or texture id is 0.
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte, or if the codepoints loaded from `chars`
    /// exceed [`i32::MAX`] elements.
    #[inline]
    pub fn load_font_ex(
        &mut self,
        _: &RaylibThread,
        filename: &str,
        font_size: i32,
        chars: Option<&str>,
    ) -> Result<Font, LoadFontError> {
        let c_filename =
            CString::new(filename).expect("filename should not contain an internal 0 byte");
        let mut co;
        let (co_ptr, co_len) = match chars {
            Some(c) => {
                co = Self::load_codepoints(c);
                (
                    co.0.as_mut_ptr(),
                    co.0.len().try_into().expect(TOO_MANY_CODEPOINTS),
                )
            }
            None => (std::ptr::null_mut(), 0),
        };
        let f = unsafe { ffi::LoadFontEx(c_filename.as_ptr(), font_size, co_ptr, co_len) };
        if f.glyphs.is_null() || f.texture.id == 0 {
            return Err(LoadFontError::LoadFromFileFailed {
                path: filename.into(),
            });
        }
        Ok(Font(f))
    }

    /// Load font from Image (XNA style)
    ///
    /// # Errors
    ///
    /// This method returns [`LoadFontError::LoadFromImageFailed`] if [`ffi::LoadFontFromImage`] returns
    /// a font whose glyphs are null or texture id is 0.
    #[inline]
    pub fn load_font_from_image(
        &mut self,
        _: &RaylibThread,
        image: &Image,
        key: impl Into<ffi::Color>,
        first_char: char,
    ) -> Result<Font, LoadFontError> {
        let f = unsafe { ffi::LoadFontFromImage(image.0, key.into(), first_char as i32) };
        if f.glyphs.is_null() || f.texture.id == 0 {
            Err(LoadFontError::LoadFromImageFailed)
        } else {
            Ok(Font(f))
        }
    }
    /// Load font data from a given memory buffer.
    /// `file_type` refers to the extension, e.g. ".ttf".
    /// You can pass [`Some`] to chars to get the desired characters, or None to get the whole set.
    ///
    /// # Errors
    ///
    /// This method returns [`LoadFontError::LoadFromMemoryFailed`] if [`ffi::LoadFontFromMemory`] returns
    /// a font whose glyphs are null or texture id is 0.
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte, or if either `file_data`
    /// or the codepoints loaded from `chars` exceed [`i32::MAX`] elements.
    #[inline]
    pub fn load_font_from_memory(
        &mut self,
        _: &RaylibThread,
        file_type: &str,
        file_data: &[u8],
        font_size: i32,
        chars: Option<&str>,
    ) -> Result<Font, LoadFontError> {
        let c_file_type =
            CString::new(file_type).expect("file_type should not contain an internal 0 byte");
        let mut co;
        let (co_ptr, co_len) = match chars {
            Some(c) => {
                co = Self::load_codepoints(c);
                (
                    co.0.as_mut_ptr(),
                    co.0.len().try_into().expect(TOO_MANY_CODEPOINTS),
                )
            }
            None => (std::ptr::null_mut(), 0),
        };
        let f = unsafe {
            ffi::LoadFontFromMemory(
                c_file_type.as_ptr(),
                file_data.as_ptr(),
                file_data
                    .len()
                    .try_into()
                    .expect("file_data should not exceed i32::MAX elements"),
                font_size,
                co_ptr,
                co_len,
            )
        };
        if f.glyphs.is_null() || f.texture.id == 0 {
            return Err(LoadFontError::LoadFromMemoryFailed);
        }
        Ok(Font(f))
    }
    /// Loads font data for further use (see also `Font::from_data`).
    /// Now supports .tiff
    ///
    /// Returns [`None`] if [`ffi::LoadFontData`] returns null.
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte, or if either `file_data`
    /// or the codepoints loaded from `chars` exceed [`i32::MAX`] elements.
    #[inline]
    pub fn load_font_data(
        &mut self,
        data: &[u8],
        font_size: i32,
        chars: Option<&str>,
        sdf: i32,
    ) -> Option<RSliceGlyphInfo> {
        let mut co;
        let (co_ptr, co_len) = match chars {
            Some(c) => {
                co = Self::load_codepoints(c);
                (
                    co.0.as_mut_ptr(),
                    co.0.len().try_into().expect(TOO_MANY_CODEPOINTS),
                )
            }
            None => (std::ptr::null_mut(), 0),
        };
        let ci_arr_ptr = unsafe {
            ffi::LoadFontData(
                data.as_ptr(),
                data.len()
                    .try_into()
                    .expect("data should not exceed i32::MAX elements"),
                font_size,
                co_ptr,
                co_len,
                sdf,
            )
        };
        let ci_size = chars.map_or(95, str::len); // raylib assumes 95 if none given
        (!ci_arr_ptr.is_null()).then(|| {
            let data = unsafe { std::slice::from_raw_parts_mut(ci_arr_ptr.cast(), ci_size) };
            RSliceGlyphInfo(std::mem::ManuallyDrop::new(unsafe { Box::from_raw(data) }))
        })
    }
}

impl RaylibFont for WeakFont {}
impl RaylibFont for Font {}

/// [`Font`] accessors and helper methods.
pub trait RaylibFont {
    /// Base size (default chars height)
    #[inline]
    #[must_use]
    fn base_size(&self) -> i32
    where
        Self: AsRef<ffi::Font>,
    {
        self.as_ref().baseSize
    }
    /// Texture atlas containing the glyphs
    #[inline]
    #[must_use]
    fn texture(&self) -> &Texture2D
    where
        Self: AsRef<ffi::Font>,
    {
        unsafe { &*std::ptr::from_ref(&self.as_ref().texture).cast() }
    }
    /// Glyphs info data
    #[inline]
    #[must_use]
    fn chars(&self) -> &[GlyphInfo]
    where
        Self: AsRef<ffi::Font>,
    {
        let glyph_count = self
            .as_ref()
            .glyphCount
            .try_into()
            .expect("glyphCount should not be negative");
        unsafe { std::slice::from_raw_parts(self.as_ref().glyphs.cast(), glyph_count) }
    }
    /// Glyphs info data
    #[inline]
    #[must_use]
    fn chars_mut(&mut self) -> &mut [GlyphInfo]
    where
        Self: AsMut<ffi::Font>,
    {
        let glyph_count = self
            .as_mut()
            .glyphCount
            .try_into()
            .expect("glyphCount should not be negative");
        unsafe { std::slice::from_raw_parts_mut(self.as_mut().glyphs.cast(), glyph_count) }
    }

    /// Check if a font is valid
    #[inline]
    #[must_use]
    fn is_font_valid(&self) -> bool
    where
        Self: AsRef<ffi::Font>,
    {
        unsafe { ffi::IsFontValid(*self.as_ref()) }
    }

    /// Export font as code file, returns true on success
    #[must_use]
    fn export_font_as_code<A>(&self, filename: A) -> bool
    where
        Self: AsRef<ffi::Font>,
        A: Into<OsString>,
    {
        let c_str = CString::new(filename.into().to_string_lossy().as_bytes())
            .expect("lossy filename string should not contain an internal 0 byte");
        unsafe { ffi::ExportFontAsCode(*self.as_ref(), c_str.as_ptr()) }
    }

    /// Get glyph font info data for a codepoint (unicode character), fallback to '?' if not found
    #[inline]
    #[must_use]
    fn get_glyph_info(&self, codepoint: char) -> GlyphInfo
    where
        Self: AsRef<ffi::Font>,
    {
        unsafe { GlyphInfo(ffi::GetGlyphInfo(*self.as_ref(), codepoint as i32)) }
    }

    /// Gets index position for a unicode character on `font`.
    #[inline]
    #[must_use]
    fn get_glyph_index(&self, codepoint: char) -> i32
    where
        Self: AsRef<ffi::Font>,
    {
        unsafe { ffi::GetGlyphIndex(*self.as_ref(), codepoint as i32) }
    }

    /// Get glyph rectangle in font atlas for a codepoint (unicode character), fallback to '?' if not found
    #[inline]
    #[must_use]
    fn get_glyph_atlas_rec(&self, codepoint: char) -> Rectangle
    where
        Self: AsRef<ffi::Font>,
    {
        unsafe { ffi::GetGlyphAtlasRec(*self.as_ref(), codepoint as i32) }
    }

    /// Measures string width in pixels for `font`.
    #[must_use]
    fn measure_text(&self, text: &str, font_size: f32, spacing: f32) -> Vector2
    where
        Self: AsRef<ffi::Font>,
    {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe { ffi::MeasureTextEx(*self.as_ref(), c_text.as_ptr(), font_size, spacing).into() }
    }
}

impl Font {
    /// Convert `self` to its weak form, allowing it to be shared by multiple containers on the condition
    /// that it is only unloaded once, manually.
    ///
    /// # Safety
    ///
    /// Must manually free memory by calling the proper unload function.
    /// Even if `self` implements [`Copy`], exactly one instance should be unloaded to avoid double-free,
    /// and copies must not be used after being unloaded to avoid use-after-free.
    #[inline]
    #[must_use]
    pub const fn make_weak(self) -> WeakFont {
        let w = WeakFont(self.0);
        std::mem::forget(self);
        w
    }
    /// Returns a new [`Font`] using provided [`GlyphInfo`] data and parameters.
    fn from_data(
        chars: &[ffi::GlyphInfo],
        base_size: i32,
        padding: i32,
        pack_method: i32,
    ) -> Result<Font, LoadFontError> {
        let mut f = unsafe { std::mem::zeroed::<Font>() };
        f.baseSize = base_size;
        f.set_chars(chars)
            .expect("should be able to allocate memory for chars");

        let atlas = unsafe {
            ffi::GenImageFontAtlas(
                f.glyphs,
                &raw mut f.0.recs,
                f.baseSize,
                f.glyphCount,
                padding,
                pack_method,
            )
        };
        f.texture = unsafe { ffi::LoadTextureFromImage(atlas) };
        unsafe {
            ffi::UnloadImage(atlas);
        }
        if f.0.glyphs.is_null() || f.0.texture.id == 0 {
            return Err(LoadFontError::LoadFromImageFailed);
        }
        Ok(f)
    }

    /// Sets the character data on the current [`Font`].
    fn set_chars(&mut self, chars: &[ffi::GlyphInfo]) -> Result<(), AllocationError> {
        let glyph_count = chars
            .len()
            .try_into()
            .expect("chars should not exceed i32::MAX elements");
        // raylib frees this data in UnloadFont
        let glyphs = DataBuf::<[ffi::GlyphInfo]>::alloc_from_copy(chars)?
            .into_inner()
            .into_inner()
            .as_ptr()
            .cast();
        self.glyphCount = glyph_count;
        self.glyphs = glyphs;
        Ok(())
    }

    /// Sets the texture on the current [`Font`], and takes ownership of `tex`.
    fn set_texture(&mut self, tex: Texture2D) {
        self.texture = tex.0;
        std::mem::forget(tex); // UnloadFont will also unload the texture
    }
}

/// Generates image font atlas using `chars` info.
///
/// # Panics
///
/// This function will panic if `chars` has a length greater than [`i32::MAX`],
/// or if [`ffi::GenImageFontAtlas`] does not assign a value to `glyphRecs`.
#[inline]
pub fn gen_image_font_atlas(
    _: &RaylibThread,
    chars: &mut [ffi::GlyphInfo],
    font_size: i32,
    padding: i32,
    pack_method: i32,
) -> (Image, DataBuf<[ffi::Rectangle]>) {
    let mut ptr = std::ptr::null_mut();
    let glyph_count = chars
        .len()
        .try_into()
        .expect("chars must not exceed i32::MAX elements");

    let img = Image(unsafe {
        ffi::GenImageFontAtlas(
            chars.as_mut_ptr(),
            &raw mut ptr,
            glyph_count,
            font_size,
            padding,
            pack_method,
        )
    });

    let recs = unsafe { DataBuf::slice_from_raw(ptr, std::mem::MaybeUninit::new(glyph_count)) }
        .expect("ptr should not be null");

    (img, recs)
}

impl RaylibHandle {
    /// Gets the default font.
    #[inline]
    #[must_use]
    pub fn get_font_default(&self) -> WeakFont {
        WeakFont(unsafe { ffi::GetFontDefault() })
    }
    /// Measures string width in pixels for default font.
    ///
    /// # Panics
    ///
    /// This method will panic if `text` contains an internal 0 byte.
    #[must_use]
    pub fn measure_text(&self, text: &str, font_size: i32) -> i32 {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe { ffi::MeasureText(c_text.as_ptr(), font_size) }
    }

    /// Set vertical line spacing when drawing with line-breaks
    #[inline]
    pub fn set_text_line_spacing(&self, spacing: i32) {
        unsafe { ffi::SetTextLineSpacing(spacing) }
    }
}
