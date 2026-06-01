//! Text and Font related functions
//! Text manipulation functions are super unsafe so use rust String functions

use crate::core::math::Vector2;
use crate::core::texture::{Image, Texture2D};
use crate::core::{RaylibHandle, RaylibThread};
use crate::error::LoadFontError;
use crate::ffi;
use crate::ffi::Rectangle;

use std::convert::{AsMut, AsRef, TryInto};
use std::ffi::{CString, OsString};
use std::mem::ManuallyDrop;

fn no_drop<T>(_thing: T) {}
make_thin_wrapper!(
    /// Font: a glyph atlas texture plus per-glyph metrics.
    ///
    /// raylib ships a built-in default font (used when `None` or no font is specified in
    /// draw-text calls). User-defined fonts are loaded via `RaylibHandle` methods:
    ///
    /// - [`RaylibHandle::load_font`] — loads the default size and full glyph set from a
    ///   TTF/BDF/FNT file.
    /// - [`RaylibHandle::load_font_ex`] — loads with an explicit point size and optional
    ///   Unicode codepoint list (pass `None` for the entire set).
    /// - [`RaylibHandle::load_font_from_image`] — XNA-style bitmap font from an image.
    /// - [`RaylibHandle::load_font_from_memory`] — loads from an in-memory file buffer.
    ///
    /// Freed via `UnloadFont` on drop.
    ///
    /// # Examples
    ///
    /// Load a font with explicit size and a Latin-1 subset, then draw text:
    ///
    /// ```rust,no_run
    /// use raylib::prelude::*;
    /// let (mut rl, thread) = raylib::init().size(640, 480).title("font demo").build();
    /// // Load a TTF at 32 pt, restricting to printable ASCII (codepoints 32-126).
    /// let ascii: String = (32u8..=126).map(|c| c as char).collect();
    /// let font = rl
    ///     .load_font_ex(&thread, "assets/Roboto.ttf", 32, Some(&ascii))
    ///     .unwrap();
    /// while !rl.window_should_close() {
    ///     let mut d = rl.begin_drawing(&thread);
    ///     d.clear_background(Color::RAYWHITE);
    ///     d.draw_text_ex(&font, "Hello, raylib!", Vector2::new(10.0, 10.0), 32.0, 2.0, Color::BLACK);
    /// }
    /// ```
    Font,
    ffi::Font,
    ffi::UnloadFont
);
make_thin_wrapper!(WeakFont, ffi::Font, no_drop);
make_thin_wrapper!(
    /// GlyphInfo, font characters glyphs info
    GlyphInfo,
    ffi::GlyphInfo,
    no_drop
);

/// An owned slice of [`GlyphInfo`] data allocated by raylib, freed via `UnloadFontData` on drop.
///
/// Wraps a raylib-allocated `Box<[GlyphInfo]>` and calls `UnloadFontData` on drop so the
/// glyph buffer is returned to raylib's allocator rather than Rust's. Implements
/// [`Deref`](std::ops::Deref) + [`DerefMut`](std::ops::DerefMut) to `Box<[GlyphInfo]>` so
/// it acts like a slice: index with `slice[i]`, iterate with `slice.iter()`, or read
/// `slice.len()` directly.
///
/// # Construction
///
/// `RSliceGlyphInfo` has no public constructor; instances are produced by
/// internal font-loading paths and surface to callers through [`Font`]'s loading APIs.
/// Never call `libc::free` on the inner data pointer — the `Drop` impl routes through
/// `UnloadFontData` so custom allocators stay correct.
///
/// # See also
///
/// - [`GlyphInfo`] — per-codepoint metrics held in the slice.
/// - [`Font`] — owning font built from this glyph data.
#[repr(transparent)]
#[derive(Debug)]
pub struct RSliceGlyphInfo(pub(crate) std::mem::ManuallyDrop<std::boxed::Box<[GlyphInfo]>>);

