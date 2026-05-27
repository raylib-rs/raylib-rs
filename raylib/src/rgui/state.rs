use crate::core::text::WeakFont;
use crate::ffi;
use std::cell::RefCell;
use std::ffi::CString;

thread_local! {
    /// Holds the current tooltip string. `GuiSetTooltip` stores the raw pointer and
    /// dereferences it on later control draws, so the owning `CString` must outlive
    /// the call — it lives here until the next `gui_set_tooltip` on this thread.
    static TOOLTIP: RefCell<Option<CString>> = const { RefCell::new(None) };
}

/// raygui global state, style, font and tooltip controls. Implemented for the
/// draw-handle types (call during drawing) and for [`crate::RaylibHandle`] (call during
/// setup).
pub trait RaylibGuiState {
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
    /// Set gui controls alpha (global state); `alpha` is clamped 0.0..=1.0 by raygui.
    #[inline]
    fn gui_set_alpha(&mut self, alpha: f32) {
        unsafe { ffi::GuiSetAlpha(alpha) }
    }
    /// Set gui state (global state)
    #[inline]
    fn gui_set_state(&mut self, state: crate::consts::GuiState) {
        unsafe { ffi::GuiSetState(state as i32) }
    }
    /// Get gui state (global state)
    #[inline]
    fn gui_get_state(&self) -> crate::consts::GuiState {
        use crate::consts::GuiState;
        // raygui returns one of the GuiState discriminants; map explicitly rather
        // than transmuting an arbitrary i32 into the enum.
        match unsafe { ffi::GuiGetState() } {
            x if x == GuiState::STATE_NORMAL as i32 => GuiState::STATE_NORMAL,
            x if x == GuiState::STATE_FOCUSED as i32 => GuiState::STATE_FOCUSED,
            x if x == GuiState::STATE_PRESSED as i32 => GuiState::STATE_PRESSED,
            x if x == GuiState::STATE_DISABLED as i32 => GuiState::STATE_DISABLED,
            _ => GuiState::STATE_NORMAL,
        }
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
    /// Get one style property
    #[inline]
    fn gui_get_style(&self, control: crate::consts::GuiControl, property: impl GuiProperty) -> i32 {
        unsafe { ffi::GuiGetStyle(control as i32, property.as_i32()) }
    }
    /// Load style file (.rgs)
    #[inline]
    fn gui_load_style(&mut self, filename: impl AsRef<str>) {
        // CString used here (not scratch): GuiLoadStyle is not per-frame and may
        // retain the pointer's contents during parsing; a fresh allocation is correct.
        let c = CString::new(filename.as_ref()).unwrap();
        unsafe { ffi::GuiLoadStyle(c.as_ptr()) }
    }
    /// Load style default over global style
    #[inline]
    fn gui_load_style_default(&mut self) {
        unsafe { ffi::GuiLoadStyleDefault() }
    }
    /// Enable gui tooltips (global state)
    #[inline]
    fn gui_enable_tooltip(&mut self) {
        unsafe { ffi::GuiEnableTooltip() }
    }
    /// Disable gui tooltips (global state)
    #[inline]
    fn gui_disable_tooltip(&mut self) {
        unsafe { ffi::GuiDisableTooltip() }
    }
    /// Set tooltip string. The string is retained until the next call, because
    /// raygui stores the pointer and reads it on later control draws.
    #[inline]
    fn gui_set_tooltip(&mut self, tooltip: impl AsRef<str>) {
        let c = CString::new(tooltip.as_ref()).unwrap_or_default();
        TOOLTIP.with(|cell| {
            let mut slot = cell.borrow_mut();
            *slot = Some(c);
            let ptr = slot.as_ref().unwrap().as_ptr();
            // SAFETY: GuiSetTooltip stores `ptr` and dereferences it during later
            // control draws; the owning CString is retained in TOOLTIP, so the
            // pointer stays valid until the next gui_set_tooltip on this thread.
            unsafe { ffi::GuiSetTooltip(ptr) };
        });
    }
}

#[diagnostic::on_unimplemented(
    message = "{Self} is not a gui property, or does not implement the GuiProperty trait.",
    note = "As of Raylib 5.5, raygui functions that once took \"property enum as i32\" now just take the enum."
)]
/// Trait for types that can be used as raygui style property identifiers.
pub trait GuiProperty {
    /// Convert this property to its `i32` discriminant for the raygui C API.
    // The convention `as_i32(self)` matches the original API; properties are
    // cheap `Copy` enums so consuming `self` is intentional.
    #[allow(clippy::wrong_self_convention)]
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
