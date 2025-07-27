use crate::core::RaylibHandle;
use crate::core::drawing::RaylibDraw;
use crate::core::text::WeakFont;
use crate::ffi::{Color, Rectangle, Vector2};
use crate::{MintVec2, ffi};

use std::ffi::{CStr, CString, c_char};

/// Global gui modification functions
impl RaylibHandle {
    /// Enable gui controls (global state)
    #[inline]
    pub fn gui_enable(&mut self) {
        unsafe { ffi::GuiEnable() }
    }
    /// Disable gui controls (global state)
    #[inline]
    pub fn gui_disable(&mut self) {
        unsafe { ffi::GuiDisable() }
    }
    /// Lock gui controls (global state)
    #[inline]
    pub fn gui_lock(&mut self) {
        unsafe { ffi::GuiLock() }
    }
    /// Unlock gui controls (global state)
    #[inline]
    pub fn gui_unlock(&mut self) {
        unsafe { ffi::GuiUnlock() }
    }
    /// Set gui controls alpha (global state), alpha goes from 0.0f to 1.0f
    #[inline]
    pub fn gui_fade(&mut self, color: Color, alpha: f32) -> Color {
        unsafe { ffi::Fade(color, alpha) }
    }
    /// Set gui state (global state)
    #[inline]
    pub fn gui_set_state(&mut self, state: crate::consts::GuiState) {
        unsafe { ffi::GuiSetState(state as i32) }
    }
    /// Get gui state (global state)
    #[inline]
    pub fn gui_get_state(&mut self) -> crate::consts::GuiState {
        let state = unsafe { ffi::GuiGetState() };
        unsafe { std::mem::transmute(state) }
    }
    /// Set gui custom font (global state)
    #[inline]
    pub fn gui_set_font(&mut self, font: impl AsRef<ffi::Font>) {
        unsafe { ffi::GuiSetFont(*font.as_ref()) }
    }
    /// Get gui custom font (global state)
    #[inline]
    pub fn gui_get_font(&mut self) -> WeakFont {
        unsafe { WeakFont(ffi::GuiGetFont()) }
    }
    /// Set one style property
    #[inline]
    pub fn gui_set_style(
        &mut self,
        control: crate::consts::GuiControl,
        property: impl GuiProperty,
        value: i32,
    ) {
        unsafe { ffi::GuiSetStyle(control as i32, property.as_i32(), value) }
    }

    /// Get one style property
    #[inline]
    pub fn gui_get_style(
        &mut self,
        control: crate::consts::GuiControl,
        property: impl GuiProperty,
    ) -> i32 {
        unsafe { ffi::GuiGetStyle(control as i32, property.as_i32()) }
    }
    /// Load style file (.rgs)
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte.
    #[inline]
    pub fn gui_load_style(&mut self, filename: &str) {
        let c_filename =
            CString::new(filename).expect("filename should not contain an internal 0 byte");
        unsafe { ffi::GuiLoadStyle(c_filename.as_ptr()) }
    }
    /// Load style default over global style
    #[inline]
    pub fn gui_load_style_default(&mut self) {
        unsafe { ffi::GuiLoadStyleDefault() }
    }

    /// Enable gui tooltips (global state)
    #[inline]
    pub fn gui_enable_tooltip(&mut self) {
        unsafe { ffi::GuiEnableTooltip() };
    }

    /// Disable gui tooltips (global state)
    #[inline]
    pub fn gui_disable_tooltip(&mut self) {
        unsafe { ffi::GuiDisableTooltip() };
    }

    /// Set tooltip string
    ///
    /// # Panics
    ///
    /// This method will panic if `tooltip` contains an internal 0 byte.
    #[inline]
    pub fn gui_set_tooltip(&mut self, tooltip: &str) {
        let c_text = CString::new(tooltip).expect("tooltip should not contain an internal 0 byte");
        unsafe {
            ffi::GuiSetTooltip(c_text.as_ptr());
        }
    }
}

unsafe impl<D: RaylibDraw> RaylibDrawGui for D {}

