//! WS5b Tier-2: rlgl immediate-mode + matrix stack draw into the software
//! framebuffer. Headless. Uses the WS5-prep normalized `render_frame`.
//!
//! ## rlsw texcoord requirement
//! The software renderer (rlsw / PLATFORM=Memory) rasterises textured quads
//! via the texture sampler.  Vertices without explicit texcoords default to
//! (0, 0).  The shapes texture at (0, 0) is transparent, so any draw call
//! that omits `rlTexCoord2f` / `texcoord2f` will produce 0 pixels even when a
//! texture is bound.  All tests below bind the shapes texture and emit
//! one texcoord per vertex so the sampler returns an opaque white texel that
//! is then tinted by the vertex colour.
#![cfg(feature = "software_renderer")]
use raylib::prelude::*;

#[test]
fn rlgl_immediate_triangle_renders() {
    raylib::test_harness::with_headless(100, 100, |rl, thread| {
        // ── Baseline: raw FFI quads with texture (known-good reference) ──────
        let img_raw = raylib::test_harness::render_frame(rl, thread, |d| {
            let _ = d;
            d.clear_background(Color::BLACK);
            unsafe {
                let tex = raylib::ffi::GetShapesTexture();
                let tw = tex.width as f32;
                let th = tex.height as f32;
                let rect = raylib::ffi::GetShapesTextureRectangle();
                raylib::ffi::rlSetTexture(tex.id);
                raylib::ffi::rlBegin(raylib::ffi::RL_QUADS as i32);
                raylib::ffi::rlNormal3f(0.0, 0.0, 1.0);
                raylib::ffi::rlColor4ub(255, 0, 0, 255);
                raylib::ffi::rlTexCoord2f(rect.x / tw, rect.y / th);
                raylib::ffi::rlVertex2f(20.0, 20.0);
                raylib::ffi::rlTexCoord2f(rect.x / tw, (rect.y + rect.height) / th);
                raylib::ffi::rlVertex2f(20.0, 80.0);
                raylib::ffi::rlTexCoord2f((rect.x + rect.width) / tw, (rect.y + rect.height) / th);
                raylib::ffi::rlVertex2f(80.0, 80.0);
                raylib::ffi::rlTexCoord2f((rect.x + rect.width) / tw, rect.y / th);
                raylib::ffi::rlVertex2f(80.0, 20.0);
                raylib::ffi::rlEnd();
                raylib::ffi::rlSetTexture(0);
            }
        });
        let red_raw = count_red(&img_raw, 0, 100, 0, 100);
        println!("baseline raw FFI quads: {red_raw} red px");
        assert!(red_raw > 100, "baseline raw FFI failed: {red_raw}");

        // ── Test A: safe rl_begin / vertex API (no matrix) ───────────────────
        // Exercises DrawMode::Quads + texcoord2f + color4ub + vertex2f.
        let img_safe_nodraw = raylib::test_harness::render_frame(rl, thread, |d| {
            d.clear_background(Color::BLACK);
            unsafe {
                let tex = raylib::ffi::GetShapesTexture();
                let tw = tex.width as f32;
                let th = tex.height as f32;
                let rect = raylib::ffi::GetShapesTextureRectangle();
                raylib::ffi::rlSetTexture(tex.id);
                let mut v = d.rl_begin(DrawMode::Quads);
                v.normal3f(0.0, 0.0, 1.0);
                v.color4ub(Color::RED);
                v.texcoord2f(rect.x / tw, rect.y / th);
                v.vertex2f(20.0, 20.0);
                v.texcoord2f(rect.x / tw, (rect.y + rect.height) / th);
                v.vertex2f(20.0, 80.0);
                v.texcoord2f((rect.x + rect.width) / tw, (rect.y + rect.height) / th);
                v.vertex2f(80.0, 80.0);
                v.texcoord2f((rect.x + rect.width) / tw, rect.y / th);
                v.vertex2f(80.0, 20.0);
                drop(v); // rlEnd
                raylib::ffi::rlSetTexture(0);
            }
        });
        let red_safe_nodraw = count_red(&img_safe_nodraw, 0, 100, 0, 100);
        println!("safe rl_begin (no matrix): {red_safe_nodraw} red px");
        assert!(
            red_safe_nodraw > 100,
            "safe rl_begin (no matrix) failed: {red_safe_nodraw}"
        );

        // ── Test B: safe rl_push_matrix + rl_translatef + rl_draw ────────────
        // The quad is drawn at x=[20..50], y=[20..50] with a +30 X translate,
        // so it should land at x=[50..80], y=[20..50].
        let img_push = raylib::test_harness::render_frame(rl, thread, |d| {
            d.clear_background(Color::BLACK);
            unsafe {
                let tex = raylib::ffi::GetShapesTexture();
                let tw = tex.width as f32;
                let th = tex.height as f32;
                let rect = raylib::ffi::GetShapesTextureRectangle();
                raylib::ffi::rlSetTexture(tex.id);
                {
                    let mut m = d.rl_push_matrix();
                    m.rl_translatef(30.0, 0.0, 0.0);
                    m.rl_draw(DrawMode::Quads, |v| {
                        v.normal3f(0.0, 0.0, 1.0);
                        v.color4ub(Color::RED);
                        v.texcoord2f(rect.x / tw, rect.y / th);
                        v.vertex2f(20.0, 20.0);
                        v.texcoord2f(rect.x / tw, (rect.y + rect.height) / th);
                        v.vertex2f(20.0, 50.0);
                        v.texcoord2f((rect.x + rect.width) / tw, (rect.y + rect.height) / th);
                        v.vertex2f(50.0, 50.0);
                        v.texcoord2f((rect.x + rect.width) / tw, rect.y / th);
                        v.vertex2f(50.0, 20.0);
                    });
                    // RlMatrix drops here → rlPopMatrix
                }
                raylib::ffi::rlSetTexture(0);
            }
        });
        let red_push_total = count_red(&img_push, 0, 100, 0, 100);
        // Expected translated box: x in [50,80), y in [20,50)
        let red_push_box = count_red(&img_push, 50, 80, 20, 50);
        println!(
            "safe rl_push+translate+rl_draw: total={red_push_total} in_box=[50..80,20..50]={red_push_box}"
        );
        assert!(
            red_push_total > 100,
            "safe rl_push+rl_draw produced no pixels: {red_push_total}"
        );
        assert!(
            red_push_box > 100,
            "safe rl_push+rl_draw pixels not in expected translated region: \
             total={red_push_total}, in_box={red_push_box}"
        );

        // ── Test C: colour fidelity — red vs blue ─────────────────────────────
        // Draw the same quad in blue; confirm the counter sees zero red.
        let img_blue = raylib::test_harness::render_frame(rl, thread, |d| {
            d.clear_background(Color::BLACK);
            unsafe {
                let tex = raylib::ffi::GetShapesTexture();
                let tw = tex.width as f32;
                let th = tex.height as f32;
                let rect = raylib::ffi::GetShapesTextureRectangle();
                raylib::ffi::rlSetTexture(tex.id);
                let mut v = d.rl_begin(DrawMode::Quads);
                v.normal3f(0.0, 0.0, 1.0);
                v.color4ub(Color::BLUE);
                v.texcoord2f(rect.x / tw, rect.y / th);
                v.vertex2f(20.0, 20.0);
                v.texcoord2f(rect.x / tw, (rect.y + rect.height) / th);
                v.vertex2f(20.0, 80.0);
                v.texcoord2f((rect.x + rect.width) / tw, (rect.y + rect.height) / th);
                v.vertex2f(80.0, 80.0);
                v.texcoord2f((rect.x + rect.width) / tw, rect.y / th);
                v.vertex2f(80.0, 20.0);
                drop(v);
                raylib::ffi::rlSetTexture(0);
            }
        });
        let red_blue_frame = count_red(&img_blue, 0, 100, 0, 100);
        let blue_px = count_blue(&img_blue, 0, 100, 0, 100);
        println!("blue quad: {blue_px} blue px, {red_blue_frame} red px (should be 0)");
        assert!(
            blue_px > 100,
            "blue quad produced no blue pixels: {blue_px}"
        );
        assert!(
            red_blue_frame == 0,
            "blue quad leaked into red counter: {red_blue_frame}"
        );
    });
}

