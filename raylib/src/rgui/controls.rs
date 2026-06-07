use crate::ffi;
use crate::ffi::{Rectangle, Vector2};
use crate::rgui::scratch::{scratch_txt, scratch_txt_two};
use std::ffi::c_char;

/// raygui basic controls.
pub trait RaylibGuiControls {
    /// Label control, shows text.
    #[inline]
    fn gui_label(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>) -> bool {
        unsafe { ffi::GuiLabel(bounds.into(), scratch_txt(text)) > 0 }
    }
    /// Button control, returns true when clicked.
    #[inline]
    fn gui_button(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>) -> bool {
        unsafe { ffi::GuiButton(bounds.into(), scratch_txt(text)) > 0 }
    }
    /// Label button control, returns true when clicked.
    #[inline]
    fn gui_label_button(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>) -> bool {
        unsafe { ffi::GuiLabelButton(bounds.into(), scratch_txt(text)) > 0 }
    }
    /// Toggle Button control, returns true when active.
    #[inline]
    fn gui_toggle(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: impl AsRef<str>,
        active: &mut bool,
    ) -> bool {
        unsafe { ffi::GuiToggle(bounds.into(), scratch_txt(text), active) > 0 }
    }
    /// Toggle Group control, returns active toggle index.
    #[inline]
    fn gui_toggle_group(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: impl AsRef<str>,
        active: &mut i32,
    ) -> i32 {
        unsafe { ffi::GuiToggleGroup(bounds.into(), scratch_txt(text), active) }
    }
    /// Toggle Slider control.
    #[inline]
    fn gui_toggle_slider(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: impl AsRef<str>,
        active: &mut i32,
    ) -> bool {
        unsafe { ffi::GuiToggleSlider(bounds.into(), scratch_txt(text), active) > 0 }
    }
    /// Check Box control, returns true when active.
    #[inline]
    fn gui_check_box(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: impl AsRef<str>,
        checked: &mut bool,
    ) -> bool {
        unsafe { ffi::GuiCheckBox(bounds.into(), scratch_txt(text), checked) > 0 }
    }
    /// Combo Box control, returns selected item index.
    #[inline]
    fn gui_combo_box(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: impl AsRef<str>,
        active: &mut i32,
    ) -> i32 {
        unsafe { ffi::GuiComboBox(bounds.into(), scratch_txt(text), active) }
    }
    /// Dropdown Box control, returns selected item.
    #[inline]
    fn gui_dropdown_box(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: impl AsRef<str>,
        active: &mut i32,
        edit_mode: bool,
    ) -> bool {
        unsafe { ffi::GuiDropdownBox(bounds.into(), scratch_txt(text), active, edit_mode) > 0 }
    }
    /// Spinner control, returns selected value.
    #[inline]
    fn gui_spinner(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: impl AsRef<str>,
        value: &mut i32,
        min_value: i32,
        max_value: i32,
        edit_mode: bool,
    ) -> bool {
        debug_assert!(
            min_value <= max_value,
            "gui_spinner: min_value ({min_value}) must be <= max_value ({max_value})"
        );
        unsafe {
            ffi::GuiSpinner(
                bounds.into(),
                scratch_txt(text),
                value,
                min_value,
                max_value,
                edit_mode,
            ) > 0
        }
    }
    /// Value Box control, updates input text with numbers.
    #[inline]
    fn gui_value_box(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: impl AsRef<str>,
        value: &mut i32,
        min_value: i32,
        max_value: i32,
        edit_mode: bool,
    ) -> bool {
        debug_assert!(
            min_value <= max_value,
            "gui_value_box: min_value ({min_value}) must be <= max_value ({max_value})"
        );
        unsafe {
            ffi::GuiValueBox(
                bounds.into(),
                scratch_txt(text),
                value,
                min_value,
                max_value,
                edit_mode,
            ) > 0
        }
    }
    /// Slider control, returns selected value.
    #[inline]
    fn gui_slider(
        &mut self,
        bounds: impl Into<Rectangle>,
        text_left: impl AsRef<str>,
        text_right: impl AsRef<str>,
        value: &mut f32,
        min_value: f32,
        max_value: f32,
    ) -> bool {
        debug_assert!(
            min_value <= max_value,
            "gui_slider: min_value ({min_value}) must be <= max_value ({max_value})"
        );
        let (l, r) = scratch_txt_two(text_left, text_right);
        unsafe { ffi::GuiSlider(bounds.into(), l, r, value, min_value, max_value) > 0 }
    }
    /// Slider Bar control, returns selected value.
    #[inline]
    fn gui_slider_bar(
        &mut self,
        bounds: impl Into<Rectangle>,
        text_left: impl AsRef<str>,
        text_right: impl AsRef<str>,
        value: &mut f32,
        min_value: f32,
        max_value: f32,
    ) -> bool {
        debug_assert!(
            min_value <= max_value,
            "gui_slider_bar: min_value ({min_value}) must be <= max_value ({max_value})"
        );
        let (l, r) = scratch_txt_two(text_left, text_right);
        unsafe { ffi::GuiSliderBar(bounds.into(), l, r, value, min_value, max_value) > 0 }
    }
    /// Progress Bar control, shows current progress value.
    #[inline]
    fn gui_progress_bar(
        &mut self,
        bounds: impl Into<Rectangle>,
        text_left: impl AsRef<str>,
        text_right: impl AsRef<str>,
        value: &mut f32,
        min_value: f32,
        max_value: f32,
    ) -> bool {
        debug_assert!(
            min_value <= max_value,
            "gui_progress_bar: min_value ({min_value}) must be <= max_value ({max_value})"
        );
        let (l, r) = scratch_txt_two(text_left, text_right);
        unsafe { ffi::GuiProgressBar(bounds.into(), l, r, value, min_value, max_value) > 0 }
    }
    /// Status Bar control, shows info text.
    #[inline]
    fn gui_status_bar(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>) -> bool {
        unsafe { ffi::GuiStatusBar(bounds.into(), scratch_txt(text)) > 0 }
    }
    /// Dummy control for placeholders.
    #[inline]
    fn gui_dummy_rec(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>) -> bool {
        unsafe { ffi::GuiDummyRec(bounds.into(), scratch_txt(text)) > 0 }
    }
    /// Grid control. Returns `(result, mouse_cell)`.
    #[inline]
    fn gui_grid(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: impl AsRef<str>,
        spacing: f32,
        subdivs: i32,
    ) -> (bool, Vector2) {
        let mut mouse_cell = Vector2 { x: 0.0, y: 0.0 };
        let r = unsafe {
            ffi::GuiGrid(
                bounds.into(),
                scratch_txt(text),
                spacing,
                subdivs,
                &mut mouse_cell,
            ) > 0
        };
        (r, mouse_cell)
    }