/// Types through which it is safe to call Raylib GUI drawing functions.
///
/// # Safety
///
/// The default implementation of the draw functions provided by this trait (which generally should never be overridden)
/// access global static memory without locking, perform calls with function pointers that may not have been loaded,
/// and expect for there to be a window, buffer, & projection matrix to target--all without any checks.
///
/// Implementors must guarantee that their type can only exist if Raylib has been successfully initialized with a window,
/// has successfully loaded any necessary GL libraries, the calling thread is the same one that initialized Raylib, and
/// [`ffi::BeginDrawing`] has been called this frame without the corresponding [`ffi::EndDrawing`] having been called yet.
pub unsafe trait RaylibDrawGui {
    /// Enable gui controls (global state)
    #[inline]
    fn gui_enable(&mut self) {
        unsafe { ffi::GuiEnable() }
    }
    /// Disable gui controls (global state)
    #[inline]
    fn gui_disable(&mut self) {
        unsafe { ffi::GuiDisable() }
    }
    /// Lock gui controls (global state)
    #[inline]
    fn gui_lock(&mut self) {
        unsafe { ffi::GuiLock() }
    }
    /// Unlock gui controls (global state)
    #[inline]
    fn gui_unlock(&mut self) {
        unsafe { ffi::GuiUnlock() }
    }

    /// Check if gui is locked (global state)
    #[inline]
    fn gui_is_locked(&mut self) -> bool {
        unsafe { ffi::GuiIsLocked() }
    }

    /// Set gui controls alpha (global state), alpha goes from 0.0 to 1.0
    #[inline]
    fn gui_fade(&mut self, color: Color, alpha: f32) -> Color {
        unsafe { ffi::Fade(color, alpha) }
    }
    /// Set gui state (global state)
    #[inline]
    fn gui_set_state(&mut self, state: crate::consts::GuiState) {
        unsafe { ffi::GuiSetState(state as i32) }
    }
    /// Get gui state (global state)
    #[inline]
    fn gui_get_state(&mut self) -> crate::consts::GuiState {
        let state = unsafe { ffi::GuiGetState() };
        unsafe { std::mem::transmute(state) }
    }
    /// Set gui custom font (global state)
    #[inline]
    fn gui_set_font(&mut self, font: impl AsRef<ffi::Font>) {
        unsafe { ffi::GuiSetFont(*font.as_ref()) }
    }
    /// Get gui custom font (global state)
    #[inline]
    fn gui_get_font(&mut self) -> WeakFont {
        unsafe { WeakFont(ffi::GuiGetFont()) }
    }
    /// Set one style property
    #[inline]
    fn gui_set_style(
        &mut self,
        control: crate::consts::GuiControl,
        property: impl GuiProperty,
        value: i32,
    ) {
        unsafe { ffi::GuiSetStyle(control as i32, property.as_i32(), value) }
    }

    /// Set gui controls alpha (global state), alpha goes from 0.0f to 1.0f
    fn gui_set_alpha(&mut self, alpha: f32) {
        unsafe {
            ffi::GuiSetAlpha(alpha);
        }
    }

