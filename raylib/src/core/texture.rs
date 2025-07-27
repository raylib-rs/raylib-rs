//! Image and texture related functions

use crate::MintVec2;
use crate::core::ffi::{Color, Rectangle};
use crate::core::{RaylibHandle, RaylibThread};
use crate::ffi;
use std::convert::TryInto;
use std::ffi::CString;
use std::mem::ManuallyDrop;

use super::error::{InvalidImageError, LoadTextureError, UpdateTextureError};

make_rslice!(
    /// Raylib-managed slice of [`Color`]s unloaded by [`ffi::UnloadImagePalette`]
    ImagePalette,
    Color,
    ffi::UnloadImagePalette
);
make_rslice!(
    /// Raylib-managed slice of [`Color`]s unloaded by [`ffi::UnloadImageColors`]
    ImageColors,
    Color,
    ffi::UnloadImageColors
);

/// [`NPatchInfo`], n-patch layout info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NPatchInfo {
    /// Texture source rectangle
    pub source: Rectangle,
    /// Left border offset
    pub left: i32,
    /// Top border offset
    pub top: i32,
    /// Right border offset
    pub right: i32,
    /// Bottom border offset
    pub bottom: i32,
    /// Layout of the n-patch: 3x3, 1x3 or 3x1
    pub layout: crate::consts::NPatchLayout,
}

impl From<ffi::NPatchInfo> for NPatchInfo {
    #[inline]
    fn from(v: ffi::NPatchInfo) -> NPatchInfo {
        unsafe { std::mem::transmute(v) }
    }
}

impl From<NPatchInfo> for ffi::NPatchInfo {
    #[inline]
    fn from(v: NPatchInfo) -> Self {
        unsafe { std::mem::transmute(v) }
    }
}

impl From<&NPatchInfo> for ffi::NPatchInfo {
    #[inline]
    fn from(v: &NPatchInfo) -> Self {
        unsafe { std::mem::transmute(*v) }
    }
}

fn no_drop<T>(_thing: T) {}
make_thin_wrapper!(
    /// Image, pixel data stored in CPU memory (RAM)
    Image,
    ffi::Image,
    ffi::UnloadImage
);
make_thin_wrapper!(
    /// Texture, tex data stored in GPU memory (VRAM)
    Texture2D,
    ffi::Texture2D,
    ffi::UnloadTexture
);
make_thin_wrapper!(
    /// Unowned version of [`Texture2D`] that does not free the resource when dropped
    #[derive(Default, Clone)]
    WeakTexture2D,
    ffi::Texture2D,
    no_drop
);
make_thin_wrapper!(
    /// [`RenderTexture`], fbo for texture rendering
    RenderTexture2D,
    ffi::RenderTexture2D,
    ffi::UnloadRenderTexture
);
make_thin_wrapper!(
    /// Unowned version of [`RenderTexture2D`] that does not free the resource when dropped
    #[derive(Clone)]
    WeakRenderTexture2D,
    ffi::RenderTexture2D,
    no_drop
);

impl RaylibRenderTexture2D for WeakRenderTexture2D {}
impl RaylibRenderTexture2D for RenderTexture2D {}

impl AsRef<ffi::Texture2D> for RenderTexture2D {
    #[inline]
    fn as_ref(&self) -> &ffi::Texture2D {
        self.texture()
    }
}

impl AsMut<ffi::Texture2D> for RenderTexture2D {
    #[inline]
    fn as_mut(&mut self) -> &mut ffi::Texture2D {
        self.texture_mut()
    }
}

impl AsRef<ffi::Texture2D> for WeakRenderTexture2D {
    #[inline]
    fn as_ref(&self) -> &ffi::Texture2D {
        self.texture()
    }
}

impl AsMut<ffi::Texture2D> for WeakRenderTexture2D {
    #[inline]
    fn as_mut(&mut self) -> &mut ffi::Texture2D {
        self.texture_mut()
    }
}

impl RenderTexture2D {
    /// Convert `self` to its weak form, allowing it to be shared by multiple containers on the condition
    /// that it is only unloaded once, manually.
    ///
    /// # Safety
    ///
    /// Must manually free memory by calling the proper unload function.
    /// Even if the return implements [`Copy`], exactly one instance should be unloaded to avoid double-free,
    /// and copies must not be used after being unloaded to avoid use-after-free.
    #[inline]
    #[must_use]
    pub const unsafe fn make_weak(self) -> WeakRenderTexture2D {
        let m = WeakRenderTexture2D(self.0);
        std::mem::forget(self);
        m
    }

    /// Check if a render texture is valid (loaded in GPU)
    #[inline]
    #[must_use]
    pub fn is_render_texture_valid(&self) -> bool {
        unsafe { ffi::IsRenderTextureValid(self.0) }
    }
}

/// [`RenderTexture2D`] accessors and helper methods.
pub trait RaylibRenderTexture2D {
    /// OpenGL framebuffer object id
    #[inline]
    #[must_use]
    fn id(&self) -> u32
    where
        Self: AsRef<ffi::RenderTexture2D>,
    {
        self.as_ref().id
    }

    /// Color buffer attachment texture
    #[inline]
    #[must_use]
    fn texture(&self) -> &WeakTexture2D
    where
        Self: AsRef<ffi::RenderTexture2D>,
    {
        unsafe { &*std::ptr::from_ref(&self.as_ref().texture).cast() }
    }

    /// Color buffer attachment texture
    #[inline]
    #[must_use]
    fn texture_mut(&mut self) -> &mut WeakTexture2D
    where
        Self: AsMut<ffi::RenderTexture2D>,
    {
        unsafe { &mut *std::ptr::from_mut(&mut self.as_mut().texture).cast() }
    }
}

impl Clone for Image {
    /// Create an image duplicate (useful for transformations)
    fn clone(&self) -> Image {
        let copy = unsafe { Image(ffi::ImageCopy(self.0)) };
        assert_eq!(&copy.0, &self.0, "ImageCopy failed to produce a copy");
        copy
    }
}

impl Image {
    /// Image base width
    #[inline]
    #[must_use]
    pub const fn width(&self) -> i32 {
        self.0.width
    }

    /// Image base height
    #[inline]
    #[must_use]
    pub const fn height(&self) -> i32 {
        self.0.height
    }

    /// Mipmap levels, 1 by default
    #[inline]
    #[must_use]
    pub const fn mipmaps(&self) -> i32 {
        self.0.mipmaps
    }

