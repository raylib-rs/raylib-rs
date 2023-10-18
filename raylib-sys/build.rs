/* raylib-sys
   build.rs - Cargo build script

Copyright (c) 2018-2019 Paul Clement (@deltaphc)

This software is provided "as-is", without any express or implied warranty. In no event will the authors be held liable for any damages arising from the use of this software.

Permission is granted to anyone to use this software for any purpose, including commercial applications, and to alter it and redistribute it freely, subject to the following restrictions:

  1. The origin of this software must not be misrepresented; you must not claim that you wrote the original software. If you use this software in a product, an acknowledgment in the product documentation would be appreciated but is not required.

  2. Altered source versions must be plainly marked as such, and must not be misrepresented as being the original software.

  3. This notice may not be removed or altered from any source distribution.
*/
#![allow(dead_code)]

extern crate bindgen;

use std::env;
use std::path::{Path, PathBuf};

use bindgen::callbacks::{DeriveTrait, ImplementsTrait, ParseCallbacks};

#[derive(Debug)]
struct TypeOverrideCallback;

impl ParseCallbacks for TypeOverrideCallback {
    fn blocklisted_type_implements_trait(
        &self,
        name: &str,
        derive_trait: DeriveTrait,
    ) -> Option<ImplementsTrait> {
        const OK_TRAITS: [DeriveTrait; 3] = [
            DeriveTrait::Copy,
            DeriveTrait::Debug,
            DeriveTrait::PartialEqOrPartialOrd,
        ];
        const OVERRIDEN_TYPES: [&str; 5] =
            ["Vector2", "Vector3", "Vector4", "Matrix", "Quaternion"];

        (OK_TRAITS.contains(&derive_trait) && OVERRIDEN_TYPES.contains(&name))
            .then_some(ImplementsTrait::Yes)
    }
}

#[cfg(feature = "nobuild")]
fn build_with_cmake(_src_path: &str) {}

#[cfg(not(feature = "nobuild"))]
fn build_with_cmake(src_path: &str) {
    // CMake uses different lib directories on different systems.
    // I do not know how CMake determines what directory to use,
    // so we will check a few possibilities and use whichever is present.
    fn join_cmake_lib_directory(path: PathBuf) -> PathBuf {
        let possible_cmake_lib_directories = ["lib", "lib64", "lib32"];
        for lib_directory in &possible_cmake_lib_directories {
            let lib_path = path.join(lib_directory);
            if lib_path.exists() {
                return lib_path;
            }
        }
        path
    }

    let target = env::var("TARGET").expect("Cargo build scripts always have TARGET");
    let (platform, platform_os) = platform_from_target(&target);

    let mut conf = cmake::Config::new(src_path);
    let mut builder;
    #[cfg(debug_assertions)]
    {
        builder = conf.profile("Debug");
        builder = builder.define("CMAKE_BUILD_TYPE", "Debug")
    }

    #[cfg(not(debug_assertions))]
    {
        builder = conf.profile("Release");
        builder = builder.define("CMAKE_BUILD_TYPE", "Release")
    }

    builder
        .generator("Ninja")
        .define("BUILD_EXAMPLES", "OFF")
        .define("CMAKE_BUILD_TYPE", "Release")
        // turn off until this is fixed
        .define("SUPPORT_BUSY_WAIT_LOOP", "OFF");

    // Enable wayland cmake flag if feature is specified
    #[cfg(feature = "wayland")]
    {
        builder.define("USE_WAYLAND", "ON");
        builder.define("USE_EXTERNAL_GLFW", "ON"); // Necessary for wayland support in my testing
    }

    // This seems redundant, but I felt it was needed incase raylib changes it's default
    #[cfg(not(feature = "wayland"))]
    builder.define("USE_WAYLAND", "OFF");

    // Scope implementing flags for forcing OpenGL version
    // See all possible flags at https://github.com/raysan5/raylib/wiki/CMake-Build-Options
    {
        #[cfg(feature = "opengl_33")]
        builder.define("OPENGL_VERSION", "3.3");

        #[cfg(feature = "opengl_21")]
        builder.define("OPENGL_VERSION", "2.1");

        #[cfg(feature = "opengl_11")]
        builder.define("OPENGL_VERSION", "1.1");

        #[cfg(feature = "opengl_es_20")]
        builder.define("OPENGL_VERSION", "ES 2.0");

        #[cfg(feature = "opengl_es_30")]
        builder.define("OPENGL_VERSION", "ES 3.0");

        // Once again felt this was necessary incase a default was changed :)
        #[cfg(not(any(
            feature = "opengl_33",
            feature = "opengl_21",
            feature = "opengl_11",
            feature = "opengl_es_20",
            feature = "opengl_es_30",
        )))]
        builder.define("OPENGL_VERSION", "OFF");
    }

    match platform {
        Platform::Desktop => conf.define("PLATFORM", "Desktop"),
        Platform::Web => conf
            .define("PLATFORM", "Web")
            .define("CMAKE_C_FLAGS", "-s ASYNCIFY"),
        Platform::DRM => conf.define("PLATFORM", "DRM"),
    };

    let dst = conf.build();
    let dst_lib = join_cmake_lib_directory(dst);
    // on windows copy the static library to the proper file name
    if platform_os == PlatformOS::Windows {
        if Path::new(&dst_lib.join("raylib.lib")).exists() {
            // DO NOTHING
        } else if Path::new(&dst_lib.join("raylib_static.lib")).exists() {
            std::fs::copy(
                dst_lib.join("raylib_static.lib"),
                dst_lib.join("raylib.lib"),
            )
            .expect("failed to create windows library");
        } else if Path::new(&dst_lib.join("libraylib_static.a")).exists() {
            std::fs::copy(
                dst_lib.join("libraylib_static.a"),
                dst_lib.join("libraylib.a"),
            )
            .expect("failed to create windows library");
        } else if Path::new(&dst_lib.join("libraylib.a")).exists() {
            // DO NOTHING
        } else {
            panic!("failed to create windows library");
        }
    } // on web copy libraylib.bc to libraylib.a
    if platform == Platform::Web && !Path::new(&dst_lib.join("libraylib.a")).exists() {
        std::fs::copy(dst_lib.join("libraylib.bc"), dst_lib.join("libraylib.a"))
            .expect("failed to create wasm library");
    }
    // println!("cmake build {}", c.display());
    println!("cargo:rustc-link-search=native={}", dst_lib.display());
}