    /// Get one style property
    #[inline]
    fn gui_get_style(&self, control: crate::consts::GuiControl, property: impl GuiProperty) -> i32 {
        unsafe { ffi::GuiGetStyle(control as i32, property.as_i32()) }
    }
    /// Load style file (.rgs)
    #[inline]
    fn gui_load_style(&mut self, filename: &str) {
        let c_filename =
            CString::new(filename).expect("filename should not contain an internal 0 byte");
        unsafe { ffi::GuiLoadStyle(c_filename.as_ptr()) }
    }
    /// Load style default over global style
    #[inline]
    fn gui_load_style_default(&mut self) {
        unsafe { ffi::GuiLoadStyleDefault() }
    }
    /// Window Box control, shows a window that can be closed
    #[inline]
    fn gui_window_box(&mut self, bounds: impl Into<ffi::Rectangle>, title: &str) -> bool {
        let c_filename = CString::new(title).expect("title should not contain an internal 0 byte");
        unsafe { ffi::GuiWindowBox(bounds.into(), c_filename.as_ptr()) > 0 }
    }
    /// Group Box control with text name
    #[inline]
    fn gui_group_box(&mut self, bounds: impl Into<ffi::Rectangle>, text: &str) -> bool {
        let c_filename = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe { ffi::GuiGroupBox(bounds.into(), c_filename.as_ptr()) > 0 }
    }
    /// Line separator control, could contain text
    #[inline]
    fn gui_line(&mut self, bounds: impl Into<ffi::Rectangle>, text: &str) -> bool {
        let c_filename = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe { ffi::GuiLine(bounds.into(), c_filename.as_ptr()) > 0 }
    }
    /// Panel control, useful to group controls
    #[inline]
    fn gui_panel(&mut self, bounds: impl Into<ffi::Rectangle>, text: &str) -> bool {
        let cstr: CString;
        let c_text = if text.is_empty() {
            std::ptr::null()
        } else {
            cstr = CString::new(text).expect("text should not contain an internal 0 byte");
            cstr.as_ptr()
        };
        unsafe { ffi::GuiPanel(bounds.into(), c_text) > 0 }
    }
    /// Scroll Panel control
    #[inline]
    fn gui_scroll_panel(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text: &str,
        content: impl Into<ffi::Rectangle>,
        scroll: impl Into<MintVec2>,
        view: impl Into<ffi::Rectangle>,
    ) -> (bool, Rectangle, Vector2) {
        let mut scroll = scroll.into();
        let mut view = view.into();
        let c_filename = CString::new(text).expect("text should not contain an internal 0 byte");
        let result = unsafe {
            ffi::GuiScrollPanel(
                bounds.into(),
                c_filename.as_ptr(),
                content.into(),
                &mut scroll,
                &mut view,
            )
        };
        (result > 0, view, scroll)
    }
    /// Label control, shows text
    #[inline]
    fn gui_label(&mut self, bounds: impl Into<ffi::Rectangle>, text: &str) -> bool {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe { ffi::GuiLabel(bounds.into(), c_text.as_ptr()) > 0 }
    }
    /// Button control, returns true when clicked
    #[inline]
    fn gui_button(&mut self, bounds: impl Into<ffi::Rectangle>, text: &str) -> bool {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe { ffi::GuiButton(bounds.into(), c_text.as_ptr()) > 0 }
    }
    /// Label button control, show true when clicked
    #[inline]
    fn gui_label_button(&mut self, bounds: impl Into<ffi::Rectangle>, text: &str) -> bool {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe { ffi::GuiLabelButton(bounds.into(), c_text.as_ptr()) > 0 }
    }
    /// Toggle Button control, returns true when active
    #[inline]
    fn gui_toggle(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text: &str,
        active: &mut bool,
    ) -> bool {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe { ffi::GuiToggle(bounds.into(), c_text.as_ptr(), active) > 0 }
    }
    /// Toggle Group control, returns active toggle index
    #[inline]
    fn gui_toggle_group(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text: &str,
        active: &mut i32,
    ) -> i32 {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe { ffi::GuiToggleGroup(bounds.into(), c_text.as_ptr(), active) }
    }
    /// Check Box control, returns true when active
    #[inline]
    fn gui_check_box(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text: &str,
        checked: &mut bool,
    ) -> bool {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe { ffi::GuiCheckBox(bounds.into(), c_text.as_ptr(), checked) > 0 }
    }
    /// Combo Box control, returns selected item index
    #[inline]
    fn gui_combo_box(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text: &str,
        active: &mut i32,
    ) -> i32 {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe { ffi::GuiComboBox(bounds.into(), c_text.as_ptr(), active) }
    }
    /// Dropdown Box control, returns selected item
    #[inline]
    fn gui_dropdown_box(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text: &str,
        active: &mut i32,
        edit_mode: bool,
    ) -> bool {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe { ffi::GuiDropdownBox(bounds.into(), c_text.as_ptr(), active, edit_mode) > 0 }
    }
    /// Spinner control, returns selected value
    #[inline]
    fn gui_spinner(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text: &str,
        value: &mut i32,
        min_value: i32,
        max_value: i32,
        edit_mode: bool,
    ) -> bool {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe {
            ffi::GuiSpinner(
                bounds.into(),
                // text.map(CStr::as_ptr).unwrap_or(crate::rstr!("").as_ptr()),
                c_text.as_ptr(),
                value,
                min_value,
                max_value,
                edit_mode,
            ) > 0
        }
    }
    /// Value Box control, updates input text with numbers
    #[inline]
    fn gui_value_box(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text: &str,
        value: &mut i32,
        min_value: i32,
        max_value: i32,
        edit_mode: bool,
    ) -> bool {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe {
            ffi::GuiValueBox(
                bounds.into(),
                c_text.as_ptr(),
                value,
                min_value,
                max_value,
                edit_mode,
            ) > 0
        }
    }
    /// Text Box control, updates input text
    /// Use at your own risk!!! The allocated vector MUST have enough space for edits.
    #[inline]
    fn gui_text_box(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        buffer: &mut String,
        edit_mode: bool,
    ) -> bool {
        // NOTE: method of dealing with null terminated strings copied from imgui(input_widget.rs:289, build func)
        buffer.push('\0');
        let (ptr, capacity) = (buffer.as_mut_ptr(), buffer.capacity());
        let res = unsafe {
            ffi::GuiTextBox(
                bounds.into(),
                ptr.cast(),
                capacity
                    .try_into()
                    .expect("capacity should not exceed i32::MAX"),
                edit_mode,
            ) > 0
        };
        let cap = buffer.capacity();

        // SAFETY: this slice is simply a view into the underlying buffer
        // of a String. We MAY be holding onto a view of uninitialized memory,
        // however, since we're holding this as a u8 slice, I think it should be
        // alright...
        // additionally, we can go over the bytes directly, rather than char indices,
        // because NUL will never appear in any UTF8 outside the NUL character (ie, within
        // a char).
        let buf = unsafe { std::slice::from_raw_parts(buffer.as_ptr(), cap) };
        if let Some(len) = buf.iter().position(|x| *x == b'\0') {
            // `len` is the position of the first `\0` byte in the String
            unsafe {
                buffer.as_mut_vec().set_len(len);
            }
        } else {
            // There is no null terminator, the best we can do is to not
            // update the string length.
        }
        res
    }