    // --- editable-buffer controls: keep &mut String (NOT scratch) ---

    /// Text Box control, edits `buffer` in place. The String's spare capacity is
    /// used as the edit buffer; reserve enough before calling for expected input.
    #[inline]
    fn gui_text_box(
        &mut self,
        bounds: impl Into<Rectangle>,
        buffer: &mut String,
        edit_mode: bool,
    ) -> bool {
        // GuiTextBox honors the `cap` (textSize) we pass, so the caller's reserved
        // capacity bounds editing; no extra minimum is required (0).
        gui_edit_string(buffer, 0, |ptr, cap| unsafe {
            ffi::GuiTextBox(bounds.into(), ptr, cap, edit_mode) > 0
        })
    }
    /// Value box control for float values; `text_value` is the editable text repr.
    #[inline]
    fn gui_value_box_float(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: impl AsRef<str>,
        text_value: &mut String,
        value: &mut f32,
        edit_mode: bool,
    ) -> bool {
        let label = scratch_txt(text);
        // GuiValueBoxFloat ignores any size and writes up to RAYGUI_VALUEBOX_MAX_CHARS
        // (+ NUL), so the buffer MUST be at least that large to avoid an overrun.
        gui_edit_string(
            text_value,
            RAYGUI_VALUEBOX_MAX_CHARS + 1,
            |ptr, _cap| unsafe {
                ffi::GuiValueBoxFloat(bounds.into(), label, ptr, value, edit_mode) > 0
            },
        )
    }
}

/// Largest buffer `GuiValueBoxFloat` may write: it takes no size argument and is
/// bounded internally by raygui's `RAYGUI_VALUEBOX_MAX_CHARS`, so callers must
/// supply at least this many bytes + 1 for the NUL or C overruns the buffer.
pub(crate) const RAYGUI_VALUEBOX_MAX_CHARS: usize = 32;

/// Shared helper for the `&mut String` editable-buffer controls. Guarantees the
/// backing buffer holds at least `min_capacity` bytes (and room for a NUL) and
/// that its ENTIRE capacity is initialized, so C may edit in place up to that
/// size and the result can be scanned back as bytes soundly. Hands C the pointer
/// and capacity via `call`, then truncates the String to the resulting C string.
/// (Extracted from the duplicated logic in `gui_text_box` / `gui_text_input_box`.)
pub(crate) fn gui_edit_string(
    buffer: &mut String,
    min_capacity: usize,
    call: impl FnOnce(*mut c_char, i32) -> bool,
) -> bool {
    // Ensure capacity >= max(min_capacity, len + 1): room for the content plus a
    // trailing NUL, and enough for controls (e.g. GuiValueBoxFloat) that write a
    // fixed amount regardless of the size we pass.
    let needed = min_capacity.max(buffer.len() + 1);
    if buffer.capacity() < needed {
        buffer.reserve(needed - buffer.len());
    }
    let capacity = buffer.capacity();
    // SAFETY: zero-fill the spare capacity (NUL is a valid UTF-8 byte) and grow the
    // length to the full capacity, so [0..capacity] is entirely initialized and the
    // String stays valid UTF-8 (original text followed by NUL bytes). This makes the
    // post-call byte scan over the whole buffer sound; C then edits in place.
    unsafe {
        let v = buffer.as_mut_vec();
        for slot in v.spare_capacity_mut() {
            slot.write(0);
        }
        v.set_len(capacity);
    }
    let ptr = buffer.as_mut_ptr() as *mut c_char;
    let res = call(ptr, capacity as i32);
    // SAFETY: every byte in [0..capacity] was initialized above (and C only writes
    // initialized bytes over it), so this shared byte view is sound.
    let scanned = unsafe { std::slice::from_raw_parts(buffer.as_ptr(), capacity) };
    let len = scanned.iter().position(|&b| b == 0).unwrap_or(capacity);
    // SAFETY: `len <= capacity`; bytes [0..len] contain no NUL and are valid UTF-8
    // as written originally or by raygui's (UTF-8 codepoint) text editing.
    unsafe { buffer.as_mut_vec().set_len(len) };
    res
}
