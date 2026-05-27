use crate::ffi;
use crate::ffi::{Color, Rectangle, Vector3};
use crate::rgui::controls::gui_edit_string;
use crate::rgui::scratch::{scratch_txt, scratch_txt_slice, scratch_txt_three};

/// raygui advanced controls (lists, dialogs, color controls).
pub trait RaylibGuiAdvanced {
    /// List View control, returns selected list item index.
    #[inline]
    fn gui_list_view(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: impl AsRef<str>,
        scroll_index: &mut i32,
        active: &mut i32,
    ) -> i32 {
        unsafe { ffi::GuiListView(bounds.into(), scratch_txt(text), scroll_index, active) }
    }
    /// List View with an explicit list of items.
    #[inline]
    fn gui_list_view_ex(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: &[impl AsRef<str>],
        focus: &mut i32,
        scroll_index: &mut i32,
        active: &mut i32,
    ) -> i32 {
        let mut ptrs = scratch_txt_slice(text);
        unsafe {
            ffi::GuiListViewEx(
                bounds.into(),
                ptrs.as_mut_ptr(),
                ptrs.len() as i32,
                focus,
                scroll_index,
                active,
            )
        }
    }
    /// Message Box control, displays a message. Returns clicked button index.
    #[inline]
    fn gui_message_box(
        &mut self,
        bounds: impl Into<Rectangle>,
        title: impl AsRef<str>,
        message: impl AsRef<str>,
        buttons: impl AsRef<str>,
    ) -> i32 {
        // All three pointers must stay valid across the C call; use scratch_txt_three
        // to push all strings into the buffer before computing any pointer.
        let (t, m, b) = scratch_txt_three(title, message, buttons);
        unsafe { ffi::GuiMessageBox(bounds.into(), t, m, b) }
    }
    /// Text Input Box control, asks for text. Returns clicked button index.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn gui_text_input_box(
        &mut self,
        bounds: impl Into<Rectangle>,
        title: impl AsRef<str>,
        message: impl AsRef<str>,
        buttons: impl AsRef<str>,
        text: &mut String,
        text_max_size: i32,
        secret_view_active: &mut bool,
    ) -> i32 {
        text.reserve(text_max_size as usize);
        // title/message/buttons must all stay valid across the C call.
        let (t, m, b) = scratch_txt_three(title, message, buttons);
        let mut result = 0;
        gui_edit_string(text, |ptr, cap| unsafe {
            result = ffi::GuiTextInputBox(bounds.into(), t, m, b, ptr, cap, secret_view_active);
            true
        });
        result
    }
    /// Color Picker control.
    #[inline]
    fn gui_color_picker(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: impl AsRef<str>,
        color: &mut Color,
    ) -> i32 {
        unsafe { ffi::GuiColorPicker(bounds.into(), scratch_txt(text), color) }
    }
    /// Color Panel control.
    #[inline]
    fn gui_color_panel(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: impl AsRef<str>,
        color: &mut Color,
    ) -> i32 {
        unsafe { ffi::GuiColorPanel(bounds.into(), scratch_txt(text), color) }
    }
    /// Color Picker control (HSV colorspace). Avoids RGB conversion on each call.
    #[inline]
    fn gui_color_picker_hsv(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: impl AsRef<str>,
        color_hsv: &mut Vector3,
    ) -> i32 {
        unsafe { ffi::GuiColorPickerHSV(bounds.into(), scratch_txt(text), color_hsv) }
    }
    /// Color Panel control (HSV colorspace).
    #[inline]
    fn gui_color_panel_hsv(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: impl AsRef<str>,
        color_hsv: &mut Vector3,
    ) -> i32 {
        unsafe { ffi::GuiColorPanelHSV(bounds.into(), scratch_txt(text), color_hsv) }
    }
    /// Color Bar Alpha control. Returns alpha value normalized [0..1].
    #[inline]
    fn gui_color_bar_alpha(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: impl AsRef<str>,
        alpha: &mut f32,
    ) -> bool {
        unsafe { ffi::GuiColorBarAlpha(bounds.into(), scratch_txt(text), alpha) > 0 }
    }
    /// Color Bar Hue control.
    #[inline]
    fn gui_color_bar_hue(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: impl AsRef<str>,
        value: &mut f32,
    ) -> bool {
        unsafe { ffi::GuiColorBarHue(bounds.into(), scratch_txt(text), value) > 0 }
    }
}
