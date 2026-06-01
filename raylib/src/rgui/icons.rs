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
/// `GuiLoadIconsFromMemory` with **exactly `icon_count`** valid entries
/// (one per icon declared in the loaded `.rgi` header), each a NUL-terminated
/// C string allocated via `RAYGUI_MALLOC`. raygui allocates the outer array
/// as `iconCount * sizeof(char *)`, so reading past `icon_count - 1` is
/// out-of-bounds. The caller is responsible for passing the same `icon_count`
/// that `validate_rgi_header` returned for the same payload.
unsafe fn copy_and_free_names(
    ptr: *mut *mut std::os::raw::c_char,
    icon_count: usize,
) -> Vec<String> {
    if ptr.is_null() {
        return Vec::new();
    }
    let mut names = Vec::with_capacity(icon_count);
    for i in 0..icon_count {
        // SAFETY: i in [0, icon_count); raygui allocated exactly icon_count
        // entries. Each entry is a NUL-terminated cstring.
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

/// Read the first 12 bytes of a `.rgi` file for header validation. Returns
/// fewer bytes if the file is shorter (the caller's `validate_rgi_header`
/// turns that into `HeaderTruncated`).
///
/// Uses `take(12).read_to_end` (NOT plain `Read::read`) so a short-read on a
/// valid file — permitted by `Read::read`'s contract even when the file has
/// more bytes — cannot produce a spurious `HeaderTruncated`.
fn read_header_bytes(path: &std::path::Path) -> Result<Vec<u8>, LoadIconsError> {
    use std::io::Read;
    let file = std::fs::File::open(path)?;
    let mut buf = Vec::with_capacity(12);
    file.take(12).read_to_end(&mut buf)?;
    Ok(buf)
}

/// Convert a Rust path to a C string for raygui's `fopen`-based loaders. On
/// Windows this lossy-converts non-UTF-8 components to U+FFFD (consistent
/// with the rest of the crate). An interior NUL byte in the path produces
/// [`LoadIconsError::Io`] rather than a panic.
fn path_to_c_string(path: &std::path::Path) -> Result<std::ffi::CString, LoadIconsError> {
    std::ffi::CString::new(path.to_string_lossy().as_bytes()).map_err(|_| {
        LoadIconsError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path contains an interior NUL byte",
        ))
    })
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

    /// Load icons from an in-memory `.rgi` buffer, returning one icon name per
    /// icon declared in the file (`iconCount` entries; up to
    /// `RAYGUI_ICON_MAX_ICONS` = 256). Same upstream-leak caveat as
    /// [`Self::gui_load_icons_from_memory`].
    ///
    /// Names with fewer than 32 (`RAYGUI_ICON_MAX_NAME_LENGTH`) characters are
    /// returned trimmed at the first NUL byte.
    #[inline]
    fn gui_load_icons_from_memory_with_names(
        &mut self,
        data: &[u8],
    ) -> Result<Vec<String>, LoadIconsError> {
        let (icon_count, _icon_size) = validate_rgi_header(data)?;
        let len = check_i32_len(data.len())?;
        // SAFETY: data lives for the duration of the call.
        let ptr = unsafe { ffi::GuiLoadIconsFromMemory(data.as_ptr(), len, true) };
        // SAFETY: raygui allocates exactly `icon_count` entries in the outer
        // array when load_names=true (raygui.h:4923). `icon_count` came from
        // the same validated header that raygui parsed, so the counts match.
        let names = unsafe { copy_and_free_names(ptr, icon_count as usize) };
        Ok(names)
    }

    /// Load icons from a `.rgi` file, discarding names. Pre-validates the file
    /// existence, signature, icon size (must equal `RAYGUI_ICON_SIZE` = 16),
    /// and icon count (≤ `RAYGUI_ICON_MAX_ICONS` = 256) before delegating to
    /// raygui.
    ///
    /// Returns [`LoadIconsError::FileNotFound`] if the path doesn't exist;
    /// other I/O failures (permission denied, interior-NUL path bytes, mid-read
    /// errors) surface as [`LoadIconsError::Io`]. Header validation surfaces
    /// [`LoadIconsError::HeaderTruncated`], [`LoadIconsError::InvalidSignature`],
    /// [`LoadIconsError::UnsupportedIconSize`], and
    /// [`LoadIconsError::TooManyIcons`] as appropriate.
    ///
    /// # TOCTOU
    ///
    /// The existence check, header read, and raygui's internal `fopen` are
    /// three separate operations. A file deleted after the existence check
    /// surfaces as [`LoadIconsError::Io`] rather than `FileNotFound`; a file
    /// deleted after header validation causes raygui to silently no-op,
    /// returning `Ok(())` with no icon change.
    #[inline]
    fn gui_load_icons(&mut self, path: impl AsRef<std::path::Path>) -> Result<(), LoadIconsError> {
        let path = path.as_ref();
        // 1. Existence check via fs::metadata (NOT path.exists() — exists()
        //    swallows PermissionDenied on some platforms, mis-routing it as
        //    FileNotFound).
        match std::fs::metadata(path) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(LoadIconsError::FileNotFound(path.to_path_buf()));
            }
            Err(e) => return Err(LoadIconsError::Io(e)),
        }
        // 2. Read header + validate.
        let header = read_header_bytes(path)?;
        let _hdr = validate_rgi_header(&header)?;
        // 3. Delegate to raygui.
        let c_path = path_to_c_string(path)?;
        // SAFETY: c_path lives until the end of this fn; raygui opens the file
        // synchronously via fopen. load_names=false ⇒ returned char** is NULL.
        unsafe {
            let _ = ffi::GuiLoadIcons(c_path.as_ptr(), false);
        }
        Ok(())
    }

    /// Load icons from a `.rgi` file, returning one icon name per icon declared
    /// in the file (`iconCount` entries; up to `RAYGUI_ICON_MAX_ICONS` = 256).
    ///
    /// Same error semantics as [`Self::gui_load_icons`].
    #[inline]
    fn gui_load_icons_with_names(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<Vec<String>, LoadIconsError> {
        let path = path.as_ref();
        match std::fs::metadata(path) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(LoadIconsError::FileNotFound(path.to_path_buf()));
            }
            Err(e) => return Err(LoadIconsError::Io(e)),
        }
        let header = read_header_bytes(path)?;
        let (icon_count, _icon_size) = validate_rgi_header(&header)?;
        let c_path = path_to_c_string(path)?;
        // SAFETY: c_path lives until the end of this fn.
        let ptr = unsafe { ffi::GuiLoadIcons(c_path.as_ptr(), true) };
        // SAFETY: raygui allocates exactly icon_count entries in the outer array
        // when loadIconsName=true (raygui.h:4873). icon_count came from the same
        // validated header that raygui parsed, so the counts match.
        let names = unsafe { copy_and_free_names(ptr, icon_count as usize) };
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

    #[test]
    fn load_icons_file_not_found() {
        use crate::core::error::LoadIconsError;
        with_headless(64, 64, |rl, _thread| {
            let err = rl.gui_load_icons("definitely-nonexistent.rgi").unwrap_err();
            assert!(
                matches!(err, LoadIconsError::FileNotFound(_)),
                "got {err:?}"
            );
        });
    }

    #[test]
    fn load_icons_file_bad_signature() {
        use crate::core::error::LoadIconsError;
        with_headless(64, 64, |rl, _thread| {
            // Unique per-process to avoid cross-test races under nextest's
            // parallel scheduling (each test runs in its own process, but
            // a stale file from a prior run could survive).
            let tmp = std::env::temp_dir()
                .join(format!("raylib-rs-test-bad-sig-{}.rgi", std::process::id()));
            std::fs::write(&tmp, b"NOPE_NOT_AN_RGI_FILE_AT_ALL").unwrap();
            let err = rl.gui_load_icons(&tmp).unwrap_err();
            assert!(
                matches!(err, LoadIconsError::InvalidSignature(_)),
                "got {err:?}"
            );
            let _ = std::fs::remove_file(&tmp);
        });
    }

    /// Regression test for the iconCount-vs-256 bug caught in code review:
    /// `copy_and_free_names` must loop to `icon_count`, not unconditionally
    /// to 256, otherwise it reads past raygui's outer `iconCount * sizeof(char*)`
    /// allocation for any sub-256 .rgi.
    #[test]
    fn load_icons_from_memory_with_names_handles_sub_256_icon_count() {
        with_headless(64, 64, |rl, _thread| {
            // Build a minimal .rgi payload: 1 icon, 1 name ("save"), 1 bitmap.
            // Layout (raygui.h:4824-4845):
            //   12 bytes header (sig+version+reserved+iconCount+iconSize)
            //   + iconCount * 32 bytes of names
            //   + iconCount * 8 u32s of bitmap data (16*16/32 = 8 words per icon)
            const ICON_COUNT: u16 = 1;
            const NAME_LEN: usize = 32; // RAYGUI_ICON_MAX_NAME_LENGTH
            const WORDS_PER_ICON: usize = 8;
            let mut buf =
                Vec::with_capacity(12 + ICON_COUNT as usize * (NAME_LEN + 4 * WORDS_PER_ICON));
            // Header
            buf.extend_from_slice(b"rGI ");
            buf.extend_from_slice(&100u16.to_le_bytes()); // version
            buf.extend_from_slice(&0u16.to_le_bytes()); // reserved
            buf.extend_from_slice(&ICON_COUNT.to_le_bytes());
            buf.extend_from_slice(&16u16.to_le_bytes()); // iconSize
            // Names: "save" NUL-padded to 32 bytes
            let mut name = [0u8; NAME_LEN];
            name[..4].copy_from_slice(b"save");
            buf.extend_from_slice(&name);
            // Bitmap: 8 u32s of 0xAAAAAAAA per icon (checkerboard-ish, arbitrary)
            for _ in 0..WORDS_PER_ICON {
                buf.extend_from_slice(&0xAAAA_AAAAu32.to_le_bytes());
            }

            let names = rl
                .gui_load_icons_from_memory_with_names(&buf)
                .expect("well-formed 1-icon payload");
            assert_eq!(names.len(), 1, "exactly iconCount names returned");
            assert_eq!(names[0], "save", "name round-trips through raygui + Rust");
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