    /// Slider control, returns selected value
    #[inline]
    fn gui_slider(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text_left: &str,
        text_right: &str,
        value: &mut f32,
        min_value: f32,
        max_value: f32,
    ) -> bool {
        let c_text_left =
            CString::new(text_left).expect("text_left should not contain an internal 0 byte");
        let c_text_right =
            CString::new(text_right).expect("text_right should not contain an internal 0 byte");
        unsafe {
            ffi::GuiSlider(
                bounds.into(),
                c_text_left.as_ptr(),
                c_text_right.as_ptr(),
                value,
                min_value,
                max_value,
            ) > 0
        }
    }
    /// Slider Bar control, returns selected value
    #[inline]
    fn gui_slider_bar(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text_left: &str,
        text_right: &str,
        value: &mut f32,
        min_value: f32,
        max_value: f32,
    ) -> bool {
        let c_text_left =
            CString::new(text_left).expect("text_left should not contain an internal 0 byte");
        let c_text_right =
            CString::new(text_right).expect("text_right should not contain an internal 0 byte");
        unsafe {
            ffi::GuiSliderBar(
                bounds.into(),
                c_text_left.as_ptr(),
                c_text_right.as_ptr(),
                value,
                min_value,
                max_value,
            ) > 0
        }
    }
    /// Progress Bar control, shows current progress value
    #[inline]
    fn gui_progress_bar(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text_left: &str,
        text_right: &str,
        value: &mut f32,
        min_value: f32,
        max_value: f32,
    ) -> bool {
        let c_text_left =
            CString::new(text_left).expect("text_left should not contain an internal 0 byte");
        let c_text_right =
            CString::new(text_right).expect("text_right should not contain an internal 0 byte");
        unsafe {
            ffi::GuiProgressBar(
                bounds.into(),
                c_text_left.as_ptr(),
                c_text_right.as_ptr(),
                value,
                min_value,
                max_value,
            ) > 0
        }
    }
    /// Status Bar control, shows info text
    #[inline]
    fn gui_status_bar(&mut self, bounds: impl Into<ffi::Rectangle>, text: &str) -> bool {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe { ffi::GuiStatusBar(bounds.into(), c_text.as_ptr()) > 0 }
    }

