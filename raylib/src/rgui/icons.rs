use crate::core::error::LoadIconsError;
use crate::ffi;
use crate::ffi::Color;
use crate::rgui::scratch::scratch_txt;
use std::ffi::CStr;

/// Number of raygui icons (= `RAYGUI_ICON_MAX_ICONS`).
pub const RAYGUI_ICON_MAX_ICONS: usize = 256;

/// Number of `u32` words per icon (= `RAYGUI_ICON_DATA_ELEMENTS`,
/// `RAYGUI_ICON_SIZE * RAYGUI_ICON_SIZE / 32` = `16 * 16 / 32`).
pub const RAYGUI_ICON_DATA_ELEMENTS: usize = 8;

/// Pre-validate a `.rgi` payload's 12-byte header. Returns `(icon_count, icon_size)`
/// on success. Public-in-crate so the from-file path can reuse it.
pub(crate) fn validate_rgi_header(buf: &[u8]) -> Result<(u16, u16), LoadIconsError> {
    if buf.len() < 12 {
        return Err(LoadIconsError::HeaderTruncated(buf.len()));
    }
    let sig: [u8; 4] = buf[..4].try_into().expect("sliced to 4");
    if &sig != b"rGI " {
        return Err(LoadIconsError::InvalidSignature(sig));
    }
    // Offset 4-7: version (2) + reserved (2). We don't check these.
    let icon_count = u16::from_le_bytes(buf[8..10].try_into().expect("sliced to 2"));
    let icon_size = u16::from_le_bytes(buf[10..12].try_into().expect("sliced to 2"));
    if icon_size != 16 {
        return Err(LoadIconsError::UnsupportedIconSize {
            expected: 16,
            actual: icon_size,
        });
    }
    if (icon_count as usize) > RAYGUI_ICON_MAX_ICONS {
        return Err(LoadIconsError::TooManyIcons {
            max: RAYGUI_ICON_MAX_ICONS as u16,
            actual: icon_count,
        });
    }
    Ok((icon_count, icon_size))
}

/// Verify `len` fits in `i32`. Used by from-memory paths before passing the length
/// to raygui (which takes `int dataSize`).
pub(crate) fn check_i32_len(len: usize) -> Result<i32, LoadIconsError> {
    i32::try_from(len).map_err(|_| LoadIconsError::LengthOverflow(len))
}

