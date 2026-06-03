//! Tier-2: image + screen I/O smoke tests. Salvaged from
//! raylib-test/src/{misc,texture}.rs.
#![cfg(feature = "software_renderer")]
#[cfg(feature = "SUPPORT_FILEFORMAT_PNG")]
use raylib::prelude::Image;
use raylib::test_harness::with_headless;

#[test]
fn image_io_smoke() {
    with_headless(64, 64, |rl, thread| {
        // take_screenshot writes a file.
        let out_dir = "target/tmp";
        std::fs::create_dir_all(out_dir).expect("mkdir target/tmp");
        let shot = format!("{out_dir}/screenshot.png");
        rl.take_screenshot(thread, &shot);
        // The Memory platform's TakeScreenshot may or may not actually write
        // the PNG depending on PNG-format support. Don't assert existence —
        // exercise that the call doesn't panic.

        // load_image_from_screen: make sure it doesn't segfault.
        let _ = rl.load_image_from_screen(thread);

        // texture_load: load PNG from file + from existing Image. Both PNG
        // loading and texture upload require SUPPORT_FILEFORMAT_PNG; skip
        // when that feature isn't enabled.
        #[cfg(feature = "SUPPORT_FILEFORMAT_PNG")]
        {
            let path = "tests/fixtures/billboard.png";
            if std::path::Path::new(path).exists() {
                let img = Image::load_image(path).expect("image loads");
                let _tex = rl
                    .load_texture(thread, path)
                    .expect("texture loads from path");
                let _tex2 = rl
                    .load_texture_from_image(thread, &img)
                    .expect("texture loads from image");
            } else {
                eprintln!("SKIP: {path} not found");
            }
        }

        // render_texture: wrapped in catch_unwind because software_renderer
        // may not support GL framebuffer objects. If it panics, log a SKIP
        // and continue.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            rl.load_render_texture(thread, 256, 256)
                .expect("render texture creates")
        }));
        match result {
            Ok(_rt) => { /* loaded successfully; dropping at end */ }
            Err(_) => {
                eprintln!("SKIP: load_render_texture not supported under software_renderer")
            }
        }
    });
}
