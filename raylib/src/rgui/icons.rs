use crate::ffi;
use crate::ffi::Color;
use crate::rgui::scratch::scratch_txt;
use std::ffi::CStr;

/// Number of raygui icons (= `RAYGUI_ICON_MAX_ICONS`).
pub const RAYGUI_ICON_MAX_ICONS: usize = 256;

/// Number of `u32` words per icon (= `RAYGUI_ICON_DATA_ELEMENTS`,
/// `RAYGUI_ICON_SIZE * RAYGUI_ICON_SIZE / 32` = `16 * 16 / 32`).
pub const RAYGUI_ICON_DATA_ELEMENTS: usize = 8;

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
        unsafe { CStr::from_ptr(buffer) }
            .to_string_lossy()
            .into_owned()
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

    /// Borrow raygui's icon buffer as a typed 256×8 grid (read-only).
    ///
    /// `256` = [`RAYGUI_ICON_MAX_ICONS`]; `8` = [`RAYGUI_ICON_DATA_ELEMENTS`]
    /// (`RAYGUI_ICON_SIZE * RAYGUI_ICON_SIZE / 32` = `16 * 16 / 32`). Each
    /// entry is one icon's bitmap: 256 bits, 1 bit per pixel, packed into 8
    /// `u32` words.
    ///
    /// # Bitmap layout
    ///
    /// Within each `[u32; 8]` entry, bit `0` of word `0` is the **top-left
    /// pixel**; bits `0..16` of word `0` are row `0` (LSB-first, left-to-right);
    /// bits `16..32` of word `0` are row `1`; word `1` holds rows `2..4`; …;
    /// word `7` holds rows `14..16`. (Each 16×16 icon = 256 bits = 8 × `u32`.)
    ///
    /// The buffer is live: mutations via [`gui_get_icons_mut`](Self::gui_get_icons_mut)
    /// are visible to subsequent [`gui_draw_icon`](Self::gui_draw_icon) calls.
    #[inline]
    fn gui_get_icons(&self) -> &[[u32; RAYGUI_ICON_DATA_ELEMENTS]; RAYGUI_ICON_MAX_ICONS] {
        // SAFETY: GuiGetIcons returns a non-null pointer to raygui's icon
        // buffer (either the static `guiIcons` array or a RAYGUI_MALLOC'd
        // replacement from GuiLoadIconsFromMemory — note: GuiLoadIcons from
        // file does NOT swap the pointer; it `fread`s in place. Only
        // GuiLoadIconsFromMemory replaces guiIconsPtr). The buffer holds
        // exactly RAYGUI_ICON_MAX_ICONS * RAYGUI_ICON_DATA_ELEMENTS u32s and
        // is aligned to `align_of::<u32>() == 4`; the cast to
        // `*const [[u32; 8]; 256]` preserves both size and alignment (no
        // inter-element padding in Rust arrays).
        //
        // Lifetime: the returned `&` is tied to `&self`. The only raygui
        // call that *replaces* the underlying pointer is
        // GuiLoadIconsFromMemory, which Task 5 wraps as an `&mut self`
        // method — making it statically impossible to swap the pointer
        // while this shared borrow is alive.
        unsafe {
            let ptr = ffi::GuiGetIcons() as *const [u32; RAYGUI_ICON_DATA_ELEMENTS];
            &*(ptr as *const [[u32; RAYGUI_ICON_DATA_ELEMENTS]; RAYGUI_ICON_MAX_ICONS])
        }
    }

    /// Borrow raygui's icon buffer mutably. Edits are observable on the next
    /// [`gui_draw_icon`](Self::gui_draw_icon) call.
    ///
    /// See [`gui_get_icons`](Self::gui_get_icons) for the bit layout (bit 0
    /// of word 0 = top-left pixel; LSB-first, 16 pixels per `u32` row half).
    #[inline]
    fn gui_get_icons_mut(
        &mut self,
    ) -> &mut [[u32; RAYGUI_ICON_DATA_ELEMENTS]; RAYGUI_ICON_MAX_ICONS] {
        // SAFETY: Same alignment + size argument as gui_get_icons. The
        // `&mut self` receiver guarantees no other `gui_*` method (including
        // the pointer-swapping gui_load_icons_from_memory in Task 5) can run
        // while this exclusive borrow is alive.
        unsafe {
            let ptr = ffi::GuiGetIcons() as *mut [u32; RAYGUI_ICON_DATA_ELEMENTS];
            &mut *(ptr as *mut [[u32; RAYGUI_ICON_DATA_ELEMENTS]; RAYGUI_ICON_MAX_ICONS])
        }
    }
}

#[cfg(all(test, feature = "software_renderer"))]
mod tests {
    use super::*;
    use crate::test_harness::with_headless;

    #[test]
    fn icons_buffer_round_trip() {
        with_headless(64, 64, |rl, _thread| {
            // Slots 234..=255 are unassigned (all-zero) in raygui's default
            // guiIcons table — ICON_COLLISION = 233 is the last populated icon.
            // Slot 200 is ICON_FILETYPE_BINARY (populated); don't use it.
            const SLOT: usize = 240;
            let pattern: [u32; 8] = [0xDEADBEEF; 8];

            // Mutate via gui_get_icons_mut.
            {
                let icons = rl.gui_get_icons_mut();
                icons[SLOT] = pattern;
            }

            // Read back via gui_get_icons.
            let icons = rl.gui_get_icons();
            assert_eq!(
                icons[SLOT], pattern,
                "icon buffer aliases raygui's live state"
            );
            assert_eq!(icons.len(), 256, "RAYGUI_ICON_MAX_ICONS = 256");
        });
    }
}
