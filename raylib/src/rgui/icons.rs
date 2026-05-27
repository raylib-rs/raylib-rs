use crate::ffi;
use crate::ffi::Color;
use crate::rgui::scratch::scratch_txt;
use std::ffi::CStr;

/// raygui icon controls.
pub trait RaylibGuiIcons {
    /// Get text with an icon id prepended (e.g. `#23#Save`). The returned string
    /// is copied out of raygui's internal static buffer (no leak; do not free).
    #[inline]
    fn gui_icon_text(
        &mut self,
        icon_id: crate::consts::GuiIconName,
        text: impl AsRef<str>,
    ) -> String {
        let buffer = unsafe { ffi::GuiIconText(icon_id as i32, scratch_txt(text)) };
        if buffer.is_null() {
            return String::new();
        }
        // SAFETY: GuiIconText returns a pointer to a raygui-internal static buffer
        // holding a valid C string; we copy it out and never free it.
        unsafe { CStr::from_ptr(buffer) }.to_string_lossy().into_owned()
    }
    /// Set default icon drawing size (in pixels).
    #[inline]
    fn gui_set_icon_scale(&mut self, scale: i32) {
        unsafe { ffi::GuiSetIconScale(scale) }
    }
    /// Draw an icon at a position using a pixel size.
    #[inline]
    fn gui_draw_icon(
        &mut self,
        icon_id: crate::consts::GuiIconName,
        pos_x: i32,
        pos_y: i32,
        pixel_size: i32,
        color: impl Into<Color>,
    ) {
        unsafe { ffi::GuiDrawIcon(icon_id as i32, pos_x, pos_y, pixel_size, color.into()) }
    }
    /// Get a raw pointer to raygui's internal icons data (advanced; see raygui).
    ///
    /// # Safety
    /// The pointer aliases raygui-global mutable state with no lifetime tracking;
    /// the caller must not retain it across `GuiLoadStyle`/icon mutations and must
    /// respect raygui's `RAYGUI_ICON_MAX_ICONS * RAYGUI_ICON_DATA_ELEMENTS` layout.
    #[inline]
    unsafe fn gui_get_icons_raw(&mut self) -> *mut std::os::raw::c_uint {
        unsafe { ffi::GuiGetIcons() }
    }
    /// Load a raygui icons file (.rgi); returns the raw `char**` name array.
    ///
    /// # Safety
    /// Ownership and length of the returned `char**` follow raygui's contract
    /// (RAYGUI_ICON_MAX_ICONS entries); the caller is responsible for freeing it
    /// per raygui. Prefer not to use unless porting raygui icon tooling.
    #[inline]
    unsafe fn gui_load_icons_raw(
        &mut self,
        file_name: impl AsRef<str>,
        load_icons_name: bool,
    ) -> *mut *mut std::os::raw::c_char {
        unsafe { ffi::GuiLoadIcons(scratch_txt(file_name), load_icons_name) }
    }
}