fn count_red(img: &Image, x0: i32, x1: i32, y0: i32, y1: i32) -> u32 {
    let mut count = 0u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = raylib::test_harness::pixel_at(img, x, y);
            if p.r > 150 && p.g < 90 && p.b < 90 {
                count += 1;
            }
        }
    }
    count
}

fn count_blue(img: &Image, x0: i32, x1: i32, y0: i32, y1: i32) -> u32 {
    let mut count = 0u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = raylib::test_harness::pixel_at(img, x, y);
            if p.b > 150 && p.r < 90 && p.g < 90 {
                count += 1;
            }
        }
    }
    count
}

#[test]
fn rlgl_vertex2i_renders() {
    raylib::test_harness::with_headless(100, 100, |rl, thread| {
        let img = raylib::test_harness::render_frame(rl, thread, |d| {
            d.clear_background(Color::BLACK);
            unsafe {
                let tex = raylib::ffi::GetShapesTexture();
                let tw = tex.width as f32;
                let th = tex.height as f32;
                let rect = raylib::ffi::GetShapesTextureRectangle();
                raylib::ffi::rlSetTexture(tex.id);
                let mut v = d.rl_begin(DrawMode::Quads);
                v.normal3f(0.0, 0.0, 1.0);
                v.color4ub(Color::RED);
                v.texcoord2f(rect.x / tw, rect.y / th);
                v.vertex2i(20, 20);
                v.texcoord2f(rect.x / tw, (rect.y + rect.height) / th);
                v.vertex2i(20, 80);
                v.texcoord2f((rect.x + rect.width) / tw, (rect.y + rect.height) / th);
                v.vertex2i(80, 80);
                v.texcoord2f((rect.x + rect.width) / tw, rect.y / th);
                v.vertex2i(80, 20);
                drop(v); // rlEnd
                raylib::ffi::rlSetTexture(0);
            }
        });
        let red = count_red(&img, 0, 100, 0, 100);
        assert!(red > 100, "vertex2i quad produced too few red px: {red}");
    });
}
