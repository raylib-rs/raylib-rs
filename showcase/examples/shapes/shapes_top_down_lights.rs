/*******************************************************************************************
*
*   raylib [shapes] example - top down lights
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 4.2, last time updated with raylib 4.2
*
*   Example contributed by Jeffery Myers (@JeffM2501) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2022-2025 Jeffery Myers (@JeffM2501)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// Custom Blend Modes
const RLGL_SRC_ALPHA: i32 = 0x0302;
const RLGL_MIN: i32 = 0x8007;
const RLGL_MAX: i32 = 0x8008;

const MAX_BOXES: usize = 20;
const MAX_SHADOWS: usize = MAX_BOXES * 3; // MAX_BOXES*3 - Each box can cast up to two shadow volumes for the edges it is away from, and one for the box itself
const MAX_LIGHTS: usize = 16;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
// Shadow geometry type
#[derive(Clone, Copy)]
struct ShadowGeometry {
    vertices: [Vector2; 4],
}

impl Default for ShadowGeometry {
    fn default() -> Self {
        ShadowGeometry {
            vertices: [Vector2::zero(); 4],
        }
    }
}

// Light info type
struct LightInfo {
    active: bool, // Is this light slot active?
    dirty: bool,  // Does this light need to be updated?
    valid: bool,  // Is this light in a valid position?

    position: Vector2,             // Light position
    mask: Option<RenderTexture2D>, // Alpha mask for the light
    outer_radius: f32,             // The distance the light touches
    bounds: Rectangle,             // A cached rectangle of the light bounds to help with culling

    shadows: [ShadowGeometry; MAX_SHADOWS],
    shadow_count: i32,
}

impl Default for LightInfo {
    fn default() -> Self {
        LightInfo {
            active: false,
            dirty: false,
            valid: false,
            position: Vector2::zero(),
            mask: None,
            outer_radius: 0.0,
            bounds: Rectangle::new(0.0, 0.0, 0.0, 0.0),
            shadows: [ShadowGeometry::default(); MAX_SHADOWS],
            shadow_count: 0,
        }
    }
}

//------------------------------------------------------------------------------------
// Module Functions Declaration
//------------------------------------------------------------------------------------
// (declarations inlined as Rust fns below)

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shapes] example - top down lights")
        .build();

    let mut lights: Vec<LightInfo> = (0..MAX_LIGHTS).map(|_| LightInfo::default()).collect();

    // Initialize our 'world' of boxes
    let mut box_count: usize = 0;
    let mut boxes: [Rectangle; MAX_BOXES] = [Rectangle::new(0.0, 0.0, 0.0, 0.0); MAX_BOXES];
    setup_boxes(&rl, &mut boxes, &mut box_count);

    // Create a checkerboard ground texture
    // SAFETY: GenImageChecked allocates an Image owned by raylib; we hand ownership to the
    // `Image` newtype which calls UnloadImage on drop (matches the C example's UnloadImage(img)).
    let img = unsafe {
        let raw = ffi::GenImageChecked(64, 64, 32, 32, Color::DARKBROWN, Color::DARKGRAY);
        Image::from_raw(raw)
    };
    let background_texture = rl
        .load_texture_from_image(&thread, &img)
        .expect("failed to upload background texture");
    drop(img); // UnloadImage(img) handled by RAII

    // Create a global light mask to hold all the blended lights
    let mut light_mask = rl
        .load_render_texture(&thread, screen_width as u32, screen_height as u32)
        .expect("failed to load light mask");

    // Setup initial light
    setup_light(&mut rl, &thread, &mut lights, 0, 600.0, 400.0, 300.0);
    let mut next_light: usize = 1;

    let mut show_lines = false;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Drag light 0
        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            let mp = rl.get_mouse_position();
            move_light(&mut lights, 0, mp.x, mp.y);
        }

        // Make a new light
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) && (next_light < MAX_LIGHTS)
        {
            let mp = rl.get_mouse_position();
            setup_light(&mut rl, &thread, &mut lights, next_light, mp.x, mp.y, 200.0);
            next_light += 1;
        }

        // Toggle debug info
        if rl.is_key_pressed(KeyboardKey::KEY_F1) {
            show_lines = !show_lines;
        }

        // Update the lights and keep track if any were dirty so we know if we need to update the master light mask
        let mut dirty_lights = false;
        for i in 0..MAX_LIGHTS {
            if update_light(&mut rl, &thread, &mut lights, i, &boxes, box_count) {
                dirty_lights = true;
            }
        }

        // Update the light mask
        if dirty_lights {
            // Build up the light mask
            let mut d = rl.begin_drawing(&thread);
            {
                let mut t = d.begin_texture_mode(&thread, &mut light_mask);

                t.clear_background(Color::BLACK);

                // Force the blend mode to only set the alpha of the destination
                // SAFETY: rlgl blend factor/mode state-setters; no preconditions beyond
                // being inside an active draw frame (which `t` proves).
                unsafe {
                    ffi::rlSetBlendFactors(RLGL_SRC_ALPHA, RLGL_SRC_ALPHA, RLGL_MIN);
                    ffi::rlSetBlendMode(ffi::BlendMode::BLEND_CUSTOM as i32);
                }

                // Merge in all the light masks
                #[expect(
                    clippy::needless_range_loop,
                    reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
                )]
                for i in 0..MAX_LIGHTS {
                    if lights[i].active {
                        if let Some(mask) = lights[i].mask.as_ref() {
                            t.draw_texture_rec(
                                mask.texture(),
                                Rectangle::new(
                                    0.0,
                                    0.0,
                                    screen_width as f32,
                                    -(screen_height as f32),
                                ),
                                Vector2::zero(),
                                Color::WHITE,
                            );
                        }
                    }
                }

                // SAFETY: rlgl batch-flush + blend-mode restore; same scope guarantee.
                unsafe {
                    ffi::rlDrawRenderBatchActive();

                    // Go back to normal blend
                    ffi::rlSetBlendMode(ffi::BlendMode::BLEND_ALPHA as i32);
                }
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        // Draw the tile background
        d.draw_texture_rec(
            &background_texture,
            Rectangle::new(0.0, 0.0, screen_width as f32, screen_height as f32),
            Vector2::zero(),
            Color::WHITE,
        );

        // Overlay the shadows from all the lights
        d.draw_texture_rec(
            light_mask.texture(),
            Rectangle::new(0.0, 0.0, screen_width as f32, -(screen_height as f32)),
            Vector2::zero(),
            Color::WHITE.alpha(if show_lines { 0.75 } else { 1.0 }),
        );

        // Draw the lights
        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for i in 0..MAX_LIGHTS {
            if lights[i].active {
                d.draw_circle(
                    lights[i].position.x as i32,
                    lights[i].position.y as i32,
                    10.0,
                    if i == 0 { Color::YELLOW } else { Color::WHITE },
                );
            }
        }

        if show_lines {
            for s in 0..(lights[0].shadow_count as usize) {
                d.draw_triangle_fan(&lights[0].shadows[s].vertices, Color::DARKPURPLE);
            }

            #[expect(
                clippy::needless_range_loop,
                reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
            )]
            for b in 0..box_count {
                if boxes[b].check_collision_recs(lights[0].bounds) {
                    d.draw_rectangle_rec(boxes[b], Color::PURPLE);
                }

                d.draw_rectangle_lines(
                    boxes[b].x as i32,
                    boxes[b].y as i32,
                    boxes[b].width as i32,
                    boxes[b].height as i32,
                    Color::DARKBLUE,
                );
            }

            d.draw_text("(F1) Hide Shadow Volumes", 10, 50, 10, Color::GREEN);
        } else {
            d.draw_text("(F1) Show Shadow Volumes", 10, 50, 10, Color::GREEN);
        }

        d.draw_fps(screen_width - 80, 10);
        d.draw_text("Drag to move light #1", 10, 10, 10, Color::DARKGREEN);
        d.draw_text("Right click to add new light", 10, 30, 10, Color::DARKGREEN);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture / UnloadRenderTexture handled by RAII drop.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}

//------------------------------------------------------------------------------------
// Module Functions Definition
//------------------------------------------------------------------------------------
// Move a light and mark it as dirty so that we update it's mask next frame
fn move_light(lights: &mut [LightInfo], slot: usize, x: f32, y: f32) {
    lights[slot].dirty = true;
    lights[slot].position.x = x;
    lights[slot].position.y = y;

    // update the cached bounds
    lights[slot].bounds.x = x - lights[slot].outer_radius;
    lights[slot].bounds.y = y - lights[slot].outer_radius;
}

// Compute a shadow volume for the edge
// It takes the edge and projects it back by the light radius and turns it into a quad
fn compute_shadow_volume_for_edge(lights: &mut [LightInfo], slot: usize, sp: Vector2, ep: Vector2) {
    if lights[slot].shadow_count as usize >= MAX_SHADOWS {
        return;
    }

    let extension = lights[slot].outer_radius * 2.0;

    let sp_vector = (sp - lights[slot].position).normalize();
    let sp_projection = sp + sp_vector.scale(extension);

    let ep_vector = (ep - lights[slot].position).normalize();
    let ep_projection = ep + ep_vector.scale(extension);

    let idx = lights[slot].shadow_count as usize;
    lights[slot].shadows[idx].vertices[0] = sp;
    lights[slot].shadows[idx].vertices[1] = ep;
    lights[slot].shadows[idx].vertices[2] = ep_projection;
    lights[slot].shadows[idx].vertices[3] = sp_projection;

    lights[slot].shadow_count += 1;
}

// Setup a light
fn setup_light(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    lights: &mut [LightInfo],
    slot: usize,
    x: f32,
    y: f32,
    radius: f32,
) {
    lights[slot].active = true;
    lights[slot].valid = false; // The light must prove it is valid
    lights[slot].mask = Some(
        rl.load_render_texture(
            thread,
            rl.get_screen_width() as u32,
            rl.get_screen_height() as u32,
        )
        .expect("failed to load light mask texture"),
    );
    lights[slot].outer_radius = radius;

    lights[slot].bounds.width = radius * 2.0;
    lights[slot].bounds.height = radius * 2.0;

    move_light(lights, slot, x, y);

    // Force the render texture to have something in it
    draw_light_mask(rl, thread, lights, slot);
}

// See if a light needs to update it's mask
fn update_light(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    lights: &mut [LightInfo],
    slot: usize,
    boxes: &[Rectangle],
    count: usize,
) -> bool {
    if !lights[slot].active || !lights[slot].dirty {
        return false;
    }

    lights[slot].dirty = false;
    lights[slot].shadow_count = 0;
    lights[slot].valid = false;

    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for i in 0..count {
        // Are we in a box? if so we are not valid
        if boxes[i].check_collision_point_rec(lights[slot].position) {
            return false;
        }

        // If this box is outside our bounds, we can skip it
        if !lights[slot].bounds.check_collision_recs(boxes[i]) {
            continue;
        }

        // Check the edges that are on the same side we are, and cast shadow volumes out from them

        // Top
        let mut sp = Vector2::new(boxes[i].x, boxes[i].y);
        let mut ep = Vector2::new(boxes[i].x + boxes[i].width, boxes[i].y);

        if lights[slot].position.y > ep.y {
            compute_shadow_volume_for_edge(lights, slot, sp, ep);
        }

        // Right
        sp = ep;
        ep.y += boxes[i].height;
        if lights[slot].position.x < ep.x {
            compute_shadow_volume_for_edge(lights, slot, sp, ep);
        }

        // Bottom
        sp = ep;
        ep.x -= boxes[i].width;
        if lights[slot].position.y < ep.y {
            compute_shadow_volume_for_edge(lights, slot, sp, ep);
        }

        // Left
        sp = ep;
        ep.y -= boxes[i].height;
        if lights[slot].position.x > ep.x {
            compute_shadow_volume_for_edge(lights, slot, sp, ep);
        }

        // The box itself
        let idx = lights[slot].shadow_count as usize;
        lights[slot].shadows[idx].vertices[0] = Vector2::new(boxes[i].x, boxes[i].y);
        lights[slot].shadows[idx].vertices[1] =
            Vector2::new(boxes[i].x, boxes[i].y + boxes[i].height);
        lights[slot].shadows[idx].vertices[2] =
            Vector2::new(boxes[i].x + boxes[i].width, boxes[i].y + boxes[i].height);
        lights[slot].shadows[idx].vertices[3] =
            Vector2::new(boxes[i].x + boxes[i].width, boxes[i].y);
        lights[slot].shadow_count += 1;
    }

    lights[slot].valid = true;

    draw_light_mask(rl, thread, lights, slot);

    true
}

// Draw the light and shadows to the mask for a light
fn draw_light_mask(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    lights: &mut [LightInfo],
    slot: usize,
) {
    // Use the light mask
    let mut d = rl.begin_drawing(thread);
    let mask = match lights[slot].mask.as_mut() {
        Some(m) => m,
        None => return,
    };
    let mut t = d.begin_texture_mode(thread, mask);

    t.clear_background(Color::WHITE);

    // Force the blend mode to only set the alpha of the destination
    // SAFETY: rlgl blend factor/mode state-setters; no preconditions beyond active draw frame.
    unsafe {
        ffi::rlSetBlendFactors(RLGL_SRC_ALPHA, RLGL_SRC_ALPHA, RLGL_MIN);
        ffi::rlSetBlendMode(ffi::BlendMode::BLEND_CUSTOM as i32);
    }

    // If we are valid, then draw the light radius to the alpha mask
    if lights[slot].valid {
        t.draw_circle_gradient(
            lights[slot].position.x as i32,
            lights[slot].position.y as i32,
            lights[slot].outer_radius,
            Color::WHITE.alpha(0.0),
            Color::WHITE,
        );
    }

    // SAFETY: rlgl batch-flush + blend mode switch; active draw frame holds.
    unsafe {
        ffi::rlDrawRenderBatchActive();

        // Cut out the shadows from the light radius by forcing the alpha to maximum
        ffi::rlSetBlendMode(ffi::BlendMode::BLEND_ALPHA as i32);
        ffi::rlSetBlendFactors(RLGL_SRC_ALPHA, RLGL_SRC_ALPHA, RLGL_MAX);
        ffi::rlSetBlendMode(ffi::BlendMode::BLEND_CUSTOM as i32);
    }

    // Draw the shadows to the alpha mask
    for i in 0..(lights[slot].shadow_count as usize) {
        t.draw_triangle_fan(&lights[slot].shadows[i].vertices, Color::WHITE);
    }

    // SAFETY: rlgl batch-flush + blend mode restore; active draw frame holds.
    unsafe {
        ffi::rlDrawRenderBatchActive();

        // Go back to normal blend mode
        ffi::rlSetBlendMode(ffi::BlendMode::BLEND_ALPHA as i32);
    }
}

// Set up some boxes
fn setup_boxes(rl: &RaylibHandle, boxes: &mut [Rectangle], count: &mut usize) {
    boxes[0] = Rectangle::new(150.0, 80.0, 40.0, 40.0);
    boxes[1] = Rectangle::new(1200.0, 700.0, 40.0, 40.0);
    boxes[2] = Rectangle::new(200.0, 600.0, 40.0, 40.0);
    boxes[3] = Rectangle::new(1000.0, 50.0, 40.0, 40.0);
    boxes[4] = Rectangle::new(500.0, 350.0, 40.0, 40.0);

    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for i in 5..MAX_BOXES {
        boxes[i] = Rectangle::new(
            rl.get_random_value::<i32>(0..=rl.get_screen_width()) as f32,
            rl.get_random_value::<i32>(0..=rl.get_screen_height()) as f32,
            rl.get_random_value::<i32>(10..=100) as f32,
            rl.get_random_value::<i32>(10..=100) as f32,
        );
    }

    *count = MAX_BOXES;
}