fn gen_bindings() {
    let target = env::var("TARGET").expect("Cargo build scripts always have TARGET");
    let (platform, os) = platform_from_target(&target);

    let plat = match platform {
        Platform::Desktop => "-DPLATFORM_DESKTOP",
        Platform::DRM => "-DPLATFORM_DRM",
        Platform::Web => "-DPLATFORM_WEB",
    };

    let mut builder = bindgen::Builder::default()
        .header("binding/binding.h")
        .rustified_enum(".+")
        .blocklist_type("Vector2")
        .blocklist_type("Vector3")
        .blocklist_type("Vector4")
        .blocklist_type("Matrix")
        .blocklist_type("Quaternion")
        .parse_callbacks(Box::new(TypeOverrideCallback))
        .clang_arg("-I../raylib/src")
        .clang_arg("-std=c99")
        .clang_arg(plat);
    //.parse_callbacks(Box::new(bindgen::CargoCallbacks));

    if platform == Platform::Desktop && os == PlatformOS::Windows {
        // odd workaround for booleans being broken
        builder = builder.clang_arg("-D__STDC__");
    }

    if platform == Platform::Web {
        builder = builder
            .clang_arg("-fvisibility=default")
            .clang_arg("--target=wasm32-emscripten");
    }

    // Build
    let bindings = builder.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn gen_rgui() {
    // Compile the code and link with cc crate
    cc::Build::new()
        .file("binding/wrapper.c")
        .include("binding")
        .warnings(false)
        // .flag("-std=c99")
        .extra_warnings(false)
        .compile("rgui");
}

#[cfg(feature = "nobuild")]
fn link(_platform: Platform, _platform_os: PlatformOS) {}

#[cfg(not(feature = "nobuild"))]
fn link(platform: Platform, platform_os: PlatformOS) {
    match platform_os {
        PlatformOS::Windows => {
            println!("cargo:rustc-link-lib=dylib=winmm");
            println!("cargo:rustc-link-lib=dylib=gdi32");
            println!("cargo:rustc-link-lib=dylib=user32");
            println!("cargo:rustc-link-lib=dylib=shell32");

            // Good old opengl32.dll
            #[cfg(feature = "opengl_11")]
            println!("cargo:rustc-link-lib=opengl32");
        }
        PlatformOS::Linux if platform != Platform::DRM => {
            // X11 linking
            #[cfg(not(feature = "wayland"))]
            {
                println!("cargo:rustc-link-search=/usr/local/lib");
                println!("cargo:rustc-link-lib=X11");
            }

            // Wayland linking
            #[cfg(feature = "wayland")]
            {
                println!("cargo:rustc-link-search=/usr/local/lib");
                println!("cargo:rustc-link-lib=wayland-client");
                println!("cargo:rustc-link-lib=glfw"); // Link against locally installed glfw
            }

            // OpenGL 1.1 library
            #[cfg(feature = "opengl_11")]
            println!("cargo:rustc-link-lib=GL");
        }
        PlatformOS::OSX => {
            println!("cargo:rustc-link-search=native=/usr/local/lib");
            println!("cargo:rustc-link-lib=framework=OpenGL");
            println!("cargo:rustc-link-lib=framework=Cocoa");
            println!("cargo:rustc-link-lib=framework=IOKit");
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=CoreVideo");
        }
        _ => (),
    }
    if platform == Platform::Web {
        println!("cargo:rustc-link-lib=glfw");
    } else if platform == Platform::DRM {
        println!("cargo:rustc-link-lib=GLESv2");
        println!("cargo:rustc-link-lib=EGL");
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:rustc-link-lib=rt");
        println!("cargo:rustc-link-lib=gbm");
        println!("cargo:rustc-link-lib=drm");
        println!("cargo:rustc-link-lib=atomic");
        println!("cargo:rustc-link-lib=dl");
    }

    println!("cargo:rustc-link-lib=static=raylib");
}

fn main() {
    let target = env::var("TARGET").expect("Cargo build scripts always have TARGET");
    let (platform, platform_os) = platform_from_target(&target);

    // Donwload raylib source
    let src = cp_raylib();
    build_with_cmake(&src);

    gen_bindings();

    link(platform, platform_os);

    gen_rgui();
}

// cp_raylib copy raylib to an out dir
fn cp_raylib() -> String {
    let out = env::var("OUT_DIR").unwrap();
    let out = Path::new(&out); //.join("raylib_source");

    let mut options = fs_extra::dir::CopyOptions::new();
    options.skip_exist = true;
    fs_extra::dir::copy("raylib", out, &options)
        .unwrap_or_else(|_| panic!("failed to copy raylib source to {}", out.to_string_lossy()));

    out.join("raylib").to_string_lossy().to_string()
}

fn cp_raygui() -> String {
    let out = env::var("OUT_DIR").unwrap();
    let out = Path::new(&out); //.join("raylib_source");

    let mut options = fs_extra::dir::CopyOptions::new();
    options.skip_exist = true;
    fs_extra::dir::copy("raygui", out, &options)
        .unwrap_or_else(|_| panic!("failed to copy raygui source to {}", out.to_string_lossy()));

    out.join("raygui").to_string_lossy().to_string()
}

// run_command runs a command to completion or panics. Used for running curl and powershell.
fn run_command(cmd: &str, args: &[&str]) {
    use std::process::Command;
    match Command::new(cmd).args(args).output() {
        Ok(output) => {
            if !output.status.success() {
                let error = std::str::from_utf8(&output.stderr).unwrap();
                panic!("Command '{}' failed: {}", cmd, error);
            }
        }
        Err(error) => {
            panic!("Error running command '{}': {:#}", cmd, error);
        }
    }
}

fn platform_from_target(target: &str) -> (Platform, PlatformOS) {
    let platform = if target.contains("wasm32") {
        // make sure cmake knows that it should bundle glfw in
        // Cargo web takes care of this but better safe than sorry
        Platform::Web
    } else if cfg!(feature = "drm") {
        Platform::DRM
    } else {
        Platform::Desktop
    };

    let platform_os = if platform == Platform::Desktop {
        // Determine PLATFORM_OS in case PLATFORM_DESKTOP selected
        if env::var("OS")
            .unwrap_or("".to_owned())
            .contains("Windows_NT")
            || env::var("TARGET")
                .unwrap_or("".to_owned())
                .contains("windows")
        {
            // No uname.exe on MinGW!, but OS=Windows_NT on Windows!
            // ifeq ($(UNAME),Msys) -> Windows
            PlatformOS::Windows
        } else {
            let un: &str = &uname();
            match un {
                "Linux" => PlatformOS::Linux,
                "FreeBSD" => PlatformOS::BSD,
                "OpenBSD" => PlatformOS::BSD,
                "NetBSD" => PlatformOS::BSD,
                "DragonFly" => PlatformOS::BSD,
                "Darwin" => PlatformOS::OSX,
                _ => panic!("Unknown platform {}", uname()),
            }
        }
    } else if platform == Platform::DRM {
        PlatformOS::Linux
    } else {
        PlatformOS::Unknown
    };

    (platform, platform_os)
}

fn uname() -> String {
    use std::process::Command;
    String::from_utf8_lossy(
        &Command::new("uname")
            .output()
            .expect("failed to run uname")
            .stdout,
    )
    .trim()
    .to_owned()
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Platform {
    Web,
    Desktop,
    DRM, // KMS and raspberry pi
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlatformOS {
    Windows,
    Linux,
    BSD,
    OSX,
    Unknown,
}

#[derive(Debug, PartialEq)]
enum LibType {
    Static,
    _Shared,
}

#[derive(Debug, PartialEq)]
enum BuildMode {
    Release,
    Debug,
}

struct BuildSettings {
    pub platform: Platform,
    pub platform_os: PlatformOS,
    pub bundled_glfw: bool,
}