    /// Image raw data
    #[inline]
    #[must_use]
    pub const fn data(&self) -> *const ::std::os::raw::c_void {
        self.0.data
    }

    /// Image raw data
    #[inline]
    #[must_use]
    pub const fn data_mut(&mut self) -> *mut ::std::os::raw::c_void {
        self.0.data
    }

    /// Apply Gaussian blur using a box blur approximation
    #[inline]
    pub fn blur_gaussian(&mut self, blur_size: i32) {
        unsafe { ffi::ImageBlurGaussian(&mut self.0, blur_size) }
    }

    /// Rotate image by input angle in degrees (-359 to 359)
    #[inline]
    pub fn rotate(&mut self, degrees: i32) {
        unsafe { ffi::ImageRotate(&mut self.0, degrees) }
    }

    /// Get image pixel color at (x, y) position
    #[inline]
    #[must_use]
    pub fn get_color(&self, x: i32, y: i32) -> Color {
        unsafe { ffi::GetImageColor(self.0, x, y) }
    }

    /// Draw circle outline within an image
    #[inline]
    pub fn draw_circle_lines(&mut self, center_x: i32, center_y: i32, radius: i32, color: Color) {
        unsafe { ffi::ImageDrawCircleLines(&mut self.0, center_x, center_y, radius, color) }
    }

    /// Draw circle outline within an image (Vector version)
    #[inline]
    pub fn draw_circle_lines_v(
        &mut self,
        center: impl Into<MintVec2>,
        center_y: i32,
        color: Color,
    ) {
        unsafe { ffi::ImageDrawCircleLinesV(&mut self.0, center.into(), center_y, color) }
    }

    /// Data format ([`PixelFormat`](crate::consts::PixelFormat) type)
    #[inline]
    #[must_use]
    pub fn format(&self) -> crate::consts::PixelFormat {
        unsafe { std::mem::transmute(self.format) }
    }

    /// Create an image from another image piece
    #[inline]
    #[must_use]
    pub fn from_image(&self, rec: impl Into<ffi::Rectangle>) -> Image {
        unsafe { Image(ffi::ImageFromImage(self.0, rec.into())) }
    }

    /// Create an image from a selected channel of another image (GRAYSCALE)
    #[inline]
    #[must_use]
    pub fn from_channel(&self, selected_channel: i32) -> Image {
        unsafe { Image(ffi::ImageFromChannel(self.0, selected_channel)) }
    }

    /// Exports image as a PNG file.
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte.
    #[inline]
    pub fn export_image(&self, filename: &str) {
        let c_filename =
            CString::new(filename).expect("filename should not contain an internal 0 byte");
        unsafe {
            ffi::ExportImage(self.0, c_filename.as_ptr());
        }
    }

    /// Exports image as a PNG file.
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte.
    #[inline]
    pub fn export_image_as_code(&self, filename: &str) {
        let c_filename =
            CString::new(filename).expect("filename should not contain an internal 0 byte");
        unsafe {
            ffi::ExportImageAsCode(self.0, c_filename.as_ptr());
        }
    }

    /// Get pixel data size in bytes (image or texture)
    ///
    /// # Panics
    ///
    /// This method will panic if [`ffi::GetPixelDataSize`] returns a negative number (possibly due to overflowing).
    #[inline]
    #[must_use]
    pub fn get_pixel_data_size(&self) -> usize {
        unsafe {
            ffi::GetPixelDataSize(self.width(), self.height(), self.format() as i32)
                .try_into()
                .expect("ffi::GetPixelDataSize should return a positive number")
        }
    }

    /// Gets pixel data from `self` as an slice of [`Color`]s.
    ///
    /// # Panics
    ///
    /// This method may panic if `width` or `height` is negative.
    #[must_use]
    pub fn get_image_data(&self) -> ImageColors {
        let image_data_len = usize::try_from(self.width).expect("width should not be negative")
            * usize::try_from(self.height).expect("height should not be negative");
        let image_data = unsafe { ffi::LoadImageColors(self.0) };
        let data = unsafe { std::slice::from_raw_parts_mut(image_data.cast(), image_data_len) };
        ImageColors(ManuallyDrop::new(unsafe { Box::from_raw(data) }))
    }

    /// Gets pixel data from `self` as a Vec of Color structs.
    ///
    /// `flip` - Reverse the image vertically
    ///
    /// # Panics
    ///
    /// This method may panic if `width` or `height` is negative.
    #[must_use]
    pub fn get_image_data_u8(&self, flip: bool) -> Vec<u8> {
        let image_data_len = 4
            * usize::try_from(self.width).expect("width should not be negative")
            * usize::try_from(self.height).expect("height should not be negative");
        let mut res = Vec::with_capacity(image_data_len);
        if flip {
            for y in (0..self.height).rev() {
                for x in 0..self.width {
                    let color = self.get_color(x, y);
                    res.push(color.r);
                    res.push(color.g);
                    res.push(color.b);
                    res.push(color.a);
                }
            }
        } else {
            for y in 0..self.height {
                for x in 0..self.width {
                    let color = self.get_color(x, y);
                    res.push(color.r);
                    res.push(color.g);
                    res.push(color.b);
                    res.push(color.a);
                }
            }
        }
        res
    }

    /// Extract color palette from image to maximum size
    ///
    /// # Panics
    ///
    /// This method will panic if `max_palette_size` is greater than [`i32::MAX`]
    /// or if [`ffi::LoadImagePalette`] outputs a negative `colorCount`
    #[inline]
    #[must_use]
    pub fn extract_palette(&self, max_palette_size: u32) -> ImagePalette {
        let mut palette_len = 0;
        let image_data = unsafe {
            ffi::LoadImagePalette(
                self.0,
                max_palette_size
                    .try_into()
                    .expect("max_palette_size should not exceed i32::MAX"),
                &mut palette_len,
            )
        };
        let data = unsafe {
            std::slice::from_raw_parts_mut(
                image_data.cast(),
                palette_len
                    .try_into()
                    .expect("palette_len should not be negative"),
            )
        };
        ImagePalette(ManuallyDrop::new(unsafe { Box::from_raw(data) }))
    }

    /// Converts `self` to POT (power-of-two).
    #[inline]
    pub fn to_pot(&mut self, fill_color: impl Into<ffi::Color>) {
        unsafe {
            ffi::ImageToPOT(&mut self.0, fill_color.into());
        }
    }

