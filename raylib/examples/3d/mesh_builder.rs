use glam::Vec2;
use rand::prelude::*;
use raylib::prelude::*;

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;
fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Hello, world!")
        .build();

    let mut camera = Camera3D::perspective(
        Vector3::new(4.0, 2.0, 4.0),
        Vector3::new(0.0, 1.8, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        60.0,
    );

    let mesh = Mesh::gen_mesh(
        &[
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 1.0),
        ],
        &[
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(1.0, 1.0),
        ],
    )
    .normals(&[
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    ])
    .colors(&[Color::RED, Color::GREEN, Color::BLUE])
    .build(&thread)
    .unwrap();

    let mut model = unsafe { rl.load_model_from_mesh(&thread, mesh.make_weak()).unwrap() };
    let mat = rl.load_material_default(&thread);

    rl.set_target_fps(60);

    while !rl.window_should_close() {
        camera.update_camera(CameraMode::CAMERA_FIRST_PERSON);
        let matrix = camera.view_matrix();

        rl.draw(&thread, |mut d| {
            d.clear_background(Color::DARKGREEN);
            d.draw_mode3D(camera, |mut d2| {
                d2.draw_plane(
                    Vector3::new(0.0, 0.0, 0.0),
                    Vector2::new(32.0, 32.0),
                    Color::LIGHTGRAY,
                );
                d2.draw_model(&model, Vector3::ZERO, 1.0, Color::RED);
            });
            d.draw_rectangle(10, 10, 220, 70, Color::SKYBLUE);
            d.draw_rectangle_lines(10, 10, 220, 70, Color::BLUE);
            d.draw_text(
                "First person camera default controls:",
                20,
                20,
                10,
                Color::BLACK,
            );
            d.draw_text("- Move with keys: W, A, S, D", 40, 40, 10, Color::DARKGRAY);
            d.draw_text("- Mouse move to look around", 40, 60, 10, Color::DARKGRAY);
        });
    }
}
