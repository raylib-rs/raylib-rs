/*******************************************************************************************
*
*   raylib [shaders] example - postprocessing
*
*   Example complexity rating: [★★★☆] 3/4
*
*   NOTE: This example requires raylib OpenGL 3.3 or ES2 versions for shaders support,
*         OpenGL 1.1 does not support shaders, recompile raylib to OpenGL 3.3 version
*
*   NOTE: Shaders used in this example are #version 330 (OpenGL 3.3), to test this example
*         on OpenGL ES 2.0 platforms (Android, Raspberry Pi, HTML5), use #version 100 shaders
*         raylib comes with shaders ready for both versions, check raylib/shaders install folder
*
*   Example originally created with raylib 1.3, last time updated with raylib 4.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2015-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::texture::RaylibTexture2D;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

#[cfg(target_family = "wasm")]
const GLSL_VERSION: i32 = 100;
#[cfg(not(target_family = "wasm"))]
const GLSL_VERSION: i32 = 330;

const MAX_POSTPRO_SHADERS: usize = 12;

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
#[expect(
    clippy::enum_variant_names,
    reason = "C-parity: variant names mirror the C enum prefix"
)]
enum PostproShader {
    FxGrayscale = 0,
    FxPosterization,
    FxDreamVision,
    FxPixelizer,
    FxCrossHatching,
    FxCrossStitching,
    FxPredatorView,
    FxScanlines,
    FxFisheye,
    FxSobel,
    FxBloom,
    FxBlur,
    //FX_FXAA
}

//------------------------------------------------------------------------------------
// Global Variables Definition
//------------------------------------------------------------------------------------
static POSTPRO_SHADER_TEXT: [&str; MAX_POSTPRO_SHADERS] = [
    "GRAYSCALE",
    "POSTERIZATION",
    "DREAM_VISION",
    "PIXELIZER",
    "CROSS_HATCHING",
    "CROSS_STITCHING",
    "PREDATOR_VIEW",
    "SCANLINES",
    "FISHEYE",
    "SOBEL",
    "BLOOM",
    "BLUR",
    //"FXAA"
];

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    // SetConfigFlags(FLAG_MSAA_4X_HINT) — Enable Multi Sampling Anti Aliasing 4x (if available)
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shaders] example - postprocessing")
        .msaa_4x()
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(2.0, 3.0, 2.0), // Camera position
        Vector3::new(0.0, 1.0, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        45.0,                        // Camera field-of-view Y
    );

    let mut model = rl
        .load_model(&thread, "resources/shaders/models/church.obj")
        .unwrap(); // Load OBJ model
    let texture = rl
        .load_texture(&thread, "resources/shaders/models/church_diffuse.png")
        .unwrap(); // Load model texture (diffuse map)
    // SAFETY: install diffuse texture into material[0]; Texture2D RAII keeps id alive.
    unsafe {
        (*model.materials_mut()[0]
            .as_mut()
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture = *texture.as_ref(); // Set model diffuse texture
    }

    let position = Vector3::new(0.0, 0.0, 0.0); // Set model position

    // Load all postpro shaders
    // NOTE 1: All postpro shader use the base vertex shader (DEFAULT_VERTEX_SHADER)
    // NOTE 2: We load the correct shader depending on GLSL version
    // NOTE: Defining 0 (NULL) for vertex shader forces usage of internal default vertex shader
    let names = [
        "grayscale",
        "posterization",
        "dream_vision",
        "pixelizer",
        "cross_hatching",
        "cross_stitching",
        "predator",
        "scanlines",
        "fisheye",
        "sobel",
        "bloom",
        "blur",
    ];
    let mut shaders: Vec<Shader> = Vec::with_capacity(MAX_POSTPRO_SHADERS);
    for name in names.iter() {
        shaders.push(rl.load_shader(
            &thread,
            None,
            Some(&format!(
                "resources/shaders/shaders/glsl{}/{}.fs",
                GLSL_VERSION, name
            )),
        ));
    }

    let mut current_shader: i32 = PostproShader::FxGrayscale as i32;

    // Create a RenderTexture2D to be used for render to texture
    let mut target = rl
        .load_render_texture(&thread, screen_width as u32, screen_height as u32)
        .unwrap();

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        camera.update_camera(CameraMode::CAMERA_ORBITAL);

        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            current_shader += 1;
        } else if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            current_shader -= 1;
        }

        if current_shader >= MAX_POSTPRO_SHADERS as i32 {
            current_shader = 0;
        } else if current_shader < 0 {
            current_shader = MAX_POSTPRO_SHADERS as i32 - 1;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let target_tex_w = target.texture().width();
        let target_tex_h = target.texture().height();
        let mut d = rl.begin_drawing(&thread);

        {
            let mut t = d.begin_texture_mode(&thread, &mut target); // Enable drawing to texture
            t.clear_background(Color::RAYWHITE); // Clear texture background

            {
                let mut c = t.begin_mode3D(camera); // Begin 3d mode drawing
                c.draw_model(&model, position, 0.1, Color::WHITE); // Draw 3d model with texture
                c.draw_grid(10, 1.0); // Draw a grid
            } // End 3d mode drawing, returns to orthographic 2d mode
        } // End drawing to texture (now we have a texture available for next passes)

        d.clear_background(Color::RAYWHITE); // Clear screen background

        // Render generated texture using selected postprocessing shader
        {
            let mut s = d.begin_shader_mode(&mut shaders[current_shader as usize]);
            // NOTE: Render texture must be y-flipped due to default OpenGL coordinates (left-bottom)
            s.draw_texture_rec(
                target.texture(),
                Rectangle::new(0.0, 0.0, target_tex_w as f32, -(target_tex_h as f32)),
                Vector2::new(0.0, 0.0),
                Color::WHITE,
            );
        }

        // Draw 2d shapes and text over drawn texture
        d.draw_rectangle(0, 9, 580, 30, Color::LIGHTGRAY.alpha(0.7));

        d.draw_text(
            "(c) Church 3D model by Alberto Cano",
            screen_width - 200,
            screen_height - 20,
            10,
            Color::GRAY,
        );
        d.draw_text("CURRENT POSTPRO SHADER:", 10, 15, 20, Color::BLACK);
        d.draw_text(
            POSTPRO_SHADER_TEXT[current_shader as usize],
            330,
            15,
            20,
            Color::RED,
        );
        d.draw_text("< >", 540, 10, 30, Color::DARKBLUE);
        d.draw_fps(700, 15);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // Unbind external diffuse texture from model.material so Drop doesn't double-free.
    // SAFETY: clear diffuse texture id in materials[0].
    unsafe {
        (*model.materials_mut()[0]
            .as_mut()
            .maps
            .offset(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO as isize))
        .texture
        .id = 0;
    }
    // UnloadShader (per element) / UnloadTexture / UnloadModel / UnloadRenderTexture / CloseWindow
    // handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