    /// Converts `self` data to desired pixel format.
    #[inline]
    pub fn set_format(&mut self, new_format: crate::consts::PixelFormat) {
        unsafe {
            ffi::ImageFormat(&mut self.0, new_format as i32);
        }
    }

    /// Applies alpha mask to `self`.
    /// Alpha mask must be same size as the image. If alpha mask is not greyscale
    /// Ensure the colors are white (255, 255, 255, 255) or black (0, 0, 0, 0)
    #[inline]
    pub fn alpha_mask(&mut self, alpha_mask: &Image) {
        unsafe {
            ffi::ImageAlphaMask(&mut self.0, alpha_mask.0);
        }
    }

    /// Clears alpha channel on `self` to desired color.
    #[inline]
    pub fn alpha_clear(&mut self, color: impl Into<ffi::Color>, threshold: f32) {
        unsafe {
            ffi::ImageAlphaClear(&mut self.0, color.into(), threshold);
        }
    }

    /// Crops `self` depending on alpha value.
    #[inline]
    pub fn alpha_crop(&mut self, threshold: f32) {
        unsafe {
            ffi::ImageAlphaCrop(&mut self.0, threshold);
        }
    }

    /// Premultiplies alpha channel on `self`.
    #[inline]
    pub fn alpha_premultiply(&mut self) {
        unsafe {
            ffi::ImageAlphaPremultiply(&mut self.0);
        }
    }

    /// Crops `self` to a defined rectangle.
    #[inline]
    pub fn crop(&mut self, crop: impl Into<ffi::Rectangle>) {
        unsafe {
            ffi::ImageCrop(&mut self.0, crop.into());
        }
    }

    /// Resizes `self` (bilinear filtering).
    #[inline]
    pub fn resize(&mut self, new_width: i32, new_height: i32) {
        unsafe {
            ffi::ImageResize(&mut self.0, new_width, new_height);
        }
    }

    /// Resizes `self` (nearest-neighbor scaling).
    #[inline]
    pub fn resize_nn(&mut self, new_width: i32, new_height: i32) {
        unsafe {
            ffi::ImageResizeNN(&mut self.0, new_width, new_height);
        }
    }