impl Drop for RSliceGlyphInfo {
    #[allow(unused_unsafe)]
    fn drop(&mut self) {
        unsafe {
            let inner = std::mem::ManuallyDrop::take(&mut self.0);
            let len = inner.len();
            ffi::UnloadFontData(
                std::boxed::Box::leak(inner).as_mut_ptr() as *mut _,
                len as i32,
            );
        }
    }
}

impl std::convert::AsRef<Box<[GlyphInfo]>> for RSliceGlyphInfo {
    fn as_ref(&self) -> &Box<[GlyphInfo]> {
        &self.0
    }
}

impl std::convert::AsMut<Box<[GlyphInfo]>> for RSliceGlyphInfo {
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
    fn as_ref(&self) -> &ffi::Texture2D {
        &self.0.texture
    }
}

impl AsRef<ffi::Texture2D> for WeakFont {
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
    #[must_use]
    /// Load all codepoints from a UTF-8 text string, codepoints count returned by parameter
    pub(crate) fn load_codepoints(&mut self, text: &str) -> Codepoints {
        let ptr = CString::new(text).unwrap();
        let mut len = 0;
        let u = unsafe { ffi::LoadCodepoints(ptr.as_ptr(), &mut len) };

        unsafe {
            Codepoints(std::mem::ManuallyDrop::new(Box::from_raw(
                std::slice::from_raw_parts_mut(
                    u,
                    len.try_into()
                        .expect("codepoint count should never be negative"),
                ),
            )))
        }
    }

    #[must_use]
    #[inline]
    /// Get total number of codepoints in a UTF-8 encoded string
    pub fn get_codepoint_count(text: &str) -> i32 {
        let ptr = CString::new(text).unwrap();
        unsafe { ffi::GetCodepointCount(ptr.as_ptr()) }
    }

    /// Unload font from GPU memory (VRAM)
    #[inline]
    pub fn unload_font(&mut self, font: WeakFont) {
        unsafe { ffi::UnloadFont(font.0) };
    }

    /// Loads font from file into GPU memory (VRAM).
    #[inline]
    pub fn load_font(&mut self, _: &RaylibThread, filename: &str) -> Result<Font, LoadFontError> {
        let c_filename = CString::new(filename).unwrap();
        let f = unsafe { ffi::LoadFont(c_filename.as_ptr()) };
        if f.glyphs.is_null() || f.texture.id == 0 {
            return Err(LoadFontError::LoadFromFileFailed {
                path: filename.into(),
            });
        }
        Ok(Font(f))
    }

    /// Loads font from file with extended parameters.
    /// Supplying None for chars loads the entire character set.
    #[inline]
    pub fn load_font_ex(
        &mut self,
        _: &RaylibThread,
        filename: &str,
        font_size: i32,
        chars: Option<&str>,
    ) -> Result<Font, LoadFontError> {
        let c_filename = CString::new(filename).unwrap();
        let f = unsafe {
            match chars {
                Some(c) => {
                    let mut co = self.load_codepoints(c);
                    ffi::LoadFontEx(
                        c_filename.as_ptr(),
                        font_size,
                        co.0.as_mut_ptr(),
                        co.0.len().try_into().expect(TOO_MANY_CODEPOINTS),
                    )
                }
                None => ffi::LoadFontEx(c_filename.as_ptr(), font_size, std::ptr::null_mut(), 0),
            }
        };
        if f.glyphs.is_null() || f.texture.id == 0 {
            return Err(LoadFontError::LoadFromFileFailed {
                path: filename.into(),
            });
        }
        Ok(Font(f))
    }

