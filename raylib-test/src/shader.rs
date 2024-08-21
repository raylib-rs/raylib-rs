#[cfg(test)]
mod shader_test {
    use crate::tests::*;
    use colored::Colorize;
    use raylib::prelude::*;
    ray_3d_draw_test!(test_multi_shader);
    fn test_multi_shader(
        d: &mut RaylibMode3D<RaylibDrawHandle>,
        thread: &RaylibThread,
        _: &TestAssets,
    ) {
        let imRed = Image::gen_image_color(800, 450, Color::new(255, 0, 0, 255));
        let texRed = d.load_texture_from_image(&thread, &imRed).unwrap();

        let imBlue = Image::gen_image_color(800, 450, Color::new(0, 0, 255, 255));
        let texBlue = d.load_texture_from_image(&thread, &imBlue).unwrap();

        let mut shader = d.load_shader(
            &thread,
            None,
            Some("resources/shaders/glsl330/color_mix.fs"),
        );

        // Get an additional sampler2D location to be enabled on drawing
        let texBlueLoc = shader.get_shader_location("texture1");

        // Get shader uniform for divider
        let dividerLoc = shader.get_shader_location("divider");
        let dividerValue = 0.5;

        let mut s = d.begin_shader_mode(&mut shader);

        s.set_inner_shader_value(dividerLoc, dividerValue);
        s.set_inner_shader_value_texture(texBlueLoc, &texBlue);

        s.clear_background(Color::WHITE);
        s.draw_texture(&texRed, 0, 0, Color::WHITE);
    }
}
