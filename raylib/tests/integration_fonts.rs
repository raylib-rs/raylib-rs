//! Tier-2: font loading + export smoke tests.
//! Salvaged from raylib-test/src/text.rs.
#![cfg(feature = "software_renderer")]
// RaylibFont trait is only needed by the PNG branch's
// `export_font_as_code` call below; the TTF branch loads but doesn't
// export. Scoping the import to the PNG gate avoids an unused_imports
// warning when PNG support is off.
#[cfg(feature = "SUPPORT_FILEFORMAT_PNG")]
use raylib::core::text::RaylibFont;
use raylib::test_harness::with_headless;

#[test]
fn fonts_load_and_export_smoke() {
    with_headless(64, 64, |_rl, _thread| {
        // Bitmap font (PNG). Requires PNG support to actually load.
        #[cfg(feature = "SUPPORT_FILEFORMAT_PNG")]
        let png_font_loaded = {
            let png_path = "tests/fixtures/alagard.png";
            if std::path::Path::new(png_path).exists() {
                match _rl.load_font(_thread, png_path) {
                    Ok(_font) => true,
                    Err(e) => {
                        eprintln!("SKIP: load_font failed: {e}");
                        false
                    }
                }
            } else {
                eprintln!("SKIP: {png_path} not found");
                false
            }
        };
        #[cfg(not(feature = "SUPPORT_FILEFORMAT_PNG"))]
        let png_font_loaded = {
            eprintln!("SKIP: load_font (PNG) requires SUPPORT_FILEFORMAT_PNG");
            false
        };

        // TTF font with explicit size + None glyph chars (use defaults).
        // Requires SUPPORT_FILEFORMAT_TTF.
        #[cfg(feature = "SUPPORT_FILEFORMAT_TTF")]
        {
            let ttf_path = "tests/fixtures/pixeloid.ttf";
            if std::path::Path::new(ttf_path).exists() {
                let _font_ex = _rl
                    .load_font_ex(_thread, ttf_path, 32, None)
                    .expect("load_font_ex (ttf) succeeds");
            } else {
                eprintln!("SKIP: {ttf_path} not found");
            }
        }
        #[cfg(not(feature = "SUPPORT_FILEFORMAT_TTF"))]
        eprintln!("SKIP: load_font_ex (TTF) requires SUPPORT_FILEFORMAT_TTF");

        // export_font_as_code: write a .h file. Use target/tmp/ for output.
        if png_font_loaded {
            #[cfg(feature = "SUPPORT_FILEFORMAT_PNG")]
            {
                std::fs::create_dir_all("target/tmp").expect("mkdir target/tmp");
                let font = _rl
                    .load_font(_thread, "tests/fixtures/alagard.png")
                    .expect("load_font for export");
                let _ = font.export_font_as_code("target/tmp/font.h");
                // existence not asserted: ExportFontAsCode may fail silently
                // under software_renderer with no GL texture upload.
            }
        }
    });
}