    /// Load font from Image (XNA style)
    #[inline]
    pub fn load_font_from_image(
        &mut self,
        _: &RaylibThread,
        image: &Image,
        key: impl Into<ffi::Color>,
        first_char: i32,
    ) -> Result<Font, LoadFontError> {
        let f = unsafe { ffi::LoadFontFromImage(image.0, key.into(), first_char) };
        if f.glyphs.is_null() {
            return Err(LoadFontError::LoadFromImageFailed);
        }
        Ok(Font(f))
    }
    /// Load font data from a given memory buffer.
    /// `file_type` refers to the extension, e.g. ".ttf".
    /// You can pass Some(...) to chars to get the desired characters, or None to get the whole set.
    #[inline]
    pub fn load_font_from_memory(
        &mut self,
        _: &RaylibThread,
        file_type: &str,
        file_data: &[u8],
        font_size: i32,
        chars: Option<&str>,
    ) -> Result<Font, LoadFontError> {
        let c_file_type = CString::new(file_type).unwrap();
        let f = unsafe {
            match chars {
                Some(c) => {
                    let mut co = self.load_codepoints(c);
                    ffi::LoadFontFromMemory(
                        c_file_type.as_ptr(),
                        file_data.as_ptr(),
                        file_data.len() as i32,
                        font_size,
                        co.0.as_mut_ptr(),
                        co.0.len().try_into().expect(TOO_MANY_CODEPOINTS),
                    )
                }
                None => ffi::LoadFontFromMemory(
                    c_file_type.as_ptr(),
                    file_data.as_ptr(),
                    file_data.len() as i32,
                    font_size,
                    std::ptr::null_mut(),
                    0,
                ),
            }
        };
        if f.glyphs.is_null() || f.texture.id == 0 {
            return Err(LoadFontError::LoadFromMemoryFailed);
        }
        Ok(Font(f))
    }
    /// Loads font data for further use (see also `Font::from_data`).
    /// Now supports .tiff
    #[inline]
    pub fn load_font_data(
        &mut self,
        data: &[u8],
        font_size: i32,
        chars: Option<&str>,
        sdf: i32,
    ) -> Option<GlyphInfo> {
        unsafe {
            let mut glyph_count: i32 = 0;
            let glyph_info = match chars {
                Some(c) => {
                    let mut co = self.load_codepoints(c);
                    ffi::LoadFontData(
                        data.as_ptr(),
                        data.len() as i32,
                        font_size,
                        co.0.as_mut_ptr(),
                        co.0.len().try_into().expect(TOO_MANY_CODEPOINTS),
                        sdf,
                        &mut glyph_count,
                    )
                }
                None => ffi::LoadFontData(
                    data.as_ptr(),
                    data.len() as i32,
                    font_size,
                    std::ptr::null_mut(),
                    0,
                    sdf,
                    &mut glyph_count,
                ),
            };
            if glyph_info.is_null() {
                return None;
            }

            Some(GlyphInfo::from_raw(*glyph_info))
        }
    }
}

impl RaylibFont for WeakFont {}
impl RaylibFont for Font {}