    /// Resizes `self` canvas and fills with `color`.
    #[inline]
    pub fn resize_canvas(
        &mut self,
        new_width: i32,
        new_height: i32,
        offset_x: i32,
        offset_y: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::ImageResizeCanvas(
                &mut self.0,
                new_width,
                new_height,
                offset_x,
                offset_y,
                color.into(),
            );
        }
    }

    /// Generates all mipmap levels for a provided `self`.
    #[inline]
    pub fn gen_mipmaps(&mut self) {
        unsafe {
            ffi::ImageMipmaps(&mut self.0);
        }
    }

    /// Dithers `self` data to 16bpp or lower (Floyd-Steinberg dithering).
    #[inline]
    pub fn dither(&mut self, r_bpp: i32, g_bpp: i32, b_bpp: i32, a_bpp: i32) {
        unsafe {
            ffi::ImageDither(&mut self.0, r_bpp, g_bpp, b_bpp, a_bpp);
        }
    }

    /// Get image alpha border rectangle
    #[inline]
    #[must_use]
    pub fn get_image_alpha_border(&self, threshold: f32) -> Rectangle {
        unsafe { ffi::GetImageAlphaBorder(self.0, threshold) }
    }

    /// Clear image background with given color
    #[inline]
    pub fn clear_background(&mut self, color: impl Into<ffi::Color>) {
        unsafe { ffi::ImageClearBackground(&mut self.0, color.into()) }
    }

    /// Draws a source image within a destination image.
    #[inline]
    pub fn draw(
        &mut self,
        src: &Image,
        src_rec: Rectangle,
        dst_rec: Rectangle,
        tint: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::ImageDraw(&mut self.0, src.0, src_rec, dst_rec, tint.into());
        }
    }

    /// Draw pixel within an image
    #[inline]
    pub fn draw_pixel(&mut self, pos_x: i32, pos_y: i32, color: impl Into<ffi::Color>) {
        unsafe { ffi::ImageDrawPixel(&mut self.0, pos_x, pos_y, color.into()) }
    }

    /// Draw pixel within an image (Vector version)
    #[inline]
    pub fn draw_pixel_v(&mut self, position: impl Into<MintVec2>, color: impl Into<ffi::Color>) {
        unsafe { ffi::ImageDrawPixelV(&mut self.0, position.into(), color.into()) }
    }

    /// Draw line within an image
    #[inline]
    pub fn draw_line(
        &mut self,
        start_pos_x: i32,
        start_pos_y: i32,
        end_pos_x: i32,
        end_pos_y: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::ImageDrawLine(
                &mut self.0,
                start_pos_x,
                start_pos_y,
                end_pos_x,
                end_pos_y,
                color.into(),
            );
        }
    }

    /// Draw a line (using triangles/quads)
    #[inline]
    pub fn draw_line_ex(
        &mut self,
        start_pos: impl Into<MintVec2>,
        end_pos: impl Into<MintVec2>,
        thick: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::ImageDrawLineEx(
                &mut self.0,
                start_pos.into(),
                end_pos.into(),
                thick,
                color.into(),
            );
        }
    }

    /// Draw line within an image (Vector version)
    #[inline]
    pub fn draw_line_v(
        &mut self,
        start: impl Into<MintVec2>,
        end: impl Into<MintVec2>,
        color: impl Into<ffi::Color>,
    ) {
        unsafe { ffi::ImageDrawLineV(&mut self.0, start.into(), end.into(), color.into()) }
    }

    /// Draw triangle within an image
    #[inline]
    pub fn draw_triangle(
        &mut self,
        v1: impl Into<MintVec2>,
        v2: impl Into<MintVec2>,
        v3: impl Into<MintVec2>,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::ImageDrawTriangle(&mut self.0, v1.into(), v2.into(), v3.into(), color.into());
        }
    }

    /// Draw triangle with interpolated colors within an image
    #[inline]
    pub fn draw_triangle_ex(
        &mut self,
        v1: impl Into<MintVec2>,
        v2: impl Into<MintVec2>,
        v3: impl Into<MintVec2>,
        c1: impl Into<ffi::Color>,
        c2: impl Into<ffi::Color>,
        c3: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::ImageDrawTriangleEx(
                &mut self.0,
                v1.into(),
                v2.into(),
                v3.into(),
                c1.into(),
                c2.into(),
                c3.into(),
            );
        }
    }

    /// Draw triangle outline within an image
    #[inline]
    pub fn draw_triangle_lines(
        &mut self,
        v1: impl Into<MintVec2>,
        v2: impl Into<MintVec2>,
        v3: impl Into<MintVec2>,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::ImageDrawTriangleLines(&mut self.0, v1.into(), v2.into(), v3.into(), color.into());
        }
    }

    /// Draw a triangle fan defined by points within an image (first vertex is the center)
    ///
    /// # Panics
    ///
    /// This method will panic if `points` has a length greater than [`i32::MAX`].
    pub fn draw_triangle_fan(
        &mut self,
        points: &mut [crate::math::Vector2],
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::ImageDrawTriangleFan(
                &mut self.0,
                points.as_mut_ptr().cast(),
                points
                    .len()
                    .try_into()
                    .expect("points should not exceed i32::MAX elements"),
                color.into(),
            );
        }
    }

    /// Draw a triangle strip defined by points within an image
    ///
    /// # Panics
    ///
    /// This method will panic if `points` has a length greater than [`i32::MAX`].
    pub fn draw_triangle_strip(
        &mut self,
        points: &mut [crate::math::Vector2],
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::ImageDrawTriangleStrip(
                &mut self.0,
                points.as_mut_ptr().cast(),
                points
                    .len()
                    .try_into()
                    .expect("points should not exceed i32::MAX elements"),
                color.into(),
            );
        }
    }

    /// Draw circle within an image
    #[inline]
    pub fn draw_circle(
        &mut self,
        center_x: i32,
        center_y: i32,
        radius: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe { ffi::ImageDrawCircle(&mut self.0, center_x, center_y, radius, color.into()) }
    }

    /// Draw circle within an image (Vector version)
    #[inline]
    pub fn draw_circle_v(
        &mut self,
        center: impl Into<MintVec2>,
        radius: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe { ffi::ImageDrawCircleV(&mut self.0, center.into(), radius, color.into()) }
    }

    /// Draws a rectangle within an image.
    #[inline]
    pub fn draw_rectangle(
        &mut self,
        pos_x: i32,
        pos_y: i32,
        width: i32,
        height: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::ImageDrawRectangle(&mut self.0, pos_x, pos_y, width, height, color.into());
        }
    }

    /// Draw rectangle within an image (Vector version)
    #[inline]
    pub fn draw_rectangle_v(
        &mut self,
        position: impl Into<MintVec2>,
        size: impl Into<MintVec2>,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::ImageDrawRectangleV(&mut self.0, position.into(), size.into(), color.into());
        }
    }

    /// Draw rectangle within an image (Rectangle version)
    #[inline]
    pub fn draw_rectangle_rec(
        &mut self,
        rectangle: impl Into<ffi::Rectangle>,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::ImageDrawRectangleRec(&mut self.0, rectangle.into(), color.into());
        }
    }

    /// Draws a rectangle within an image.
    #[inline]
    pub fn draw_rectangle_lines(
        &mut self,
        rec: Rectangle,
        thickness: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::ImageDrawRectangleLines(&mut self.0, rec, thickness, color.into());
        }
    }

    /// Draws text (default font) within an image (destination).
    ///
    /// # Panics
    ///
    /// This method will panic if `text` contains an internal 0 byte.
    #[inline]
    pub fn draw_text(
        &mut self,
        text: &str,
        pos_x: i32,
        pos_y: i32,
        font_size: i32,
        color: impl Into<ffi::Color>,
    ) {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe {
            ffi::ImageDrawText(
                &mut self.0,
                c_text.as_ptr(),
                pos_x,
                pos_y,
                font_size,
                color.into(),
            );
        }
    }

    /// Draws text (default font) within an image (destination).
    ///
    /// # Panics
    ///
    /// This method will panic if `text` contains an internal 0 byte.
    #[inline]
    pub fn draw_text_ex(
        &mut self,
        font: impl AsRef<ffi::Font>,
        text: &str,
        position: impl Into<MintVec2>,
        font_size: f32,
        spacing: f32,
        color: impl Into<ffi::Color>,
    ) {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe {
            ffi::ImageDrawTextEx(
                &mut self.0,
                *font.as_ref(),
                c_text.as_ptr(),
                position.into(),
                font_size,
                spacing,
                color.into(),
            );
        }
    }

    /// Flips `self` vertically.
    #[inline]
    pub fn flip_vertical(&mut self) {
        unsafe {
            ffi::ImageFlipVertical(&mut self.0);
        }
    }

    /// Flips `self` horizontally.
    #[inline]
    pub fn flip_horizontal(&mut self) {
        unsafe {
            ffi::ImageFlipHorizontal(&mut self.0);
        }
    }

    /// Rotates `self` clockwise by 90 degrees (PI/2 radians).
    #[inline]
    pub fn rotate_cw(&mut self) {
        unsafe {
            ffi::ImageRotateCW(&mut self.0);
        }
    }

    /// Rotates `self` counterclockwise by 90 degrees (PI/2 radians).
    #[inline]
    pub fn rotate_ccw(&mut self) {
        unsafe {
            ffi::ImageRotateCCW(&mut self.0);
        }
    }

    /// Tints colors in `self` using specified `color`.
    #[inline]
    pub fn color_tint(&mut self, color: impl Into<ffi::Color>) {
        unsafe {
            ffi::ImageColorTint(&mut self.0, color.into());
        }
    }

    /// Inverts the colors in `self`.
    #[inline]
    pub fn color_invert(&mut self) {
        unsafe {
            ffi::ImageColorInvert(&mut self.0);
        }
    }

    /// Converts `self` color to grayscale.
    #[inline]
    pub fn color_grayscale(&mut self) {
        unsafe {
            ffi::ImageColorGrayscale(&mut self.0);
        }
    }

    /// Adjusts the contrast of `self`.
    #[inline]
    pub fn color_contrast(&mut self, contrast: f32) {
        unsafe {
            ffi::ImageColorContrast(&mut self.0, contrast);
        }
    }

    /// Adjusts the brightness of `self`.
    #[inline]
    pub fn color_brightness(&mut self, brightness: i32) {
        unsafe {
            ffi::ImageColorBrightness(&mut self.0, brightness);
        }
    }

    /// Searches `self` for all occurrences of `color` and replaces them with `replace` color.
    #[inline]
    pub fn color_replace(&mut self, color: impl Into<ffi::Color>, replace: impl Into<ffi::Color>) {
        unsafe {
            ffi::ImageColorReplace(&mut self.0, color.into(), replace.into());
        }
    }

    /// Export image to memory buffer.
    ///
    /// # Errors
    ///
    /// See [`InvalidImageError`]
    ///
    /// # Panics
    ///
    /// This method will panic if `file_type` contains an internal 0 byte or if [`ffi::ExportImageToMemory`]
    /// outputs a negative `fileSize`.
    pub fn export_image_to_memory(&self, file_type: &str) -> Result<&[u8], InvalidImageError> {
        if self.width == 0 {
            return Err(InvalidImageError::ZeroWidth);
        }
        if self.height == 0 {
            return Err(InvalidImageError::ZeroHeight);
        }
        if self.data.is_null() {
            return Err(InvalidImageError::NullData);
        }

        let c_filetype =
            CString::new(file_type).expect("file_type should not contain an internal 0 byte");
        let mut data_size = 0;
        let data =
            unsafe { ffi::ExportImageToMemory(self.0, c_filetype.as_ptr(), &raw mut data_size) };

        // The actual function returns null if the code for converting to a file type never goes off.
        if data.is_null() {
            return Err(InvalidImageError::UnsupportedFormat);
        }

        Ok(unsafe {
            std::slice::from_raw_parts(
                data.cast_const(),
                data_size
                    .try_into()
                    .expect("data_size should not be negative"),
            )
        })
    }

    /// Apply custom square convolution kernel to image
    ///
    /// NOTE: The convolution kernel matrix is expected to be square
    ///
    /// # Errors
    ///
    /// See [`InvalidImageError`]
    ///
    /// # Panics
    ///
    /// This method will panic if `kernel` has a length greater than [`i32::MAX`]
    pub fn kernel_convolution(&mut self, kernel: &[f32]) -> Result<(), InvalidImageError> {
        if self.width == 0 {
            return Err(InvalidImageError::ZeroWidth);
        }
        if self.height == 0 {
            return Err(InvalidImageError::ZeroHeight);
        }
        if self.data.is_null() {
            return Err(InvalidImageError::NullData);
        }

        let kernel_width = kernel.len().isqrt();

        if (kernel_width * kernel_width) != kernel.len() {
            return Err(InvalidImageError::NonSquareKernel);
        }

        unsafe {
            ffi::ImageKernelConvolution(
                &mut self.0,
                kernel.as_ptr(),
                kernel
                    .len()
                    .try_into()
                    .expect("kernel should not exceed i32::MAX elements"),
            );
        }

        Ok(())
    }

    /// Generates a plain `color` Image.
    #[inline]
    #[must_use]
    pub fn gen_image_color(width: i32, height: i32, color: impl Into<ffi::Color>) -> Image {
        unsafe { Image(ffi::GenImageColor(width, height, color.into())) }
    }

    /// Generate image: perlin noise
    #[inline]
    #[must_use]
    pub fn gen_image_perlin_noise(
        &self,
        width: i32,
        height: i32,
        offset_x: i32,
        offset_y: i32,
        scale: f32,
    ) -> Image {
        Image(unsafe { ffi::GenImagePerlinNoise(width, height, offset_x, offset_y, scale) })
    }

    /// Generates an Image containing a radial gradient.
    #[inline]
    #[must_use]
    pub fn gen_image_gradient_radial(
        width: i32,
        height: i32,
        density: f32,
        inner: impl Into<ffi::Color>,
        outer: impl Into<ffi::Color>,
    ) -> Image {
        unsafe {
            Image(ffi::GenImageGradientRadial(
                width,
                height,
                density,
                inner.into(),
                outer.into(),
            ))
        }
    }

    /// Generates an Image containing a checkerboard pattern.
    #[inline]
    #[must_use]
    pub fn gen_image_checked(
        width: i32,
        height: i32,
        checks_x: i32,
        checks_y: i32,
        col1: impl Into<ffi::Color>,
        col2: impl Into<ffi::Color>,
    ) -> Image {
        unsafe {
            Image(ffi::GenImageChecked(
                width,
                height,
                checks_x,
                checks_y,
                col1.into(),
                col2.into(),
            ))
        }
    }

    /// Generate images an image linear gradient.
    /// `direction` in expected to be degrees [0..360]. 0 results in a vertical gradient
    #[inline]
    #[must_use]
    pub fn gen_image_gradient_linear(
        width: i32,
        height: i32,
        direction: i32,
        start: Color,
        end: Color,
    ) -> Image {
        unsafe {
            Image(ffi::GenImageGradientLinear(
                width, height, direction, start, end,
            ))
        }
    }

    /// Generate images an image with a square gradient
    /// For best results, `density` should be `0.0..=1.0`
    #[inline]
    #[must_use]
    pub fn gen_image_gradient_square(
        width: i32,
        height: i32,
        density: f32,
        start: Color,
        end: Color,
    ) -> Image {
        unsafe {
            Image(ffi::GenImageGradientSquare(
                width, height, density, start, end,
            ))
        }
    }

    /// Generates an image with text
    ///
    /// # Panics
    ///
    /// This method will panic if `text` contains an internal 0 byte.
    #[must_use]
    pub fn gen_image_text(width: i32, height: i32, text: &str) -> Image {
        let c_str = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe { Image(ffi::GenImageText(width, height, c_str.as_ptr())) }
    }

    /// Generates an Image containing white noise.
    #[inline]
    #[must_use]
    pub fn gen_image_white_noise(width: i32, height: i32, factor: f32) -> Image {
        unsafe { Image(ffi::GenImageWhiteNoise(width, height, factor)) }
    }

    /// Generates an Image using a cellular algorithm. Bigger `tile_size` means bigger cells.
    #[inline]
    #[must_use]
    pub fn gen_image_cellular(width: i32, height: i32, tile_size: i32) -> Image {
        unsafe { Image(ffi::GenImageCellular(width, height, tile_size)) }
    }

    /// Get clipboard image.
    ///
    /// NOTE: Only available on Windows.
    ///
    /// # Errors
    ///
    /// This method returns [`InvalidImageError::NullData`] if [`ffi::GetClipboardImage()`] returns
    /// an image whose `data` is null, or if the platform is not Windows.
    pub fn get_clipboard_image(&mut self) -> Result<Image, InvalidImageError> {
        if cfg!(target_os = "windows") {
            let i = unsafe { ffi::GetClipboardImage() };
            if i.data.is_null() {
                return Err(InvalidImageError::NullData);
            }
            Ok(Image(i))
        } else {
            Err(InvalidImageError::NullData)
        }
    }

    /// Loads image from file into CPU memory (RAM).
    ///
    /// # Errors
    ///
    /// This method returns [`InvalidImageError::NullDataFromFile`] if [`ffi::LoadImage`] returns
    /// an image whose `data` is null.
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte.
    pub fn load_image(filename: &str) -> Result<Image, InvalidImageError> {
        let c_filename =
            CString::new(filename).expect("filename should not contain an internal 0 byte");
        let i = unsafe { ffi::LoadImage(c_filename.as_ptr()) };
        if i.data.is_null() {
            return Err(InvalidImageError::NullDataFromFile);
        }
        Ok(Image(i))
    }

    /// Loads image from a given memory buffer
    ///
    /// The input data is expected to be in a supported file format such as png. Which formats are
    /// supported depend on the build flags used for the raylib (C) library.
    ///
    /// # Errors
    ///
    /// This method returns [`InvalidImageError::InvalidFile`] if `bytes` is empty, or
    /// [`InvalidImageError::NullDataFromMemory`] if [`ffi::LoadImageFromMemory`] returns an image
    /// whose `data` is null.
    ///
    /// # Panics
    ///
    /// This method will panic if `filetype` contains an internal 0 byte, or if `bytes` has a length
    /// greater than [`i32::MAX`].
    pub fn load_image_from_mem(filetype: &str, bytes: &[u8]) -> Result<Image, InvalidImageError> {
        let c_filetype =
            CString::new(filetype).expect("filetype should not contain an internal 0 byte");
        let data_size = bytes
            .len()
            .try_into()
            .expect("bytes should not exceed i32::MAX elements");
        if data_size == 0 {
            return Err(InvalidImageError::InvalidFile);
        }
        let i = unsafe { ffi::LoadImageFromMemory(c_filetype.as_ptr(), bytes.as_ptr(), data_size) };
        if i.data.is_null() {
            Err(InvalidImageError::NullDataFromMemory)
        } else {
            Ok(Image(i))
        }
    }

    /// Load image sequence from file, with the number of frames loaded saved to `frame_num`.
    ///
    /// `Image.data` buffer includes all frames.
    /// All frames returned are in RGBA format.
    /// Frames delay data is discarded
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte.
    #[must_use]
    pub fn load_image_anim(filename: &str, frame_num: &mut i32) -> Self {
        let c_filename =
            CString::new(filename).expect("filename should not contain an internal 0 byte");

        unsafe { Image(ffi::LoadImageAnim(c_filename.as_ptr(), frame_num)) }
    }

    /// Load image from memory buffer, with the number of frames loaded saved to `frame_num`.
    ///
    /// `fileType` refers to extension: i.e. ".png". File extension must be provided in lower-case
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte, or if `data` has a length
    /// greater than [`i32::MAX`].
    #[must_use]
    pub fn load_image_anim_from_memory(filetype: &str, data: &[u8], frame_num: &mut i32) -> Self {
        let c_filetype =
            CString::new(filetype).expect("filename should not contain an internal 0 byte");

        unsafe {
            Image(ffi::LoadImageAnimFromMemory(
                c_filetype.as_ptr(),
                data.as_ptr(),
                data.len()
                    .try_into()
                    .expect("data should not exceed i32::MAX elements"),
                frame_num,
            ))
        }
    }

    /// Loads image from RAW file data.
    ///
    /// # Errors
    ///
    /// This method returns [`InvalidImageError::NullDataFromFile`] if [`ffi::LoadImageRaw`] returns
    /// an image whose `data` is null.
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte.
    pub fn load_image_raw(
        filename: &str,
        width: i32,
        height: i32,
        format: i32,
        header_size: i32,
    ) -> Result<Image, InvalidImageError> {
        let c_filename =
            CString::new(filename).expect("filename should not contain an internal 0 byte");
        let i =
            unsafe { ffi::LoadImageRaw(c_filename.as_ptr(), width, height, format, header_size) };
        if i.data.is_null() {
            return Err(InvalidImageError::NullDataFromFile);
        }
        Ok(Image(i))
    }

    /// Creates an image from `text` (custom font).
    ///
    /// # Panics
    ///
    /// This method will panic if `text` contains an internal 0 byte.
    #[inline]
    #[must_use]
    pub fn image_text(text: &str, font_size: i32, color: impl Into<ffi::Color>) -> Image {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe { Image(ffi::ImageText(c_text.as_ptr(), font_size, color.into())) }
    }

    /// Creates an image from `text` (custom font).
    ///
    /// # Panics
    ///
    /// This method will panic if `text` contains an internal 0 byte.
    #[inline]
    #[must_use]
    pub fn image_text_ex(
        font: impl std::convert::AsRef<ffi::Font>,
        text: &str,
        font_size: f32,
        spacing: f32,
        tint: impl Into<ffi::Color>,
    ) -> Image {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe {
            Image(ffi::ImageTextEx(
                *font.as_ref(),
                c_text.as_ptr(),
                font_size,
                spacing,
                tint.into(),
            ))
        }
    }

    /// Check if an image is valid (data and parameters)
    #[inline]
    #[must_use]
    pub fn is_image_valid(&self) -> bool {
        unsafe { ffi::IsImageValid(self.0) }
    }
}

