/*******************************************************************************************
*
*   raylib [others] example - OpenGL interoperatibility
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 3.8, last time updated with raylib 4.0
*
*   Example contributed by Stephan Soller (@arkanis) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2021-2025 Stephan Soller (@arkanis) and Ramon Santamaria (@raysan5)
*
********************************************************************************************
*
*   Mixes raylib and plain OpenGL code to draw a GL_POINTS based particle system. The
*   primary point is to demonstrate raylib and OpenGL interop
*
*   rlgl batched draw operations internally so we have to flush the current batch before
*   doing our own OpenGL work (rlDrawRenderBatchActive())
*
*   The example also demonstrates how to get the current model view projection matrix of
*   raylib. That way raylib cameras and so on work as expected
*
********************************************************************************************/

use raylib::ffi;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// idiomatic: in C, GLSL_VERSION switches between 100 (ES2/web) and 330 (desktop GL3.3)
// at preprocessor time. In Rust we pick the same constant via `cfg(feature)`, mirroring
// what raylib-sys's opengl_* features select. Desktop GL2.1 also wants glsl120, but the
// upstream example only carries 100 / 330 paths so we mirror those.
// Cargo features on the dependent crate aren't visible here, so this picks the
// desktop path (GLSL 330) which matches the showcase's default build. If a future
// wasm port wires through opengl_es_20, the shader path can shift to glsl100.
const GLSL_VERSION: u32 = 330;

const MAX_PARTICLES: usize = 1000;

// idiomatic: a few core-GL symbols aren't exposed by raylib-sys's bindgen surface
// (binding.h includes raylib + raymath + rlgl + rcamera, but not <glad.h>). Two
// OpenGL 1.1-era entry points are enough here: `glDrawArrays` for GL_POINTS drawing
// and `glEnable` for GL_PROGRAM_POINT_SIZE. Both ship with the platform GL lib that
// raylib's own CMake already links (opengl32 on Windows, libGL on Linux, the
// OpenGL framework on macOS), so this `extern "C"` block resolves without any
// extra link directive.
const GL_POINTS: u32 = 0x0000;
const GL_PROGRAM_POINT_SIZE: u32 = 0x8642;

#[cfg_attr(windows, link(name = "opengl32"))]
#[cfg_attr(
    all(unix, not(target_os = "macos"), not(target_os = "android")),
    link(name = "GL")
)]
#[cfg_attr(target_os = "macos", link(name = "OpenGL", kind = "framework"))]
unsafe extern "C" {
    fn glDrawArrays(mode: u32, first: i32, count: i32);
    fn glEnable(cap: u32);
}