/// Copy raygui's `char**` icon-names buffer into a `Vec<String>` and free
/// the underlying memory via `MemFree` (allocator-correct after the Task 2
/// RAYGUI_MALLOC unification — both default to `RL_MALLOC`/`RL_FREE`).
///
/// Returns an empty `Vec` if `ptr` is null. raygui produces NULL for both
/// "names not requested" and "load failed", but for the `_with_names` variants
/// we have already pre-validated the payload, so a NULL here is treated as
/// "raygui produced an empty name set".
///
/// # Safety
///
/// `ptr` must either be NULL or a `char**` returned by `GuiLoadIcons` /
/// `GuiLoadIconsFromMemory` with at least `RAYGUI_ICON_MAX_ICONS` valid
/// entries, each a NUL-terminated C string allocated via `RAYGUI_MALLOC`.
unsafe fn copy_and_free_names(ptr: *mut *mut std::os::raw::c_char) -> Vec<String> {
    if ptr.is_null() {
        return Vec::new();
    }
    let mut names = Vec::with_capacity(RAYGUI_ICON_MAX_ICONS);
    for i in 0..RAYGUI_ICON_MAX_ICONS {
        // SAFETY: i in [0, 256); raygui guarantees 256 entries when char**
        // is non-NULL. Each entry is a NUL-terminated cstring.
        let cstr_ptr = unsafe { *ptr.add(i) };
        if cstr_ptr.is_null() {
            names.push(String::new());
        } else {
            let s = unsafe { CStr::from_ptr(cstr_ptr) }
                .to_string_lossy()
                .into_owned();
            names.push(s);
            // SAFETY: ffi::MemFree routes through raylib's RL_FREE, which equals
            // raygui's RAYGUI_FREE after the binding/rgui_wrapper.c define.
            unsafe { ffi::MemFree(cstr_ptr.cast()) };
        }
    }
    // Free the outer array itself.
    // SAFETY: ptr was returned by RAYGUI_MALLOC; freeing via RL_FREE-equivalent.
    unsafe { ffi::MemFree(ptr.cast()) };
    names
}

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

    /// Load icons from an in-memory `.rgi` buffer, discarding names. Validates
    /// the header signature, icon count, and icon size before delegating to
    /// raygui.
    ///
    /// # Upstream wart
    ///
    /// raygui's `GuiLoadIconsFromMemory` reassigns its internal icons pointer
    /// to a fresh allocation on every call without freeing the previous one.
    /// Calling this method multiple times in one process leaks the previous
    /// buffer (≈8 KB per call). raygui's `GuiLoadIcons` (the file variant) does
    /// not have this issue.
    #[inline]
    fn gui_load_icons_from_memory(&mut self, data: &[u8]) -> Result<(), LoadIconsError> {
        let _hdr = validate_rgi_header(data)?;
        let len = check_i32_len(data.len())?;
        // SAFETY: data lives for the duration of the call; raygui memcpys out
        // of the buffer synchronously. load_names=false ⇒ returned char** is
        // NULL by raygui's contract, which we ignore.
        unsafe {
            let _ = ffi::GuiLoadIconsFromMemory(data.as_ptr(), len, false);
        }
        Ok(())
    }

    /// Load icons from an in-memory `.rgi` buffer, returning the 256 icon names.
    /// Same upstream-leak caveat as [`Self::gui_load_icons_from_memory`].
    ///
    /// Names with fewer than 32 (`RAYGUI_ICON_MAX_NAME_LENGTH`) characters are
    /// returned trimmed at the first NUL byte.
    #[inline]
    fn gui_load_icons_from_memory_with_names(
        &mut self,
        data: &[u8],
    ) -> Result<Vec<String>, LoadIconsError> {
        let _hdr = validate_rgi_header(data)?;
        let len = check_i32_len(data.len())?;
        // SAFETY: data lives for the duration of the call.
        let ptr = unsafe { ffi::GuiLoadIconsFromMemory(data.as_ptr(), len, true) };
        // SAFETY: ptr is either NULL or a char** with RAYGUI_ICON_MAX_ICONS entries.
        let names = unsafe { copy_and_free_names(ptr) };
        Ok(names)
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

    #[test]
    fn load_icons_from_memory_rejects_short_header() {
        use crate::core::error::LoadIconsError;
        with_headless(64, 64, |rl, _thread| {
            let err = rl.gui_load_icons_from_memory(&[0u8; 7]).unwrap_err();
            assert!(
                matches!(err, LoadIconsError::HeaderTruncated(7)),
                "got {err:?}"
            );
        });
    }

    #[test]
    fn load_icons_from_memory_rejects_bad_signature() {
        use crate::core::error::LoadIconsError;
        with_headless(64, 64, |rl, _thread| {
            let mut buf = [0u8; 12];
            buf[..4].copy_from_slice(b"XXXX");
            let err = rl.gui_load_icons_from_memory(&buf).unwrap_err();
            assert!(
                matches!(err, LoadIconsError::InvalidSignature(sig) if &sig == b"XXXX"),
                "got {err:?}"
            );
        });
    }

    #[test]
    fn load_icons_from_memory_rejects_bad_icon_size() {
        use crate::core::error::LoadIconsError;
        with_headless(64, 64, |rl, _thread| {
            // Valid signature + version + reserved + iconCount=1 + iconSize=32 (not 16).
            let mut buf = [0u8; 12];
            buf[..4].copy_from_slice(b"rGI ");
            buf[8..10].copy_from_slice(&1u16.to_le_bytes()); // iconCount
            buf[10..12].copy_from_slice(&32u16.to_le_bytes()); // iconSize
            let err = rl.gui_load_icons_from_memory(&buf).unwrap_err();
            assert!(
                matches!(
                    err,
                    LoadIconsError::UnsupportedIconSize {
                        expected: 16,
                        actual: 32
                    }
                ),
                "got {err:?}"
            );
        });
    }

    #[test]
    fn load_icons_from_memory_rejects_too_many_icons() {
        use crate::core::error::LoadIconsError;
        with_headless(64, 64, |rl, _thread| {
            let mut buf = [0u8; 12];
            buf[..4].copy_from_slice(b"rGI ");
            buf[8..10].copy_from_slice(&300u16.to_le_bytes()); // iconCount
            buf[10..12].copy_from_slice(&16u16.to_le_bytes()); // iconSize
            let err = rl.gui_load_icons_from_memory(&buf).unwrap_err();
            assert!(
                matches!(
                    err,
                    LoadIconsError::TooManyIcons {
                        max: 256,
                        actual: 300
                    }
                ),
                "got {err:?}"
            );
        });
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn check_i32_len_boundary() {
        assert!(check_i32_len(0).is_ok());
        assert!(check_i32_len(i32::MAX as usize).is_ok());
        assert!(matches!(
            check_i32_len(i32::MAX as usize + 1),
            Err(crate::core::error::LoadIconsError::LengthOverflow(_))
        ));
    }

    #[test]
    fn validate_rgi_header_accepts_well_formed() {
        let mut buf = [0u8; 12];
        buf[..4].copy_from_slice(b"rGI ");
        buf[8..10].copy_from_slice(&5u16.to_le_bytes()); // iconCount
        buf[10..12].copy_from_slice(&16u16.to_le_bytes()); // iconSize
        let (count, size) = validate_rgi_header(&buf).expect("well-formed header");
        assert_eq!(count, 5);
        assert_eq!(size, 16);
    }
}