impl RaylibTexture2D for WeakTexture2D {}
impl RaylibTexture2D for Texture2D {}
impl RaylibTexture2D for WeakRenderTexture2D {}
impl RaylibTexture2D for RenderTexture2D {}

impl Texture2D {
    /// Convert `self` to its weak form, allowing it to be shared by multiple containers on the condition
    /// that it is only unloaded once, manually.
    ///
    /// # Safety
    ///
    /// Must manually free memory by calling the proper unload function.
    /// Even if the return implements [`Copy`], exactly one instance should be unloaded to avoid double-free,
    /// and copies must not be used after being unloaded to avoid use-after-free.
    #[inline]
    #[must_use]
    pub const unsafe fn make_weak(self) -> WeakTexture2D {
        let m = WeakTexture2D(self.0);
        std::mem::forget(self);
        m
    }
}

/// [`Texture2D`] accessors and helper methods.
pub trait RaylibTexture2D {
    /// Texture base width
    #[inline]
    #[must_use]
    fn width(&self) -> i32
    where
        Self: AsRef<ffi::Texture2D>,
    {
        self.as_ref().width
    }

    /// Texture base height
    #[inline]
    #[must_use]
    fn height(&self) -> i32
    where
        Self: AsRef<ffi::Texture2D>,
    {
        self.as_ref().height
    }

