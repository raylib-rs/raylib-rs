/*******************************************************************************************
*
*   raylib [core] example - directory files
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.6
*
*   Example contributed by Hugo ARNAL (@hugoarnal) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Hugo ARNAL (@hugoarnal)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;
use std::ffi::CString;
use std::path::PathBuf;

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
        .title("raylib [core] example - directory files")
        .build();

    // idiomatic: std::env::current_dir() replaces GetWorkingDirectory().
    let mut directory: PathBuf = std::env::current_dir().unwrap();

    // Load file-paths on current working directory
    // NOTE: LoadDirectoryFiles() loads files and directories by default,
    // use LoadDirectoryFilesEx() for custom filters and recursive directories loading
    //let mut files = rl.load_directory_files(&directory);
    let mut files = rl.load_directory_files_ex(directory.as_os_str(), ".png;.c".to_string(), false);

    let mut btn_back_pressed = false;

    let mut list_scroll_index: i32 = 0;
    let mut list_item_active: i32 = -1;
    let mut list_item_focused: i32 = -1;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if btn_back_pressed {
            // idiomatic: PathBuf::pop() replaces GetPrevDirectoryPath().
            directory.pop();
            drop(files);
            files = rl.load_directory_files_ex(directory.as_os_str(), ".png;.c".to_string(), false);
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let dir_string = directory.to_string_lossy().into_owned();
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        btn_back_pressed = d.gui_button(Rectangle::new(40.0, 10.0, 48.0, 28.0), "<");

        let base_size = d.gui_get_font().base_size();
        d.gui_set_style(
            GuiControl::DEFAULT,
            GuiDefaultProperty::TEXT_SIZE,
            base_size * 2,
        );
        d.gui_label(
            Rectangle::new(40.0 + 48.0 + 10.0, 10.0, 700.0, 28.0),
            &dir_string,
        );
        d.gui_set_style(
            GuiControl::DEFAULT,
            GuiDefaultProperty::TEXT_SIZE,
            base_size,
        );

        d.gui_set_style(
            GuiControl::LISTVIEW,
            GuiControlProperty::TEXT_ALIGNMENT,
            GuiTextAlignment::TEXT_ALIGN_LEFT as i32,
        );
        d.gui_set_style(GuiControl::LISTVIEW, GuiControlProperty::TEXT_PADDING, 40);

        // SAFETY: pass through the FFI-level GuiListViewEx because the safe wrapper accepts &[&str],
        // and our files come from raylib-allocated `paths`. Build a temporary &[*const c_char] vector.
        let paths: Vec<CString> = files
            .iter()
            .map(|p| CString::new(p).unwrap_or_default())
            .collect();
        let mut path_ptrs: Vec<*const std::os::raw::c_char> =
            paths.iter().map(|c| c.as_ptr()).collect();
        unsafe {
            raylib::ffi::GuiListViewEx(
                Rectangle::new(0.0, 50.0, screen_w as f32, screen_h as f32 - 50.0),
                path_ptrs.as_mut_ptr() as *mut *const std::os::raw::c_char,
                files.count() as i32,
                &mut list_scroll_index,
                &mut list_item_active,
                &mut list_item_focused,
            );
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadDirectoryFiles is handled by RAII drop of `files`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
