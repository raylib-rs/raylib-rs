/*******************************************************************************************
*
*   raylib [shaders] example - shadowmap rendering
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 5.0, last time updated with raylib 5.0
*
*   Example contributed by TheManTheMythTheGameDev (@TheManTheMythTheGameDev) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2023-2025 TheManTheMythTheGameDev (@TheManTheMythTheGameDev)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// We always run on PLATFORM_DESKTOP via raylib-rs; mirror the GLSL_VERSION fork's desktop value.
const GLSL_VERSION: i32 = 330;

const SHADOWMAP_RESOLUTION: i32 = 1024;

// Load render texture for shadowmap projection
// NOTE: Load framebuffer with only a texture depth attachment,
// no color attachment required for shadowmap
fn load_shadowmap_render_texture(width: i32, height: i32) -> ffi::RenderTexture2D {
    let mut target = ffi::RenderTexture2D {
        id: 0,
        texture: ffi::Texture {
            id: 0,
            width: 0,
            height: 0,
            mipmaps: 0,
            format: 0,
        },
        depth: ffi::Texture {
            id: 0,
            width: 0,
            height: 0,
            mipmaps: 0,
            format: 0,
        },
    };

    // SAFETY: rlgl framebuffer/texture entry points are valid after InitWindow.
    unsafe {
        target.id = ffi::rlLoadFramebuffer(); // Load an empty framebuffer
        target.texture.width = width;
        target.texture.height = height;

        if target.id > 0 {
            ffi::rlEnableFramebuffer(target.id);

            // Create depth texture
            // NOTE: No need a color texture attachment for the shadowmap
            target.depth.id = ffi::rlLoadTextureDepth(width, height, false);
            target.depth.width = width;
            target.depth.height = height;
            target.depth.format = 19; // DEPTH_COMPONENT_24BIT?
            target.depth.mipmaps = 1;

            // Attach depth texture to FBO
            ffi::rlFramebufferAttach(
                target.id,
                target.depth.id,
                ffi::rlFramebufferAttachType::RL_ATTACHMENT_DEPTH as i32,
                ffi::rlFramebufferAttachTextureType::RL_ATTACHMENT_TEXTURE2D as i32,
                0,
            );

            // Check if fbo is complete with attachments (valid)
            if ffi::rlFramebufferComplete(target.id) {
                println!(
                    "INFO: FBO: [ID {}] Framebuffer object created successfully",
                    target.id
                );
            }

            ffi::rlDisableFramebuffer();
        } else {
            println!("WARNING: FBO: Framebuffer object can not be created");
        }
    }

    target
}

// Unload shadowmap render texture from GPU memory (VRAM)
fn unload_shadowmap_render_texture(target: &ffi::RenderTexture2D) {
    if target.id > 0 {
        // NOTE: Depth texture/renderbuffer is automatically
        // queried and deleted before deleting framebuffer
        // SAFETY: matched against load_shadowmap_render_texture.
        unsafe {
            ffi::rlUnloadFramebuffer(target.id);
        }
    }
}

// Draw full scene projecting shadows
// NOTE: Required  to be called several time to generate shadowmap
fn draw_scene<D: RaylibDraw3D>(d: &mut D, cube: &Model, robot: &Model) {
    d.draw_model_ex(
        cube,
        Vector3::zero(),
        Vector3::new(0.0, 1.0, 0.0),
        0.0,
        Vector3::new(10.0, 1.0, 10.0),
        Color::BLUE,
    );
    d.draw_model_ex(
        cube,
        Vector3::new(1.5, 1.0, -1.5),
        Vector3::new(0.0, 1.0, 0.0),
        0.0,
        Vector3::one(),
        Color::WHITE,
    );
    d.draw_model_ex(
        robot,
        Vector3::new(0.0, 0.5, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        0.0,
        Vector3::new(1.0, 1.0, 1.0),
        Color::RED,
    );
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    // Shadows are a HUGE topic, and this example shows an extremely simple implementation of the shadowmapping algorithm,
    // which is the industry standard for shadows. This algorithm can be extended in a ridiculous number of ways to improve
    // realism and also adapt it for different scenes. This is pretty much the simplest possible implementation

    // SetConfigFlags(FLAG_MSAA_4X_HINT)
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shaders] example - shadowmap rendering")
        .msaa_4x()
        .build();

    let mut camera = Camera3D::perspective(
        Vector3::new(10.0, 10.0, 10.0),
        Vector3::zero(),
        Vector3::new(0.0, 1.0, 0.0),
        45.0,
    );

    let mut shadow_shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/shadowmap.vs"
        )),
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/shadowmap.fs"
        )),
    );
    let view_pos_loc = shadow_shader.get_shader_location("viewPos");
    // SAFETY: write shader.locs[SHADER_LOC_VECTOR_VIEW]; backing array lives as long as the shader.
    unsafe {
        *shadow_shader
            .as_raw_mut()
            .locs
            .offset(ffi::ShaderLocationIndex::SHADER_LOC_VECTOR_VIEW as isize) = view_pos_loc;
    }

    let mut light_dir = Vector3::new(0.35, -1.0, -0.35).normalize();
    let light_color = Color::WHITE;
    let light_color_normalized = Vector4::new(
        light_color.r as f32 / 255.0,
        light_color.g as f32 / 255.0,
        light_color.b as f32 / 255.0,
        light_color.a as f32 / 255.0,
    );
    let light_dir_loc = shadow_shader.get_shader_location("lightDir");
    let light_col_loc = shadow_shader.get_shader_location("lightColor");
    shadow_shader.set_shader_value(light_dir_loc, light_dir);
    shadow_shader.set_shader_value(light_col_loc, light_color_normalized);
    let ambient_loc = shadow_shader.get_shader_location("ambient");
    let ambient = [0.1f32, 0.1, 0.1, 1.0];
    shadow_shader.set_shader_value(ambient_loc, ambient);
    let light_vp_loc = shadow_shader.get_shader_location("lightVP");
    let shadow_map_loc = shadow_shader.get_shader_location("shadowMap");
    let shadow_map_resolution: i32 = SHADOWMAP_RESOLUTION;
    let smr_loc = shadow_shader.get_shader_location("shadowMapResolution");
    shadow_shader.set_shader_value(smr_loc, shadow_map_resolution);

    let cube_mesh = Mesh::gen_mesh_cube(&thread, 1.0, 1.0, 1.0);
    let mut cube = rl
        .load_model_from_mesh(&thread, unsafe { cube_mesh.make_weak() })
        .unwrap();
    cube.materials_mut()[0].set_shader(&shadow_shader);
    let mut robot = rl
        .load_model(&thread, "resources/shaders/models/robot.glb")
        .unwrap();
    let mat_count = robot.materials().len();
    for i in 0..mat_count {
        robot.materials_mut()[i].set_shader(&shadow_shader);
    }

    let mut anims = rl
        .load_model_animations(&thread, "resources/shaders/models/robot.glb")
        .unwrap();

    let shadow_map = load_shadowmap_render_texture(SHADOWMAP_RESOLUTION, SHADOWMAP_RESOLUTION);

    // For the shadowmapping algorithm, we will be rendering everything from the light's point of view
    let mut light_camera = Camera3D::orthographic(
        light_dir.scale(-15.0),
        Vector3::zero(),
        Vector3::new(0.0, 1.0, 0.0),
        20.0,
    );

    let mut frame_counter: i32 = 0;

    // Store the light matrices
    let mut light_view;
    let mut light_proj;
    let mut light_view_proj;
    let texture_active_slot: i32 = 10; // Can be anything 0 to 15, but 0 will probably be taken up

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let delta_time = rl.get_frame_time();

        let camera_pos = camera.position;
        shadow_shader.set_shader_value(view_pos_loc, camera_pos);
        camera.update_camera(CameraMode::CAMERA_ORBITAL);

        frame_counter += 1;
        let keyframe_count = anims.iter().next().map(|a| a.keyframeCount).unwrap_or(1);
        frame_counter %= keyframe_count;
        // SAFETY: anims[0] is a valid loaded animation; update_model_animation expects a pointer-like
        // borrow.
        unsafe {
            ffi::UpdateModelAnimation(*robot.as_ref(), *anims[0].as_ref(), frame_counter as f32);
        }

        // Move light with arrow keys
        let camera_speed = 0.05f32;
        if rl.is_key_down(KeyboardKey::KEY_LEFT) && light_dir.x < 0.6 {
            light_dir.x += camera_speed * 60.0 * delta_time;
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) && light_dir.x > -0.6 {
            light_dir.x -= camera_speed * 60.0 * delta_time;
        }
        if rl.is_key_down(KeyboardKey::KEY_UP) && light_dir.z < 0.6 {
            light_dir.z += camera_speed * 60.0 * delta_time;
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) && light_dir.z > -0.6 {
            light_dir.z -= camera_speed * 60.0 * delta_time;
        }

        light_dir = light_dir.normalize();
        light_camera.position = light_dir.scale(-15.0);
        shadow_shader.set_shader_value(light_dir_loc, light_dir);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        // PASS 01: Render all objects into the shadowmap render texture
        // We record all the objects' depths (as rendered from the light source's point of view) in a buffer
        // Anything that is "visible" to the light is in light, anything that isn't is in shadow
        // We can later use the depth buffer when rendering everything from the player's point of view
        // to determine whether a given point is "visible" to the light
        // SAFETY: render to the shadowmap depth-only framebuffer.
        unsafe {
            ffi::rlEnableFramebuffer(shadow_map.id);
            ffi::rlViewport(0, 0, SHADOWMAP_RESOLUTION, SHADOWMAP_RESOLUTION);
            ffi::rlClearColor(255, 255, 255, 255);
            ffi::rlClearScreenBuffers();
        }

        // SAFETY: capture the light view/projection matrices while light_camera is the active 3D camera.
        unsafe {
            ffi::BeginMode3D(light_camera.into());
            light_view = ffi::rlGetMatrixModelview();
            light_proj = ffi::rlGetMatrixProjection();
        }

        let mut d = rl.begin_drawing(&thread);
        // We've already begun a Mode3D outside; use a separate draw helper that issues raw DrawModelEx via d.
        {
            let mut c = d.begin_mode3D(light_camera);
            draw_scene(&mut c, &cube, &robot);
        }
        // SAFETY: end the shadow framebuffer pass and the manually-started Mode3D.
        unsafe {
            ffi::EndMode3D();
            ffi::rlDisableFramebuffer();
        }
        light_view_proj = light_view * light_proj;

        // PASS 02: Draw the scene into main framebuffer, using the generated shadowmap
        d.clear_background(Color::RAYWHITE);

        shadow_shader.set_shader_value_matrix(light_vp_loc, light_view_proj);
        // SAFETY: bind shadow map depth texture into the chosen active slot for the shader.
        unsafe {
            ffi::rlEnableShader(shadow_shader.as_ref().id);

            ffi::rlActiveTextureSlot(texture_active_slot);
            ffi::rlEnableTexture(shadow_map.depth.id);
            ffi::rlSetUniform(
                shadow_map_loc,
                &texture_active_slot as *const i32 as *const std::ffi::c_void,
                ffi::ShaderUniformDataType::SHADER_UNIFORM_INT as i32,
                1,
            );
        }

        {
            let mut c = d.begin_mode3D(camera);
            draw_scene(&mut c, &cube, &robot); // Draw the same exact things as we drew in the shadowmap!
        }

        d.draw_text(
            "Use the arrow keys to rotate the light!",
            10,
            10,
            30,
            Color::RED,
        );
        d.draw_text(
            "Shadows in raylib using the shadowmapping algorithm!",
            screen_width - 280,
            screen_height - 20,
            10,
            Color::GRAY,
        );

        viewer.draw(&mut d);
        // Drop d here (EndDrawing)
        drop(d);

        if rl.is_key_pressed(KeyboardKey::KEY_F) {
            rl.take_screenshot(&thread, "shaders_shadowmap.png");
        }
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // Unbind shader from materials so model Drop doesn't double-free.
    // SAFETY: shader is an inline value field — writing it cannot corrupt the maps pointer.
    unsafe {
        let null_shader = ffi::Shader {
            id: 0,
            locs: std::ptr::null_mut(),
        };
        cube.materials_mut()[0].as_raw_mut().shader = null_shader;
        let mat_count = robot.materials().len();
        for i in 0..mat_count {
            robot.materials_mut()[i].as_raw_mut().shader = null_shader;
        }
    }
    let _ = &mut anims; // ModelAnimations are unloaded via RAII drop
    unload_shadowmap_render_texture(&shadow_map);
    // UnloadShader / UnloadModel / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