    /// Mipmap levels, 1 by default
    #[inline]
    #[must_use]
    fn mipmaps(&self) -> i32
    where
        Self: AsRef<ffi::Texture2D>,
    {
        self.as_ref().mipmaps
    }

    /// Data format ([`PixelFormat`] type)
    #[inline]
    #[must_use]
    fn format(&self) -> i32
    where
        Self: AsRef<ffi::Texture2D>,
    {
        self.as_ref().format
    }

    /// Updates GPU texture with new data.
    ///
    /// # Errors
    ///
    /// This method returns [`UpdateTextureError::WrongDataSize`] if the incorrect number
    /// of bytes are provided to update the full texture.
    #[inline]
    fn update_texture(&mut self, pixels: &[u8]) -> Result<(), UpdateTextureError>
    where
        Self: AsMut<ffi::Texture2D>,
    {
        let expected_len = get_pixel_data_size(
            self.as_mut().width,
            self.as_mut().height,
            pixelformat_from_i32(self.as_mut().format).expect("unknown format"),
        )
        .try_into()
        .expect("get_pixel_data_size should not return a negative");
        if pixels.len() != expected_len {
            return Err(UpdateTextureError::WrongDataSize {
                expect: expected_len,
                actual: pixels.len(),
            });
        }
        unsafe {
            ffi::UpdateTexture(*self.as_mut(), pixels.as_ptr().cast());
        }

        Ok(())
    }