//------------------------------------------------------------------------------------
// Module Functions Declaration
//------------------------------------------------------------------------------------
// Particle type
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Particle {
    x: f32,
    y: f32,
    period: f32,
}

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
        .title("raylib [others] example - OpenGL interoperatibility")
        .build();

    let shader = rl.load_shader(
        &thread,
        Some(&format!(
            "resources/others/shaders/glsl{}/point_particle.vs",
            GLSL_VERSION
        )),
        Some(&format!(
            "resources/others/shaders/glsl{}/point_particle.fs",
            GLSL_VERSION
        )),
    );

    let current_time_loc = shader.get_shader_location("currentTime");
    let color_loc = shader.get_shader_location("color");

    // Initialize the vertex buffer for the particles and assign each particle random values
    let mut particles = [Particle::default(); MAX_PARTICLES];

    for i in 0..MAX_PARTICLES {
        particles[i].x = rl.get_random_value::<i32>(20..=(screen_width - 20)) as f32;
        particles[i].y = rl.get_random_value::<i32>(50..=(screen_height - 20)) as f32;

        // Give each particle a slightly different period. But don't spread it to much
        // This way the particles line up every so often and you get a glimps of what is going on
        particles[i].period = rl.get_random_value::<i32>(10..=30) as f32 / 10.0;
    }

    // Create a plain OpenGL vertex buffer with the data and an vertex array object
    // that feeds the data from the buffer into the vertexPosition shader attribute
    // idiomatic: rlgl wraps glGenVertexArrays/glGenBuffers/glBufferData behind
    // `rlLoadVertexArray` / `rlLoadVertexBuffer`, so we use those instead of going
    // direct to GL. This keeps the example portable to OpenGL ES 2.0 / Web where
    // bare `glGenVertexArraysOES` would need a runtime loader.
    let vao;
    let vbo;
    // SAFETY: rlgl ops are valid after raylib's window is initialized.
    unsafe {
        vao = ffi::rlLoadVertexArray();
        ffi::rlEnableVertexArray(vao);

        vbo = ffi::rlLoadVertexBuffer(
            particles.as_ptr() as *const std::ffi::c_void,
            (MAX_PARTICLES * std::mem::size_of::<Particle>()) as i32,
            false,
        );
        // Note: LoadShader() automatically fetches the attribute index of "vertexPosition" and saves it in shader.locs[SHADER_LOC_VERTEX_POSITION]
        let vertex_loc = *shader
            .locs
            .add(ffi::ShaderLocationIndex::SHADER_LOC_VERTEX_POSITION as usize);
        ffi::rlSetVertexAttribute(vertex_loc as u32, 3, ffi::RL_FLOAT as i32, false, 0, 0);
        ffi::rlEnableVertexAttribute(0);
        ffi::rlDisableVertexBuffer();
        ffi::rlDisableVertexArray();

        // Allows the vertex shader to set the point size of each particle individually
        // idiomatic: rlgl has no wrapper for GL_PROGRAM_POINT_SIZE (it's a desktop-GL
        // toggle that ES2 doesn't have); drop to raw glEnable for this one line.
        glEnable(GL_PROGRAM_POINT_SIZE);
    }

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        viewer.update(&mut rl, &thread);

        // Draw
        //----------------------------------------------------------------------------------
        let current_time = rl.get_time() as f32;
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);

        d.draw_rectangle(10, 10, 210, 30, Color::MAROON);
        d.draw_text(
            &format!("{} particles in one vertex buffer", MAX_PARTICLES),
            20,
            20,
            10,
            Color::RAYWHITE,
        );

        // SAFETY: rlgl/GL state mutation inside an active begin_drawing scope; vao/vbo
        // and `shader` are still live, and we restore default shader (id=0) after.
        unsafe {
            ffi::rlDrawRenderBatchActive(); // Draw iternal buffers data (previous draw calls)

            // Switch to plain OpenGL
            //------------------------------------------------------------------------------
            ffi::rlEnableShader(shader.id);

            ffi::rlSetUniform(
                current_time_loc,
                &current_time as *const f32 as *const std::ffi::c_void,
                ffi::ShaderUniformDataType::SHADER_UNIFORM_FLOAT as i32,
                1,
            );

            let color = Color::new(255, 0, 0, 128).color_normalize();
            ffi::rlSetUniform(
                color_loc,
                &color as *const Vector4 as *const std::ffi::c_void,
                ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4 as i32,
                1,
            );

            // Get the current modelview and projection matrix so the particle system is displayed and transformed
            // idiomatic: raymath `MatrixMultiply(a, b)` becomes `a * b` via the Matrix `Mul` impl.
            let model_view_projection = ffi::rlGetMatrixModelview() * ffi::rlGetMatrixProjection();

            let mvp_loc = *shader
                .locs
                .add(ffi::ShaderLocationIndex::SHADER_LOC_MATRIX_MVP as usize);
            ffi::rlSetUniformMatrix(mvp_loc, model_view_projection);

            ffi::rlEnableVertexArray(vao);
            glDrawArrays(GL_POINTS, 0, MAX_PARTICLES as i32);
            ffi::rlDisableVertexArray();

            ffi::rlDisableShader();
            //------------------------------------------------------------------------------
        }

        d.draw_fps(screen_width - 100, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // SAFETY: vbo/vao were created above with the matching rlLoadVertex* calls.
    unsafe {
        ffi::rlUnloadVertexBuffer(vbo);
        ffi::rlUnloadVertexArray(vao);
    }

    // UnloadShader and CloseWindow are handled by RAII drops of `shader` and `rl`.
    //--------------------------------------------------------------------------------------
    let _ = shader;
}
