/*******************************************************************************************
*
*   raylib [shaders] example - deferred rendering
*
*   Example complexity rating: [★★★★] 4/4
*
*   NOTE: This example requires raylib OpenGL 3.3 or OpenGL ES 3.0
*
*   Example originally created with raylib 4.5, last time updated with raylib 4.5
*
*   Example contributed by Justin Andreas Lacoste (@27justin) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2023-2025 Justin Andreas Lacoste (@27justin)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// We always run on PLATFORM_DESKTOP via raylib-rs; mirror the GLSL_VERSION fork's desktop value.
const GLSL_VERSION: i32 = 330;

const MAX_LIGHTS: usize = 4;
const MAX_CUBES: usize = 30;

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
// GBuffer data
struct GBuffer {
    framebuffer_id: u32,
    position_texture_id: u32,
    normal_texture_id: u32,
    albedo_spec_texture_id: u32,
    depth_renderbuffer_id: u32,
}

// Deferred mode passes
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum DeferredMode {
    Position = 0,
    Normal = 1,
    Albedo = 2,
    Shading = 3,
}

//----------------------------------------------------------------------------------
// Light data (inline port of rlights.h)
//----------------------------------------------------------------------------------
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum LightType {
    Directional = 0,
    Point = 1,
}

#[derive(Clone, Copy)]
struct Light {
    light_type: i32,
    enabled: i32,
    position: Vector3,
    target: Vector3,
    color: Color,
    enabled_loc: i32,
    type_loc: i32,
    position_loc: i32,
    target_loc: i32,
    color_loc: i32,
}

fn create_light(
    index: usize,
    light_type: LightType,
    position: Vector3,
    target: Vector3,
    color: Color,
    shader: &mut Shader,
) -> Light {
    let mut light = Light {
        light_type: light_type as i32,
        enabled: 1,
        position,
        target,
        color,
        enabled_loc: shader.get_shader_location(&format!("lights[{index}].enabled")),
        type_loc: shader.get_shader_location(&format!("lights[{index}].type")),
        position_loc: shader.get_shader_location(&format!("lights[{index}].position")),
        target_loc: shader.get_shader_location(&format!("lights[{index}].target")),
        color_loc: shader.get_shader_location(&format!("lights[{index}].color")),
    };

    update_light_values(shader, &mut light);
    light
}

fn update_light_values(shader: &mut Shader, light: &mut Light) {
    shader.set_shader_value(light.enabled_loc, light.enabled);
    shader.set_shader_value(light.type_loc, light.light_type);
    shader.set_shader_value(light.position_loc, light.position);
    shader.set_shader_value(light.target_loc, light.target);
    let c = [
        light.color.r as f32 / 255.0,
        light.color.g as f32 / 255.0,
        light.color.b as f32 / 255.0,
        light.color.a as f32 / 255.0,
    ];
    shader.set_shader_value(light.color_loc, Vector4::new(c[0], c[1], c[2], c[3]));
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    // -------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shaders] example - deferred rendering")
        .build();

    let mut camera = Camera3D::perspective(
        Vector3::new(5.0, 4.0, 5.0), // Camera position
        Vector3::new(0.0, 1.0, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        60.0,                        // Camera field-of-view Y
    );

    // Load plane model from a generated mesh
    // SAFETY: make_weak transfers Mesh ownership to the Model below.
    let plane_mesh = unsafe { Mesh::gen_mesh_plane(&thread, 10.0, 10.0, 3, 3).make_weak() };
    let mut model = rl.load_model_from_mesh(&thread, plane_mesh).unwrap();
    let cube_mesh = unsafe { Mesh::gen_mesh_cube(&thread, 2.0, 2.0, 2.0).make_weak() };
    let mut cube = rl.load_model_from_mesh(&thread, cube_mesh).unwrap();

    // Load geometry buffer (G-buffer) shader and deferred shader
    let gbuffer_shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/gbuffer.vs"
        )),
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/gbuffer.fs"
        )),
    );

    let mut deferred_shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/deferred_shading.vs"
        )),
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/deferred_shading.fs"
        )),
    );
    // SAFETY: Shader.locs offset write (SHADER_LOC_VECTOR_VIEW).
    unsafe {
        let view_loc = deferred_shader.get_shader_location("viewPosition");
        *deferred_shader
            .as_raw_mut()
            .locs
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_VECTOR_VIEW as isize) = view_loc;
    }

    // Initialize the G-buffer
    // SAFETY: rlgl framebuffer/texture entry points are valid after InitWindow. We unload
    // everything in the De-Init block below.
    let mut g_buffer = GBuffer {
        framebuffer_id: 0,
        position_texture_id: 0,
        normal_texture_id: 0,
        albedo_spec_texture_id: 0,
        depth_renderbuffer_id: 0,
    };

    unsafe {
        g_buffer.framebuffer_id = ffi::rlLoadFramebuffer();
        if g_buffer.framebuffer_id == 0 {
            println!("WARNING: Failed to create framebufferId");
        }

        ffi::rlEnableFramebuffer(g_buffer.framebuffer_id);

        // NOTE: Vertex positions are stored in a texture for simplicity. A better approach would use a depth texture
        // (instead of a depth renderbuffer) to reconstruct world positions in the final render shader via clip-space position,
        // depth, and the inverse view/projection matrices

        // 16-bit precision ensures OpenGL ES 3 compatibility, though it may lack precision for real scenarios
        g_buffer.position_texture_id = ffi::rlLoadTexture(
            std::ptr::null(),
            screen_width,
            screen_height,
            ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R16G16B16 as i32,
            1,
        );

        // Similarly, 16-bit precision is used for normals ensures OpenGL ES 3 compatibility
        g_buffer.normal_texture_id = ffi::rlLoadTexture(
            std::ptr::null(),
            screen_width,
            screen_height,
            ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R16G16B16 as i32,
            1,
        );

        // Albedo (diffuse color) and specular strength can be combined into one texture
        g_buffer.albedo_spec_texture_id = ffi::rlLoadTexture(
            std::ptr::null(),
            screen_width,
            screen_height,
            ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32,
            1,
        );

        // Activate the draw buffers for our framebufferId
        ffi::rlActiveDrawBuffers(3);

        // Now we attach our textures to the framebufferId
        ffi::rlFramebufferAttach(
            g_buffer.framebuffer_id,
            g_buffer.position_texture_id,
            ffi::rlFramebufferAttachType::RL_ATTACHMENT_COLOR_CHANNEL0 as i32,
            ffi::rlFramebufferAttachTextureType::RL_ATTACHMENT_TEXTURE2D as i32,
            0,
        );
        ffi::rlFramebufferAttach(
            g_buffer.framebuffer_id,
            g_buffer.normal_texture_id,
            ffi::rlFramebufferAttachType::RL_ATTACHMENT_COLOR_CHANNEL1 as i32,
            ffi::rlFramebufferAttachTextureType::RL_ATTACHMENT_TEXTURE2D as i32,
            0,
        );
        ffi::rlFramebufferAttach(
            g_buffer.framebuffer_id,
            g_buffer.albedo_spec_texture_id,
            ffi::rlFramebufferAttachType::RL_ATTACHMENT_COLOR_CHANNEL2 as i32,
            ffi::rlFramebufferAttachTextureType::RL_ATTACHMENT_TEXTURE2D as i32,
            0,
        );

        // Finally we attach the depth buffer
        g_buffer.depth_renderbuffer_id = ffi::rlLoadTextureDepth(screen_width, screen_height, true);
        ffi::rlFramebufferAttach(
            g_buffer.framebuffer_id,
            g_buffer.depth_renderbuffer_id,
            ffi::rlFramebufferAttachType::RL_ATTACHMENT_DEPTH as i32,
            ffi::rlFramebufferAttachTextureType::RL_ATTACHMENT_RENDERBUFFER as i32,
            0,
        );

        // Make sure our framebufferId is complete
        if !ffi::rlFramebufferComplete(g_buffer.framebuffer_id) {
            println!("WARNING: Framebuffer is not complete");
        }

        // Now we initialize the sampler2D uniform's in the deferred shader
        // We do this by setting the uniform's values to the texture units that
        // we later bind our g-buffer textures to
        ffi::rlEnableShader(deferred_shader.as_ref().id);
        let tex_unit_position: i32 = 0;
        let tex_unit_normal: i32 = 1;
        let tex_unit_albedo_spec: i32 = 2;
        // SAFETY: deferred_shader.id is valid; uniforms are sampler2D.
        let cs_pos = std::ffi::CString::new("gPosition").unwrap();
        let cs_n = std::ffi::CString::new("gNormal").unwrap();
        let cs_as = std::ffi::CString::new("gAlbedoSpec").unwrap();
        let pos_loc = ffi::rlGetLocationUniform(deferred_shader.as_ref().id, cs_pos.as_ptr());
        let n_loc = ffi::rlGetLocationUniform(deferred_shader.as_ref().id, cs_n.as_ptr());
        let as_loc = ffi::rlGetLocationUniform(deferred_shader.as_ref().id, cs_as.as_ptr());
        deferred_shader.set_shader_value(pos_loc, tex_unit_position);
        deferred_shader.set_shader_value(n_loc, tex_unit_normal);
        deferred_shader.set_shader_value(as_loc, tex_unit_albedo_spec);
        ffi::rlDisableShader();
    }

    // Assign our lighting shader to model
    model.materials_mut()[0].set_shader(&gbuffer_shader);
    cube.materials_mut()[0].set_shader(&gbuffer_shader);

    // Create lights
    //--------------------------------------------------------------------------------------
    let mut lights: [Light; MAX_LIGHTS] = [
        create_light(
            0,
            LightType::Point,
            Vector3::new(-2.0, 1.0, -2.0),
            Vector3::zero(),
            Color::YELLOW,
            &mut deferred_shader,
        ),
        create_light(
            1,
            LightType::Point,
            Vector3::new(2.0, 1.0, 2.0),
            Vector3::zero(),
            Color::RED,
            &mut deferred_shader,
        ),
        create_light(
            2,
            LightType::Point,
            Vector3::new(-2.0, 1.0, 2.0),
            Vector3::zero(),
            Color::GREEN,
            &mut deferred_shader,
        ),
        create_light(
            3,
            LightType::Point,
            Vector3::new(2.0, 1.0, -2.0),
            Vector3::zero(),
            Color::BLUE,
            &mut deferred_shader,
        ),
    ];

    const CUBE_SCALE: f32 = 0.25;
    let mut cube_positions: [Vector3; MAX_CUBES] = [Vector3::zero(); MAX_CUBES];
    let mut cube_rotations: [f32; MAX_CUBES] = [0.0; MAX_CUBES];

    for i in 0..MAX_CUBES {
        cube_positions[i] = Vector3::new(
            (rl.get_random_value::<i32>(0..=9) - 5) as f32,
            rl.get_random_value::<i32>(0..=4) as f32,
            (rl.get_random_value::<i32>(0..=9) - 5) as f32,
        );

        cube_rotations[i] = rl.get_random_value::<i32>(0..=359) as f32;
    }

    let mut mode = DeferredMode::Shading;

    // SAFETY: pure rlgl state toggle.
    unsafe {
        ffi::rlEnableDepthTest();
    }

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //---------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close() {
        // Update
        //----------------------------------------------------------------------------------
        camera.update_camera(CameraMode::CAMERA_ORBITAL);

        // Update the shader with the camera view vector (points towards { 0.0f, 0.0f, 0.0f })
        let camera_pos = Vector3::new(camera.position.x, camera.position.y, camera.position.z);
        // SAFETY: Shader.locs offset access (SHADER_LOC_VECTOR_VIEW).
        let view_loc_now = unsafe {
            *deferred_shader
                .as_ref()
                .locs
                .offset(ffi::ShaderLocationIndex::SHADER_LOC_VECTOR_VIEW as isize)
        };
        deferred_shader.set_shader_value(view_loc_now, camera_pos);

        // Check key inputs to enable/disable lights
        if rl.is_key_pressed(KeyboardKey::KEY_Y) {
            lights[0].enabled = if lights[0].enabled != 0 { 0 } else { 1 };
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            lights[1].enabled = if lights[1].enabled != 0 { 0 } else { 1 };
        }
        if rl.is_key_pressed(KeyboardKey::KEY_G) {
            lights[2].enabled = if lights[2].enabled != 0 { 0 } else { 1 };
        }
        if rl.is_key_pressed(KeyboardKey::KEY_B) {
            lights[3].enabled = if lights[3].enabled != 0 { 0 } else { 1 };
        }

        // Check key inputs to switch between G-buffer textures
        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            mode = DeferredMode::Position;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            mode = DeferredMode::Normal;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
            mode = DeferredMode::Albedo;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_FOUR) {
            mode = DeferredMode::Shading;
        }

        // Update light values (actually, only enable/disable them)
        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for i in 0..MAX_LIGHTS {
            update_light_values(&mut deferred_shader, &mut lights[i]);
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        // ---------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        // Draw to the geometry buffer by first activating it
        // SAFETY: rlgl framebuffer state matched by rlDisableFramebuffer below.
        unsafe {
            ffi::rlEnableFramebuffer(g_buffer.framebuffer_id);
            ffi::rlClearColor(0, 0, 0, 0);
            ffi::rlClearScreenBuffers(); // Clear color and depth buffer
            ffi::rlDisableColorBlend();
        }

        {
            let mut c = d.begin_mode3D(camera);
            // NOTE: We have to use rlEnableShader here. `BeginShaderMode` or thus `rlSetShader`
            // will not work, as they won't immediately load the shader program
            // SAFETY: rlEnableShader/rlDisableShader pair, matched.
            unsafe {
                ffi::rlEnableShader(gbuffer_shader.as_ref().id);
            }
            // When drawing a model here, make sure that the material's shaders are set to the gbuffer shader!
            c.draw_model(&model, Vector3::zero(), 1.0, Color::WHITE);
            c.draw_model(&cube, Vector3::new(0.0, 1.0, 0.0), 1.0, Color::WHITE);

            for i in 0..MAX_CUBES {
                let position = cube_positions[i];
                c.draw_model_ex(
                    &cube,
                    position,
                    Vector3::new(1.0, 1.0, 1.0),
                    cube_rotations[i],
                    Vector3::new(CUBE_SCALE, CUBE_SCALE, CUBE_SCALE),
                    Color::WHITE,
                );
            }
            // SAFETY: matched rlDisableShader.
            unsafe {
                ffi::rlDisableShader();
            }
        }

        // SAFETY: paired with rlEnableFramebuffer above.
        unsafe {
            ffi::rlEnableColorBlend();

            // Go back to the default framebufferId (0) and draw our deferred shading
            ffi::rlDisableFramebuffer();
            ffi::rlClearScreenBuffers(); // Clear color & depth buffer
        }

        match mode {
            DeferredMode::Shading => {
                {
                    let _c = d.begin_mode3D(camera);
                    // SAFETY: deferred shading pass. rlEnableShader/rlDisableShader matched.
                    unsafe {
                        ffi::rlDisableColorBlend();
                        ffi::rlEnableShader(deferred_shader.as_ref().id);

                        // Bind our g-buffer textures
                        ffi::rlActiveTextureSlot(0);
                        ffi::rlEnableTexture(g_buffer.position_texture_id);
                        ffi::rlActiveTextureSlot(1);
                        ffi::rlEnableTexture(g_buffer.normal_texture_id);
                        ffi::rlActiveTextureSlot(2);
                        ffi::rlEnableTexture(g_buffer.albedo_spec_texture_id);

                        // Finally, we draw a fullscreen quad to our default framebufferId
                        // This will now be shaded using our deferred shader
                        ffi::rlLoadDrawQuad();
                        ffi::rlDisableShader();
                        ffi::rlEnableColorBlend();
                    }
                }

                // SAFETY: copy depth buffer from g-buffer to default framebuffer.
                unsafe {
                    ffi::rlBindFramebuffer(ffi::RL_READ_FRAMEBUFFER, g_buffer.framebuffer_id);
                    ffi::rlBindFramebuffer(ffi::RL_DRAW_FRAMEBUFFER, 0);
                    ffi::rlBlitFramebuffer(
                        0,
                        0,
                        screen_width,
                        screen_height,
                        0,
                        0,
                        screen_width,
                        screen_height,
                        0x00000100, // GL_DEPTH_BUFFER_BIT
                    );
                    ffi::rlDisableFramebuffer();
                }

                // Since our shader is now done and disabled, we can draw spheres
                // that represent light positions in default forward rendering
                {
                    let mut c = d.begin_mode3D(camera);
                    // SAFETY: pure rlgl state.
                    unsafe {
                        ffi::rlEnableShader(ffi::rlGetShaderIdDefault());
                    }
                    #[expect(
                        clippy::needless_range_loop,
                        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
                    )]
                    for i in 0..MAX_LIGHTS {
                        if lights[i].enabled != 0 {
                            c.draw_sphere_ex(lights[i].position, 0.2, 8, 8, lights[i].color);
                        } else {
                            c.draw_sphere_wires(
                                lights[i].position,
                                0.2,
                                8,
                                8,
                                lights[i].color.alpha(0.3),
                            );
                        }
                    }
                    // SAFETY: matched rlDisableShader.
                    unsafe {
                        ffi::rlDisableShader();
                    }
                }

                d.draw_text("FINAL RESULT", 10, screen_height - 30, 20, Color::DARKGREEN);
            }
            DeferredMode::Position => {
                // SAFETY: temporary Texture view of the g-buffer position texture.
                let tex_handle = ffi::Texture {
                    id: g_buffer.position_texture_id,
                    width: screen_width,
                    height: screen_height,
                    mipmaps: 1,
                    format: ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R16G16B16 as i32,
                };
                let tex_weak: &WeakTexture2D = unsafe { std::mem::transmute(&tex_handle) };
                d.draw_texture_rec(
                    tex_weak,
                    Rectangle::new(0.0, 0.0, screen_width as f32, -(screen_height as f32)),
                    Vector2::zero(),
                    Color::RAYWHITE,
                );
                d.draw_text(
                    "POSITION TEXTURE",
                    10,
                    screen_height - 30,
                    20,
                    Color::DARKGREEN,
                );
            }
            DeferredMode::Normal => {
                let tex_handle = ffi::Texture {
                    id: g_buffer.normal_texture_id,
                    width: screen_width,
                    height: screen_height,
                    mipmaps: 1,
                    format: ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R16G16B16 as i32,
                };
                let tex_weak: &WeakTexture2D = unsafe { std::mem::transmute(&tex_handle) };
                d.draw_texture_rec(
                    tex_weak,
                    Rectangle::new(0.0, 0.0, screen_width as f32, -(screen_height as f32)),
                    Vector2::zero(),
                    Color::RAYWHITE,
                );
                d.draw_text(
                    "NORMAL TEXTURE",
                    10,
                    screen_height - 30,
                    20,
                    Color::DARKGREEN,
                );
            }
            DeferredMode::Albedo => {
                let tex_handle = ffi::Texture {
                    id: g_buffer.albedo_spec_texture_id,
                    width: screen_width,
                    height: screen_height,
                    mipmaps: 1,
                    format: ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32,
                };
                let tex_weak: &WeakTexture2D = unsafe { std::mem::transmute(&tex_handle) };
                d.draw_texture_rec(
                    tex_weak,
                    Rectangle::new(0.0, 0.0, screen_width as f32, -(screen_height as f32)),
                    Vector2::zero(),
                    Color::RAYWHITE,
                );
                d.draw_text(
                    "ALBEDO TEXTURE",
                    10,
                    screen_height - 30,
                    20,
                    Color::DARKGREEN,
                );
            }
        }

        d.draw_text(
            "Toggle lights keys: [Y][R][G][B]",
            10,
            40,
            20,
            Color::DARKGRAY,
        );
        d.draw_text(
            "Switch G-buffer textures: [1][2][3][4]",
            10,
            70,
            20,
            Color::DARKGRAY,
        );

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        // -----------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // Unbind shader from materials so UnloadModel doesn't double-unload it (gbuffer_shader RAII owns it).
    model.materials_mut()[0].clear_shader();
    cube.materials_mut()[0].clear_shader();

    // Unload geometry buffer and all attached textures
    // SAFETY: matched against the rlgl loaders above.
    unsafe {
        ffi::rlUnloadFramebuffer(g_buffer.framebuffer_id);
        ffi::rlUnloadTexture(g_buffer.position_texture_id);
        ffi::rlUnloadTexture(g_buffer.normal_texture_id);
        ffi::rlUnloadTexture(g_buffer.albedo_spec_texture_id);
        ffi::rlUnloadTexture(g_buffer.depth_renderbuffer_id);
    }
    // UnloadModel / UnloadShader / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