    /// Update GPU texture rectangle with new data
    ///
    /// # Errors
    ///
    /// See [`UpdateTextureError`]
    ///
    /// # Panics
    ///
    /// This method will panic if [`get_pixel_data_size`] returns a negative value.
    fn update_texture_rec(
        &mut self,
        rec: impl Into<ffi::Rectangle>,
        pixels: &[u8],
    ) -> Result<(), UpdateTextureError>
    where
        Self: AsMut<ffi::Texture2D>,
    {
        #![allow(
            clippy::cast_possible_truncation,
            reason = "truncation is intentional here"
        )]

        let rec: ffi::Rectangle = rec.into();

        if (rec.x < 0.0)
            || (rec.y < 0.0)
            || ((rec.x as i32 + rec.width as i32) > (self.as_mut().width))
            || ((rec.y as i32 + rec.height as i32) > (self.as_mut().height))
        {
            return Err(UpdateTextureError::OutOfBounds);
        }
        if (rec.width < 0.0) || (rec.height < 0.0) {
            return Err(UpdateTextureError::NegativeSize);
        }

        let expected_len = get_pixel_data_size(
            rec.width as i32,
            rec.height as i32,
            pixelformat_from_i32(self.as_mut().format).expect("unknown format"),
        )
        .try_into()
        .expect("pixel data should not be negative");
        if pixels.len() != expected_len {
            return Err(UpdateTextureError::WrongDataSize {
                expect: expected_len,
                actual: pixels.len(),
            });
        }
        unsafe {
            ffi::UpdateTextureRec(*self.as_mut(), rec, pixels.as_ptr().cast());
        }

        Ok(())
    }

    /// Gets pixel data from GPU texture and returns an `Image`.
    ///
    /// # Errors
    ///
    /// This method returns [`InvalidImageError::NullDataFromTexture`] if [`ffi::LoadImageFromTexture`]
    /// returns an image whose `data` is null.
    #[inline]
    fn load_image(&self) -> Result<Image, InvalidImageError>
    where
        Self: AsRef<ffi::Texture2D>,
    {
        let i = unsafe { ffi::LoadImageFromTexture(*self.as_ref()) };
        if i.data.is_null() {
            return Err(InvalidImageError::NullDataFromTexture);
        }
        Ok(Image(i))
    }

    /// Generates GPU mipmaps for a `texture`.
    #[inline]
    fn gen_texture_mipmaps(&mut self)
    where
        Self: AsMut<ffi::Texture2D>,
    {
        unsafe {
            ffi::GenTextureMipmaps(self.as_mut());
        }
    }

    /// Sets global `texture` scaling filter mode.
    #[inline]
    fn set_texture_filter(&self, _: &RaylibThread, filter_mode: crate::consts::TextureFilter)
    where
        Self: AsRef<ffi::Texture2D>,
    {
        unsafe {
            ffi::SetTextureFilter(*self.as_ref(), filter_mode as i32);
        }
    }

    /// Sets global texture wrapping mode.
    #[inline]
    fn set_texture_wrap(&self, _: &RaylibThread, wrap_mode: crate::consts::TextureWrap)
    where
        Self: AsRef<ffi::Texture2D>,
    {
        unsafe {
            ffi::SetTextureWrap(*self.as_ref(), wrap_mode as i32);
        }
    }

    /// Check if a texture is valid (loaded in GPU)
    #[inline]
    fn is_texture_valid(&self) -> bool
    where
        Self: AsRef<ffi::Texture2D>,
    {
        unsafe { ffi::IsTextureValid(*self.as_ref()) }
    }
}

/// Gets pixel data size in bytes (image or texture).
#[inline]
#[must_use]
pub fn get_pixel_data_size(width: i32, height: i32, format: ffi::PixelFormat) -> i32 {
    unsafe { ffi::GetPixelDataSize(width, height, format as i32) }
}

impl RaylibHandle {
    /// Loads texture from file into GPU memory (VRAM).
    ///
    /// # Errors
    ///
    /// This method returns [`LoadTextureError::TextureFromFileFailed`] if [`ffi::LoadTexture`]
    /// fails to load a texture.
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte.
    pub fn load_texture(
        &mut self,
        _: &RaylibThread,
        filename: &str,
    ) -> Result<Texture2D, LoadTextureError> {
        let c_filename =
            CString::new(filename).expect("filename should not contain an internal 0 byte");
        let t = unsafe { ffi::LoadTexture(c_filename.as_ptr()) };
        if t.id == 0 {
            return Err(LoadTextureError::TextureFromFileFailed {
                path: filename.into(),
            });
        }
        Ok(Texture2D(t))
    }