/// Extension methods for types that wrap a raylib `Font` (both owned [`Font`] and [`WeakFont`]).
///
/// Implemented for [`Font`] (RAII-owned) and [`WeakFont`] (no-drop alias from
/// [`Font::make_weak`]). Provides field accessors ([`base_size`](Self::base_size),
/// [`texture`](Self::texture), [`chars`](Self::chars), [`chars_mut`](Self::chars_mut)),
/// validity check ([`is_font_valid`](Self::is_font_valid)), glyph-table lookups
/// ([`get_glyph_info`](Self::get_glyph_info),
/// [`get_glyph_index`](Self::get_glyph_index),
/// [`get_glyph_atlas_rec`](Self::get_glyph_atlas_rec)), the measurement helper
/// [`measure_text`](Self::measure_text), and the code-export helper
/// [`export_font_as_code`](Self::export_font_as_code).
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
/// use raylib::core::text::RaylibFont;
///
/// let (mut rl, thread) = raylib::init().size(640, 480).title("font").build();
/// let font = rl
///     .load_font(&thread, "assets/font.ttf")
///     .expect("font load");
/// let size = font.measure_text("hello", 32.0, 1.0);
/// println!("rendered width: {}", size.x);
/// ```
///
/// # See also
///
/// - [`Font`] — owning font handle.
/// - [`GlyphInfo`] — per-codepoint metrics returned by [`get_glyph_info`](Self::get_glyph_info).
pub trait RaylibFont: AsRef<ffi::Font> + AsMut<ffi::Font> {
    /// Base size (default chars height)
    #[inline]
    #[must_use]
    fn base_size(&self) -> i32 {
        self.as_ref().baseSize
    }
    /// Texture atlas containing the glyphs
    #[inline]
    #[must_use]
    fn texture(&self) -> &Texture2D {
        unsafe { std::mem::transmute(&self.as_ref().texture) }
    }
    /// Glyphs info data
    #[inline]
    #[must_use]
    fn chars(&self) -> &[GlyphInfo] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().glyphs as *const GlyphInfo,
                self.as_ref().glyphCount as usize,
            )
        }
    }
    /// Glyphs info data
    #[inline]
    #[must_use]
    fn chars_mut(&mut self) -> &mut [GlyphInfo] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().glyphs as *mut GlyphInfo,
                self.as_ref().glyphCount as usize,
            )
        }
    }

    /// Check if a font is valid
    #[inline]
    #[must_use]
    fn is_font_valid(&self) -> bool {
        unsafe { ffi::IsFontValid(*self.as_ref()) }
    }

    /// Export font as code file, returns true on success
    #[must_use]
    fn export_font_as_code<A>(&self, filename: A) -> bool
    where
        A: Into<OsString>,
    {
        let c_str = CString::new(filename.into().to_string_lossy().as_bytes()).unwrap();
        unsafe { ffi::ExportFontAsCode(*self.as_ref(), c_str.as_ptr()) }
    }

    /// Get glyph font info data for a codepoint (unicode character), fallback to '?' if not found
    #[inline]
    #[must_use]
    fn get_glyph_info(&self, codepoint: char) -> GlyphInfo {
        unsafe { GlyphInfo(ffi::GetGlyphInfo(*self.as_ref(), codepoint as i32)) }
    }

    /// Gets index position for a unicode character on `font`.
    #[inline]
    #[must_use]
    fn get_glyph_index(&self, codepoint: char) -> i32 {
        unsafe { ffi::GetGlyphIndex(*self.as_ref(), codepoint as i32) }
    }

    /// Get glyph rectangle in font atlas for a codepoint (unicode character), fallback to '?' if not found
    #[inline]
    #[must_use]
    fn get_glyph_atlas_rec(&self, codepoint: char) -> Rectangle {
        unsafe { ffi::GetGlyphAtlasRec(*self.as_ref(), codepoint as i32) }
    }

    /// Measures string width in pixels for `font`.
    #[must_use]
    fn measure_text(&self, text: &str, font_size: f32, spacing: f32) -> Vector2 {
        let c_text = CString::new(text).unwrap();
        unsafe { ffi::MeasureTextEx(*self.as_ref(), c_text.as_ptr(), font_size, spacing) }
    }
}