    /// Grid control
    #[inline]
    fn gui_grid(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text: &str,
        spacing: f32,
        subdivs: i32,
    ) -> (bool, Vector2) {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        let mut mouse_cell = MintVec2 { x: 0.0, y: 0.0 };
        (
            unsafe {
                ffi::GuiGrid(
                    bounds.into(),
                    c_text.as_ptr(),
                    spacing,
                    subdivs,
                    &mut mouse_cell,
                ) > 0
            },
            mouse_cell,
        )
    }
    /// List View control, returns selected list item index
    #[inline]
    fn gui_list_view(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text: &str,
        scroll_index: &mut i32,
        active: &mut i32,
    ) -> i32 {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        unsafe { ffi::GuiListView(bounds.into(), c_text.as_ptr(), scroll_index, active) }
    }
    /// List View with extended parameters
    #[inline]
    fn gui_list_view_ex(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text: impl Iterator<Item = impl AsRef<str>>,
        focus: &mut i32,
        scroll_index: &mut i32,
        active: &mut i32,
    ) -> i32 {
        // We need to keep track of all CStr buffers.
        let buffer: Box<[Box<CStr>]> = text
            .map(|s| {
                CString::new(s.as_ref())
                    .expect("text should not contain an internal 0 byte")
                    .into_boxed_c_str()
            })
            .collect();

        let mut text_params: Box<[*const c_char]> =
            buffer.iter().map(|cstr| cstr.as_ptr()).collect();

        unsafe {
            ffi::GuiListViewEx(
                bounds.into(),
                text_params.as_mut_ptr(),
                text_params
                    .len()
                    .try_into()
                    .expect("text_params should not exceed i32::MAX elements"),
                focus,
                scroll_index,
                active,
            )
        }
    }
    /// Message Box control, displays a message
    #[inline]
    fn gui_message_box(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text: &str,
        message: &str,
        buttons: &str,
    ) -> i32 {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        let c_message =
            CString::new(message).expect("message should not contain an internal 0 byte");
        let c_buttons =
            CString::new(buttons).expect("buttons should not contain an internal 0 byte");
        unsafe {
            ffi::GuiMessageBox(
                bounds.into(),
                c_text.as_ptr(),
                c_message.as_ptr(),
                c_buttons.as_ptr(),
            )
        }
    }
    /// Text Input Box control, ask for text
    #[inline]
    #[allow(clippy::too_many_arguments, reason = "all are needed")]
    fn gui_text_input_box(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        title: &str,
        message: &str,
        buttons: &str,
        text: &mut String,
        text_max_size: i32,
        secret_view_active: &mut bool,
    ) -> i32 {
        text.reserve(
            text_max_size
                .try_into()
                .expect("text_max_size should not be negative"),
        );
        // NOTE: method of dealing with null terminated strings copied from imgui(input_widget.rs:289, build func)
        text.push('\0');
        let (ptr, capacity) = (text.as_mut_ptr(), text.capacity());

        let c_title = CString::new(title).expect("title should not contain an internal 0 byte");
        let c_message =
            CString::new(message).expect("message should not contain an internal 0 byte");
        let c_buttons =
            CString::new(buttons).expect("buttons should not contain an internal 0 byte");
        let btn_index = unsafe {
            ffi::GuiTextInputBox(
                bounds.into(),
                c_title.as_ptr(),
                c_message.as_ptr(),
                c_buttons.as_ptr(),
                ptr.cast(),
                capacity
                    .try_into()
                    .expect("capacity should not exceed i32::MAX"),
                secret_view_active,
            )
        };
        let cap = text.capacity();

        // SAFETY: this slice is simply a view into the underlying buffer
        // of a String. We MAY be holding onto a view of uninitialized memory,
        // however, since we're holding this as a u8 slice, I think it should be
        // alright...
        // additionally, we can go over the bytes directly, rather than char indices,
        // because NUL will never appear in any UTF8 outside the NUL character (ie, within
        // a char).
        let buf = unsafe { std::slice::from_raw_parts(text.as_ptr(), cap) };
        if let Some(len) = buf.iter().position(|x| *x == b'\0') {
            // `len` is the position of the first `\0` byte in the String
            unsafe {
                text.as_mut_vec().set_len(len);
            }
        } else {
            // There is no null terminator, the best we can do is to not
            // update the string length.
        }
        btn_index
    }