    /// Load cubemap from image, multiple image cubemap layouts supported
    ///
    /// # Errors
    ///
    /// This method returns [`LoadTextureError::CubemapFromImageFailed`] if
    /// [`ffi::LoadTextureCubemap`] fails to load a cubemap.
    pub fn load_texture_cubemap(
        &mut self,
        _: &RaylibThread,
        image: &Image,
        layout: crate::consts::CubemapLayout,
    ) -> Result<Texture2D, LoadTextureError> {
        let t = unsafe { ffi::LoadTextureCubemap(image.0, layout as i32) };
        if t.id == 0 {
            return Err(LoadTextureError::CubemapFromImageFailed);
        }
        Ok(Texture2D(t))
    }

    /// Loads texture from image data.
    ///
    /// # Errors
    ///
    /// This method returns [`LoadTextureError::InvalidData`] if `image.width` or `image.height` is 0,
    /// or [`LoadTextureError::TextureFromImageFailed`] if [`ffi::LoadTextureFromImage`] fails to load
    /// a texture.
    #[inline]
    pub fn load_texture_from_image(
        &mut self,
        _: &RaylibThread,
        image: &Image,
    ) -> Result<Texture2D, LoadTextureError> {
        if image.width == 0 || image.height == 0 {
            return Err(LoadTextureError::InvalidData);
        }
        let t = unsafe { ffi::LoadTextureFromImage(image.0) };
        if t.id == 0 {
            return Err(LoadTextureError::TextureFromImageFailed);
        }
        Ok(Texture2D(t))
    }

    /// Loads texture for rendering (framebuffer).
    ///
    /// # Errors
    ///
    /// This method returns [`LoadTextureError::CreateRenderTextureFailed`] if
    /// [`ffi::LoadRenderTexture`] fails to load a render texture.
    ///
    /// # Panics
    ///
    /// This method will panic if `width` or `height` is greater than [`i32::MAX`].
    pub fn load_render_texture(
        &mut self,
        _: &RaylibThread,
        width: u32,
        height: u32,
    ) -> Result<RenderTexture2D, LoadTextureError> {
        let t = unsafe {
            ffi::LoadRenderTexture(
                width.try_into().expect("width should not exceed i32::MAX"),
                height
                    .try_into()
                    .expect("height should not exceed i32::MAX"),
            )
        };
        if t.id == 0 {
            return Err(LoadTextureError::CreateRenderTextureFailed);
        }
        Ok(RenderTexture2D(t))
    }
}

impl RaylibHandle {
    /// Unload textures from GPU memory (VRAM)
    ///
    /// Weak `Textures` will leak memory if they are not unloaded
    ///
    /// # Safety
    ///
    /// This method frees the resource associated with `texture`.
    /// The caller must ensure that `texture` has not yet been unloaded, and that no copies
    /// of `texture` are accessed or unloaded after this method returns.
    #[inline]
    pub unsafe fn unload_texture(&mut self, _: &RaylibThread, texture: WeakTexture2D) {
        unsafe { ffi::UnloadTexture(texture.to_raw()) }
    }

    /// Unload `RenderTextures` from GPU memory (VRAM)
    ///
    /// Weak `RenderTextures` will leak memory if they are not unloaded
    ///
    /// # Safety
    ///
    /// This method frees the resource associated with `texture`.
    /// The caller must ensure that `texture` has not yet been unloaded, and that no copies
    /// of `texture` are accessed or unloaded after this method returns.
    #[inline]
    pub unsafe fn unload_render_texture(&mut self, _: &RaylibThread, texture: WeakRenderTexture2D) {
        unsafe { ffi::UnloadRenderTexture(texture.to_raw()) }
    }
}

/// Safely convert [`i32`] to [`ffi::PixelFormat`].
///
/// Returns [`None`] if `format` is not a valid discriminant of [`ffi::PixelFormat`].
#[must_use]
pub const fn pixelformat_from_i32(format: i32) -> Option<ffi::PixelFormat> {
    #[allow(clippy::enum_glob_use, reason = "variants are prefixed")]
    use ffi::PixelFormat::*;
    match format {
        1 => Some(PIXELFORMAT_UNCOMPRESSED_GRAYSCALE),
        2 => Some(PIXELFORMAT_UNCOMPRESSED_GRAY_ALPHA),
        3 => Some(PIXELFORMAT_UNCOMPRESSED_R5G6B5),
        4 => Some(PIXELFORMAT_UNCOMPRESSED_R8G8B8),
        5 => Some(PIXELFORMAT_UNCOMPRESSED_R5G5B5A1),
        6 => Some(PIXELFORMAT_UNCOMPRESSED_R4G4B4A4),
        7 => Some(PIXELFORMAT_UNCOMPRESSED_R8G8B8A8),
        8 => Some(PIXELFORMAT_UNCOMPRESSED_R32),
        9 => Some(PIXELFORMAT_UNCOMPRESSED_R32G32B32),
        10 => Some(PIXELFORMAT_UNCOMPRESSED_R32G32B32A32),
        11 => Some(PIXELFORMAT_UNCOMPRESSED_R16),
        12 => Some(PIXELFORMAT_UNCOMPRESSED_R16G16B16),
        13 => Some(PIXELFORMAT_UNCOMPRESSED_R16G16B16A16),
        14 => Some(PIXELFORMAT_COMPRESSED_DXT1_RGB),
        15 => Some(PIXELFORMAT_COMPRESSED_DXT1_RGBA),
        16 => Some(PIXELFORMAT_COMPRESSED_DXT3_RGBA),
        17 => Some(PIXELFORMAT_COMPRESSED_DXT5_RGBA),
        18 => Some(PIXELFORMAT_COMPRESSED_ETC1_RGB),
        19 => Some(PIXELFORMAT_COMPRESSED_ETC2_RGB),
        20 => Some(PIXELFORMAT_COMPRESSED_ETC2_EAC_RGBA),
        21 => Some(PIXELFORMAT_COMPRESSED_PVRT_RGB),
        22 => Some(PIXELFORMAT_COMPRESSED_PVRT_RGBA),
        23 => Some(PIXELFORMAT_COMPRESSED_ASTC_4x4_RGBA),
        24 => Some(PIXELFORMAT_COMPRESSED_ASTC_8x8_RGBA),
        _ => None,
    }
}