impl Font {
    /// Converts this `Font` into a [`WeakFont`] that does not run `UnloadFont` on drop.
    ///
    /// Use this when you want to hand a font to code that holds it by `WeakFont` (e.g. a
    /// renderer that doesn't own the resource) **and** you take responsibility for
    /// unloading the GPU atlas yourself. Forgets the owning `Font` so the `Drop` impl
    /// doesn't run; the returned `WeakFont` keeps the same texture and glyph table but
    /// will *not* free them on drop.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use raylib::prelude::*;
    ///
    /// let (mut rl, thread) = raylib::init().size(640, 480).title("font").build();
    /// let font = rl
    ///     .load_font(&thread, "assets/font.ttf")
    ///     .expect("font load");
    /// let weak = font.make_weak();
    /// // `weak` is now an alias with no drop responsibility — the atlas leaks unless
    /// // ownership is reclaimed before `weak` goes out of scope.
    /// ```
    ///
    /// # See also
    ///
    /// - [`WeakFont`] — the returned no-drop alias type.
    /// - [`Font`] — owning counterpart.
    #[inline]
    #[must_use]
    pub fn make_weak(self) -> WeakFont {
        let w = WeakFont(self.0);
        std::mem::forget(self);
        w
    }
    /// Returns a new `Font` using provided `GlyphInfo` data and parameters.
    fn from_data(
        chars: &[ffi::GlyphInfo],
        base_size: i32,
        padding: i32,
        pack_method: i32,
    ) -> Result<Font, LoadFontError> {
        let f = unsafe {
            let mut f = std::mem::zeroed::<Font>();
            f.baseSize = base_size;
            f.set_chars(chars);

            let atlas = ffi::GenImageFontAtlas(
                f.glyphs,
                &mut f.0.recs,
                f.baseSize,
                f.glyphCount,
                padding,
                pack_method,
            );
            f.texture = ffi::LoadTextureFromImage(atlas);
            ffi::UnloadImage(atlas);
            f
        };
        if f.0.glyphs.is_null() || f.0.texture.id == 0 {
            return Err(LoadFontError::LoadFromImageFailed);
        }
        Ok(f)
    }

    /// Sets the character data on the current Font.
    fn set_chars(&mut self, chars: &[ffi::GlyphInfo]) {
        unsafe {
            self.glyphCount = chars.len() as i32;
            let data_size = self.glyphCount as usize * std::mem::size_of::<ffi::GlyphInfo>();
            let ci_arr_ptr = ffi::MemAlloc(data_size.try_into().unwrap()); // raylib frees this data in UnloadFont
            std::ptr::copy(
                chars.as_ptr(),
                ci_arr_ptr as *mut ffi::GlyphInfo,
                chars.len(),
            );
            self.glyphs = ci_arr_ptr as *mut ffi::GlyphInfo;
        }
    }

    /// Sets the texture on the current Font, and takes ownership of `tex`.
    fn set_texture(&mut self, tex: Texture2D) {
        self.texture = tex.0;
        std::mem::forget(tex); // UnloadFont will also unload the texture
    }
}

/// Generates image font atlas using `chars` info.
/// Sets a pointer to an array of rectangles raylib allocated that MUST manually be freed.
/// Good luck freeing it safely though ;)
#[inline]
#[must_use]
pub fn gen_image_font_atlas(
    _: &RaylibThread,
    chars: &mut [ffi::GlyphInfo],
    font_size: i32,
    padding: i32,
    pack_method: i32,
) -> (Image, Vec<ffi::Rectangle>) {
    unsafe {
        let mut ptr = std::ptr::null_mut();

        let img = Image(ffi::GenImageFontAtlas(
            chars.as_mut_ptr(),
            &mut ptr,
            font_size,
            chars.len() as i32,
            padding,
            pack_method,
        ));

        let mut recs = Vec::with_capacity(chars.len());
        #[allow(clippy::uninit_vec)]
        recs.set_len(chars.len());
        std::ptr::copy(ptr, recs.as_mut_ptr(), chars.len());
        ffi::MemFree(ptr as *mut ::std::os::raw::c_void);
        (img, recs)
    }
}

impl RaylibHandle {
    /// Gets the default font.
    #[inline]
    #[must_use]
    pub fn get_font_default(&self) -> WeakFont {
        WeakFont(unsafe { ffi::GetFontDefault() })
    }
    /// Measures string width in pixels for default font.
    #[must_use]
    pub fn measure_text(&self, text: &str, font_size: i32) -> i32 {
        let c_text = CString::new(text).unwrap();
        unsafe { ffi::MeasureText(c_text.as_ptr(), font_size) }
    }

    /// Set vertical line spacing when drawing with line-breaks
    #[inline]
    pub fn set_text_line_spacing(&self, spacing: i32) {
        unsafe { ffi::SetTextLineSpacing(spacing) }
    }
}
