/*******************************************************************************************
*
*   raylib [textures] example - fog of war
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 4.2, last time updated with raylib 4.2
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2018-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAP_TILE_SIZE: i32 = 32; // Tiles size 32x32 pixels
const PLAYER_SIZE: i32 = 16; // Player size
const PLAYER_TILE_VISIBILITY: i32 = 2; // Player can see 2 tiles around its position

//----------------------------------------------------------------------------------
// Types and Structures Definition
//----------------------------------------------------------------------------------
// Map data type
struct Map {
    tiles_x: u32,      // Number of tiles in X axis
    tiles_y: u32,      // Number of tiles in Y axis
    tile_ids: Vec<u8>, // Tile ids (tilesX*tilesY), defines type of tile to draw
    tile_fog: Vec<u8>, // Tile fog state (tilesX*tilesY), defines if a tile has fog or half-fog
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
        .title("raylib [textures] example - fog of war")
        .build();

    let mut map = Map {
        tiles_x: 25,
        tiles_y: 15,
        tile_ids: Vec::new(),
        tile_fog: Vec::new(),
    };

    // NOTE: We can have up to 256 values for tile ids and for tile fog state,
    // probably we don't need that many values for fog state, it can be optimized
    // to use only 2 bits per fog state (reducing size by 4) but logic will be a bit more complex
    map.tile_ids = vec![0u8; (map.tiles_x * map.tiles_y) as usize];
    map.tile_fog = vec![0u8; (map.tiles_x * map.tiles_y) as usize];

    // Load map tiles (generating 2 random tile ids for testing)
    // NOTE: Map tile ids should be probably loaded from an external map file
    for i in 0..(map.tiles_y * map.tiles_x) {
        map.tile_ids[i as usize] = rl.get_random_value::<i32>(0..=1) as u8;
    }

    // Player position on the screen (pixel coordinates, not tile coordinates)
    let mut player_position = Vector2::new(180.0, 130.0);
    let mut player_tile_x: i32;
    let mut player_tile_y: i32;

    // Render texture to render fog of war
    // NOTE: To get an automatic smooth-fog effect we use a render texture to render fog
    // at a smaller size (one pixel per tile) and scale it on drawing with bilinear filtering
    let mut fog_of_war = rl
        .load_render_texture(&thread, map.tiles_x, map.tiles_y)
        .unwrap();
    fog_of_war
        .texture()
        .set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
    fog_of_war
        .texture()
        .set_texture_wrap(&thread, TextureWrap::TEXTURE_WRAP_CLAMP);

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Move player around
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            player_position.x += 5.0;
        }
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            player_position.x -= 5.0;
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            player_position.y += 5.0;
        }
        if rl.is_key_down(KeyboardKey::KEY_UP) {
            player_position.y -= 5.0;
        }

        // Check player position to avoid moving outside tilemap limits
        if player_position.x < 0.0 {
            player_position.x = 0.0;
        } else if (player_position.x + PLAYER_SIZE as f32)
            > (map.tiles_x as f32 * MAP_TILE_SIZE as f32)
        {
            player_position.x = map.tiles_x as f32 * MAP_TILE_SIZE as f32 - PLAYER_SIZE as f32;
        }
        if player_position.y < 0.0 {
            player_position.y = 0.0;
        } else if (player_position.y + PLAYER_SIZE as f32)
            > (map.tiles_y as f32 * MAP_TILE_SIZE as f32)
        {
            player_position.y = map.tiles_y as f32 * MAP_TILE_SIZE as f32 - PLAYER_SIZE as f32;
        }

        // Previous visited tiles are set to partial fog
        for i in 0..(map.tiles_x * map.tiles_y) {
            if map.tile_fog[i as usize] == 1 {
                map.tile_fog[i as usize] = 2;
            }
        }

        // Get current tile position from player pixel position
        player_tile_x =
            ((player_position.x + MAP_TILE_SIZE as f32 / 2.0) / MAP_TILE_SIZE as f32) as i32;
        player_tile_y =
            ((player_position.y + MAP_TILE_SIZE as f32 / 2.0) / MAP_TILE_SIZE as f32) as i32;

        // Check visibility and update fog
        // NOTE: We check tilemap limits to avoid processing tiles out-of-array-bounds (it could crash program)
        for y in (player_tile_y - PLAYER_TILE_VISIBILITY)..(player_tile_y + PLAYER_TILE_VISIBILITY)
        {
            for x in
                (player_tile_x - PLAYER_TILE_VISIBILITY)..(player_tile_x + PLAYER_TILE_VISIBILITY)
            {
                if (x >= 0) && (x < map.tiles_x as i32) && (y >= 0) && (y < map.tiles_y as i32) {
                    map.tile_fog[(y * map.tiles_x as i32 + x) as usize] = 1;
                }
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        // Draw fog of war to a small render texture for automatic smoothing on scaling
        let mut d = rl.begin_drawing(&thread);
        {
            let mut tm = d.begin_texture_mode(&thread, &mut fog_of_war);
            tm.clear_background(Color::BLANK);
            for y in 0..map.tiles_y {
                for x in 0..map.tiles_x {
                    if map.tile_fog[(y * map.tiles_x + x) as usize] == 0 {
                        tm.draw_rectangle(x as i32, y as i32, 1, 1, Color::BLACK);
                    } else if map.tile_fog[(y * map.tiles_x + x) as usize] == 2 {
                        tm.draw_rectangle(x as i32, y as i32, 1, 1, Color::BLACK.alpha(0.8));
                    }
                }
            }
        }

        d.clear_background(Color::RAYWHITE);

        for y in 0..map.tiles_y {
            for x in 0..map.tiles_x {
                // Draw tiles from id (and tile borders)
                d.draw_rectangle(
                    x as i32 * MAP_TILE_SIZE,
                    y as i32 * MAP_TILE_SIZE,
                    MAP_TILE_SIZE,
                    MAP_TILE_SIZE,
                    if map.tile_ids[(y * map.tiles_x + x) as usize] == 0 {
                        Color::BLUE
                    } else {
                        Color::BLUE.alpha(0.9)
                    },
                );
                d.draw_rectangle_lines(
                    x as i32 * MAP_TILE_SIZE,
                    y as i32 * MAP_TILE_SIZE,
                    MAP_TILE_SIZE,
                    MAP_TILE_SIZE,
                    Color::DARKBLUE.alpha(0.5),
                );
            }
        }

        // Draw player
        d.draw_rectangle_v(
            player_position,
            Vector2::new(PLAYER_SIZE as f32, PLAYER_SIZE as f32),
            Color::RED,
        );

        // Draw fog of war (scaled to full map, bilinear filtering)
        d.draw_texture_pro(
            fog_of_war.texture(),
            Rectangle::new(
                0.0,
                0.0,
                fog_of_war.texture().width() as f32,
                -(fog_of_war.texture().height() as f32),
            ),
            Rectangle::new(
                0.0,
                0.0,
                map.tiles_x as f32 * MAP_TILE_SIZE as f32,
                map.tiles_y as f32 * MAP_TILE_SIZE as f32,
            ),
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );

        // Draw player current tile
        d.draw_text(
            &format!("Current tile: [{player_tile_x},{player_tile_y}]"),
            10,
            10,
            20,
            Color::RAYWHITE,
        );
        d.draw_text(
            "ARROW KEYS to move",
            10,
            screen_height - 25,
            20,
            Color::RAYWHITE,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // map.tile_ids and map.tile_fog are dropped automatically (Rust ownership)
    // UnloadRenderTexture is handled by RAII drop of `fog_of_war`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
