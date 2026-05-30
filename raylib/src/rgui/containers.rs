use crate::ffi;
use crate::ffi::{Rectangle, Vector2};
use crate::rgui::scratch::{scratch_txt, scratch_txt_opt, scratch_txt_slice};

/// raygui container / separator controls.
pub trait RaylibGuiContainers {
    /// Window Box control, shows a window that can be closed. Returns true when closed.
    #[inline]
    fn gui_window_box(&mut self, bounds: impl Into<Rectangle>, title: impl AsRef<str>) -> bool {
        unsafe { ffi::GuiWindowBox(bounds.into(), scratch_txt(title)) > 0 }
    }
    /// Group Box control with text name.
    #[inline]
    fn gui_group_box(&mut self, bounds: impl Into<Rectangle>, text: impl AsRef<str>) -> bool {
        unsafe { ffi::GuiGroupBox(bounds.into(), scratch_txt(text)) > 0 }
    }
    /// Line separator control, optionally with embedded text (`None` = no text).
    #[inline]
    fn gui_line(&mut self, bounds: impl Into<Rectangle>, text: Option<impl AsRef<str>>) -> bool {
        unsafe { ffi::GuiLine(bounds.into(), scratch_txt_opt(text)) > 0 }
    }
    /// Panel control, useful to group controls. `None` = no title.
    #[inline]
    fn gui_panel(&mut self, bounds: impl Into<Rectangle>, text: Option<impl AsRef<str>>) -> bool {
        unsafe { ffi::GuiPanel(bounds.into(), scratch_txt_opt(text)) > 0 }
    }
    /// Scroll Panel control. Returns `(result, view, scroll)`.
    #[inline]
    fn gui_scroll_panel(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: Option<impl AsRef<str>>,
        content: impl Into<Rectangle>,
        scroll: impl Into<Vector2>,
        view: impl Into<Rectangle>,
    ) -> (bool, Rectangle, Vector2) {
        let mut scroll = scroll.into();
        let mut view = view.into();
        let result = unsafe {
            ffi::GuiScrollPanel(
                bounds.into(),
                scratch_txt_opt(text),
                content.into(),
                &mut scroll,
                &mut view,
            )
        };
        (result > 0, view, scroll)
    }
    /// Tab Bar control. `active` is the selected tab index (in/out). Returns the
    /// index of a tab requested to close, or -1 if none.
    #[inline]
    fn gui_tab_bar(
        &mut self,
        bounds: impl Into<Rectangle>,
        text: &[impl AsRef<str>],
        active: &mut i32,
    ) -> i32 {
        let mut ptrs = scratch_txt_slice(text);
        unsafe { ffi::GuiTabBar(bounds.into(), ptrs.as_mut_ptr(), ptrs.len() as i32, active) }
    }
}