    /// Color Picker control
    #[inline]
    fn gui_color_picker(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text: &str,
        color: &mut Color,
    ) -> i32 {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");

        unsafe { ffi::GuiColorPicker(bounds.into(), c_text.as_ptr(), &mut *color) }
    }
    /// Get text with icon id prepended
    ///
    /// NOTE: Useful to add icons by name id (enum) instead of
    /// a number that can change between ricon versions
    #[inline]
    fn gui_icon_text(&mut self, icon_id: crate::consts::GuiIconName, text: &str) -> String {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
        // GuiIconText returns a pointer to a static buffer that can be copied to an owned string, which will not leak memory.
        let buffer = unsafe { ffi::GuiIconText(icon_id as i32, c_text.as_ptr()) };
        if buffer.is_null() {
            let ptr = c_text.as_ptr();
            if ptr.is_null() {
                return String::default();
            }
            return unsafe { CStr::from_ptr(ptr).to_string_lossy().to_string() };
        }
        let c_str = unsafe { CStr::from_ptr(buffer) };
        let str_slice = c_str.to_str().unwrap_or("");
        str_slice.to_owned()
    }

    /// Color Bar Alpha control
    /// NOTE: Returns alpha value normalized [0..1]
    #[inline]
    fn gui_color_bar_alpha(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text: &str,
        alpha: &mut f32,
    ) -> bool {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");

        unsafe { ffi::GuiColorBarAlpha(bounds.into(), c_text.as_ptr(), alpha) > 0 }
    }

    /// Toggle Slider control
    #[inline]
    fn gui_toggle_slider(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text: &str,
        active: &mut i32,
    ) -> bool {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");

        unsafe { ffi::GuiToggleSlider(bounds.into(), c_text.as_ptr(), active) > 0 }
    }

    /// Dummy control for placeholders
    #[inline]
    fn gui_dummy_rec(&mut self, bounds: impl Into<ffi::Rectangle>, text: &str) -> bool {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");

        unsafe { ffi::GuiDummyRec(bounds.into(), c_text.as_ptr()) > 0 }
    }

    /// Color Bar Hue control
    #[inline]
    fn gui_color_bar_hue(
        &mut self,
        bounds: impl Into<ffi::Rectangle>,
        text: &str,
        value: &mut f32,
    ) -> bool {
        let c_text = CString::new(text).expect("text should not contain an internal 0 byte");

        unsafe { ffi::GuiColorBarHue(bounds.into(), c_text.as_ptr(), value) > 0 }
    }
}

/// Lossy conversion to [`i32`]
#[diagnostic::on_unimplemented(
    message = "{Self} is not a gui property, or does not implement the GuiProperty trait.",
    note = "As of Raylib 5.5, raygui functions that once took \"property enum as i32\" now just take the enum."
)]
pub trait GuiProperty {
    /// Convert to [`i32`] using the `as` keyword
    #[allow(
        clippy::wrong_self_convention,
        reason = "the 'as_' prefix in this case refers to the `as` keyword"
    )]
    fn as_i32(self) -> i32;
}

impl GuiProperty for crate::consts::GuiControlProperty {
    fn as_i32(self) -> i32 {
        self as i32
    }
}
impl GuiProperty for crate::consts::GuiDefaultProperty {
    fn as_i32(self) -> i32 {
        self as i32
    }
}
impl GuiProperty for crate::consts::GuiCheckBoxProperty {
    fn as_i32(self) -> i32 {
        self as i32
    }
}
impl GuiProperty for crate::consts::GuiColorPickerProperty {
    fn as_i32(self) -> i32 {
        self as i32
    }
}
impl GuiProperty for crate::consts::GuiComboBoxProperty {
    fn as_i32(self) -> i32 {
        self as i32
    }
}
impl GuiProperty for crate::consts::GuiDropdownBoxProperty {
    fn as_i32(self) -> i32 {
        self as i32
    }
}
impl GuiProperty for crate::consts::GuiListViewProperty {
    fn as_i32(self) -> i32 {
        self as i32
    }
}
impl GuiProperty for crate::consts::GuiProgressBarProperty {
    fn as_i32(self) -> i32 {
        self as i32
    }
}
impl GuiProperty for crate::consts::GuiScrollBarProperty {
    fn as_i32(self) -> i32 {
        self as i32
    }
}
impl GuiProperty for crate::consts::GuiSliderProperty {
    fn as_i32(self) -> i32 {
        self as i32
    }
}
impl GuiProperty for crate::consts::GuiValueBoxProperty {
    fn as_i32(self) -> i32 {
        self as i32
    }
}
impl GuiProperty for crate::consts::GuiToggleProperty {
    fn as_i32(self) -> i32 {
        self as i32
    }
}
