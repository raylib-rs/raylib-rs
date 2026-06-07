/*******************************************************************************************
*
*   raylib [models] example - loading iqm
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 3.5
*
*   Example contributed by Culacant (@culacant) and reviewed by Ramon Santamaria (@raysan5)
*
*   NOTES: To export an IQM model from blender, make sure it is not posed, the vertices need
*   to be in the same position as they would be in edit mode and the scale of the models is
*   set to 0; scaling can be set from the export menu
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Culacant (@culacant) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

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
        .title("raylib [models] example - loading iqm")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(10.0, 10.0, 10.0), // Camera position
        Vector3::new(0.0, 4.0, 0.0),    // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0),    // Camera up vector (rotation towards target)
        45.0,                           // Camera field-of-view Y
    );

    let mut model = rl
        .load_model(&thread, "resources/models/models/iqm/guy.iqm")
        .unwrap(); // Load the animated model mesh and basic data
    let texture = rl
        .load_texture(&thread, "resources/models/models/iqm/guytex.png")
        .unwrap(); // Load model texture and set material
    model.materials_mut()[0].set_material_texture(MATERIAL_MAP_ALBEDO, &texture); // Set model material map texture
    let position = Vector3::new(0.0, 0.0, 0.0); // Set model position

    // Load animation data
    let anims = rl
        .load_model_animations(&thread, "resources/models/models/iqm/guyanim.iqm")
        .unwrap();

    // Animation playing variables
    let anim_index: usize = 0; // Current animation playing
    let mut anim_current_frame: f32 = 0.0; // Current animation frame (supporting interpolated frames)

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

        // Play animation when spacebar is held down
        anim_current_frame += 1.0;
        rl.update_model_animation(&thread, &mut model, &anims[0], anim_current_frame);
        if anim_current_frame >= anims[0].keyframeCount as f32 {
            anim_current_frame = 0.0;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            c.draw_model_ex(
                &model,
                position,
                Vector3::new(1.0, 0.0, 0.0),
                -90.0,
                Vector3::new(1.0, 1.0, 1.0),
                Color::WHITE,
            );

            c.draw_grid(10, 1.0);
        }

        // SAFETY: anims[anim_index].name is a null-terminated inline char[32] embedded in the
        // ffi::ModelAnimation; the buffer lives for the lifetime of `anims`.
        let anim_name = unsafe {
            std::ffi::CStr::from_ptr(anims[anim_index].name.as_ptr())
                .to_string_lossy()
                .into_owned()
        };
        d.draw_text(
            &format!("Current animation: {anim_name}"),
            10,
            10,
            20,
            Color::MAROON,
        );
        d.draw_text(
            "(c) Guy IQM 3D model by @culacant",
            screen_width - 200,
            screen_height - 20,
            10,
            Color::GRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture / UnloadModelAnimations / UnloadModel / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
