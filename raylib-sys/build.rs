/* raylib-sys
   build.rs - Cargo build script

Copyright (c) 2018-2019 Paul Clement (@deltaphc)

This software is provided "as-is", without any express or implied warranty. In no event will the authors be held liable for any damages arising from the use of this software.

Permission is granted to anyone to use this software for any purpose, including commercial applications, and to alter it and redistribute it freely, subject to the following restrictions:

  1. The origin of this software must not be misrepresented; you must not claim that you wrote the original software. If you use this software in a product, an acknowledgment in the product documentation would be appreciated but is not required.

  2. Altered source versions must be plainly marked as such, and must not be misrepresented as being the original software.

  3. This notice may not be removed or altered from any source distribution.
*/
#![allow(dead_code, unused_imports)]

extern crate bindgen;

use bindgen::callbacks::DeriveTrait;
use bindgen::callbacks::ImplementsTrait;
use bindgen::callbacks::ParseCallbacks;
use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};

use cmake::Config;

// What version of raylib is this on? You can check:
// the github submodule
// the README or raylib-sys/raylib
// look at the tracelog when you run your game

#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);

impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        if self.0.contains(name) {
            bindgen::callbacks::MacroParsingBehavior::Ignore
        } else {
            bindgen::callbacks::MacroParsingBehavior::Default
        }
    }
}

#[derive(Debug)]
struct SerdeOnMath;

impl bindgen::callbacks::ParseCallbacks for SerdeOnMath {
    fn add_derives(&self, info: &bindgen::callbacks::DeriveInfo) -> Vec<String> {
        match info.name {
            "Vector2" | "Vector3" | "Vector4" | "Matrix" => vec![
                "serde::Serialize".to_string(),
                "serde::Deserialize".to_string(),
            ],
            _ => vec![],
        }
    }
}

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
        let overridden_types = ["Quaternion", "Rectangle", "Color"];

        (OK_TRAITS.contains(&derive_trait) && overridden_types.contains(&name))
            .then_some(ImplementsTrait::Yes)
    }
}

#[cfg(all(
    feature = "software_renderer",
    any(
        feature = "opengl_11",
        feature = "opengl_21",
        feature = "opengl_33",
        feature = "opengl_43",
        feature = "opengl_es_20",
        feature = "opengl_es_30",
        feature = "drm",
    )
))]
compile_error!(
    "feature `software_renderer` (PLATFORM=Memory) is mutually exclusive with the \
     opengl_* backends and `drm`; enable it with default-features = false."
);

#[cfg(feature = "nobuild")]
fn build_with_cmake(_src_path: &str) {}

#[cfg(not(feature = "nobuild"))]
fn build_with_cmake(src_path: &str) {
    // CMake uses different lib directories on different systems.
    // I do not know how CMake determines what directory to use,
    // so we will check a few possibilities and use whichever is present.
    if is_directory_empty(src_path) {
        panic!("raylib source does not exist in: `raylib-sys/raylib`. Please copy it in");
    }
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

    let profile;
    #[cfg(debug_assertions)]
    {
        // note: For some extra perf in debug mode, you can change this to Release by default since we're building a static lib ONCE, while your game is in debug, so you get the best of ~both worlds(high perf drawing, debugger info for the rust code)
        profile = "Debug";
    }
    #[cfg(not(debug_assertions))]
    {
        profile = "Release";
    }
    #[cfg(feature = "release_with_debug_info")]
    {
        profile = "RelWithDebInfo"
    }
    #[cfg(feature = "min_size_rel")]
    {
        profile = "MinSizeRel"
    }
    let builder = conf.profile(profile);
    builder.define("CMAKE_BUILD_TYPE", profile);
    // NOTE(WINDOWS ONLY, linux is fine): Custom builds that turn off modules might not link because the linker tries to find the function def which windowss sometimes does not optimize out
    // The correct/safe way would be to edit the raylib-rs bindings and feature flag remove calls to things you are opting out of(preferably making them stub and do nothing but still exist) but to atm to fix this dangerously(e.g UB when you call LoadSound() while SUPPORT_MODULE_RAUDIO OFF), create a `build.rs` script for in mygameproject and add the following:
    //`println!("cargo:rustc-link-arg=/FORCE:UNRESOLVED");`
    builder
        .define("BUILD_EXAMPLES", "OFF")
        .define("CUSTOMIZE_BUILD", "ON"); // MUST BE ON or else it'll ignore all other flags
    features_from_env(builder);

    // Scope implementing flags for forcing OpenGL version
    // See all possible flags at https://github.com/raysan5/raylib/wiki/CMake-Build-Options
    {
        #[cfg(feature = "opengl_43")]
        builder.define("OPENGL_VERSION", "4.3");

        #[cfg(feature = "opengl_33")]
        builder.define("OPENGL_VERSION", "3.3");

        #[cfg(feature = "opengl_21")]
        builder.define("OPENGL_VERSION", "2.1");

        #[cfg(feature = "opengl_11")]
        builder.define("OPENGL_VERSION", "1.1");

        #[cfg(feature = "opengl_es_20")]
        {
            builder.define("OPENGL_VERSION", "ES 2.0");
            println!("cargo:rustc-link-lib=GLESv2");
            println!("cargo:rustc-link-lib=GLdispatch");
        }

        #[cfg(feature = "opengl_es_30")]
        {
            builder.define("OPENGL_VERSION", "ES 3.0");
            println!("cargo:rustc-link-lib=GLESv2");
            println!("cargo:rustc-link-lib=GLdispatch");
        }
    }

    match platform {
        Platform::Desktop => {
            #[cfg(feature = "sdl")]
            {
                if pkg_config::Config::new().probe("sdl3").is_ok() {
                    println!("cargo:rustc-link-lib=SDL3");
                } else {
                    println!("cargo:rustc-link-lib=SDL2");
                }
                conf.define("PLATFORM", "SDL")
            }

            #[cfg(not(feature = "sdl"))]
            {
                conf.define("PLATFORM", "Desktop")
            }
        }
        Platform::Memory => conf.define("PLATFORM", "Memory"),
        Platform::Web => conf.define("PLATFORM", "Web"),
        Platform::Drm => conf.define("PLATFORM", "DRM"),
        Platform::Rpi => conf.define("PLATFORM", "Raspberry Pi"),
        Platform::Android => {
            // get required env variables
            let android_ndk_home = env::var("ANDROID_NDK_HOME")
                .expect("Please set the environment variable: ANDROID_NDK_HOME:(e.g /home/u/Android/Sdk/ndk/VXXX/)");
            let android_platform = target.split("-").last().expect("fail to parse the android version of the target triple, example:'aarch64-linux-android25'");
            let abi_version = android_platform
                .split("-")
                .last()
                .expect("Could not get abi version. Is ANDROID_PLATFORM valid?");
            let toolchain_file =
                format!("{}/build/cmake/android.toolchain.cmake", &android_ndk_home);
            // Detect ANDROID_ABI using the target triple
            let android_arch_abi = match target.as_str() {
                "aarch64-linux-android" => "arm64-v8a",
                "armv7-linux-androideabi" => "armeabi-v7a",
                "x86_64-linux-android" => "x86_64",
                "i686-linux-android" => "x86",
                _ => panic!("Unsupported target triple for Android"),
            };
            // we'll set as many variables as possible according to:
            // https://developer.android.com/ndk/guides/cmake#command-line_1
            // https://cmake.org/cmake/help/v3.31/manual/cmake-toolchains.7.html#cross-compiling-for-android-with-the-ndk
            // how to build:
            // 0) set the correct linker in your game project's Cargo.toml (note: platform number should match):
            // [target.aarch64-linux-android]
            // linker = "/home/u/Android/Sdk/ndk/28.0.12433566/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android<PLATFORM_NUMBER>-clang"

            // 1) set env variable ANDROID_NDK_HOME
            // 2) compile with: `cargo ndk -t <ARCH> -p <PLATFORM_NUMBER> -o ./jniLibs build`
            // example(cargo ndk -t arm64-v8a -p 25 -o ./jniLibs build)

            conf.define("CMAKE_SYSTEM_NAME", "Android")
                .define("PLATFORM", "Android")
                .define("CMAKE_SYSTEM_VERSION", abi_version)
                .define("ANDROID_ABI", android_arch_abi)
                .define("CMAKE_ANDROID_ARCH_ABI", android_arch_abi)
                .define("CMAKE_ANDROID_NDK", &android_ndk_home)
                .define("ANDROID_PLATFORM", android_platform)
                .define("CMAKE_TOOLCHAIN_FILE", &toolchain_file)
        }
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
    } else if platform == Platform::Web && !Path::new(&dst_lib.join("libraylib.a")).exists() {
        std::fs::copy(dst_lib.join("libraylib.bc"), dst_lib.join("libraylib.a"))
            .expect("failed to create wasm library");
    }
    // println!("cmake build {}", c.display());
    println!("cargo:rustc-link-search=native={}", dst_lib.display());
    if platform == Platform::Android {
        println!("cargo:rustc-link-lib=log");
        println!("cargo:rustc-link-lib=android");
        println!("cargo:rustc-link-lib=EGL");
        println!("cargo:rustc-link-lib=GLESv2");
        println!("cargo:rustc-link-lib=OpenSLES");
        println!("cargo:rustc-link-lib=c");
        println!("cargo:rustc-link-lib=m");
    }
}

#[cfg(not(feature = "nobindgen"))]
fn gen_bindings() {
    let target = env::var("TARGET").expect("Cargo build scripts always have TARGET");
    let (platform, os) = platform_from_target(&target);

    let plat = match platform {
        Platform::Desktop => "-DPLATFORM_DESKTOP",
        Platform::Memory => "-DPLATFORM_MEMORY",
        Platform::Drm => "-DPLATFORM_DRM",
        Platform::Rpi => "-DPLATFORM_RPI",
        Platform::Android => "-DPLATFORM_ANDROID",
        Platform::Web => "-DPLATFORM_WEB",
    };

    let ignored_macros = IgnoreMacros(
        vec![
            "FP_INFINITE".into(),
            "FP_NAN".into(),
            "FP_NORMAL".into(),
            "FP_SUBNORMAL".into(),
            "FP_ZERO".into(),
            "IPPORT_RESERVED".into(),
        ]
        .into_iter()
        .collect(),
    );

    let header;
    #[cfg(feature = "nobuild")]
    {
        header = "binding/nobuild.h"
    }
    #[cfg(not(feature = "nobuild"))]
    {
        header = "binding/binding.h"
    }
    let mut builder = bindgen::Builder::default()
        .header(header)
        .use_core()
        .rustified_enum(".+")
        .derive_partialeq(true)
        .derive_default(true)
        .blocklist_type("Quaternion")
        .blocklist_type("Rectangle")
        .blocklist_type("Color")
        .opaque_type("__mingw_ldbl_type_t")
        .parse_callbacks(Box::new(TypeOverrideCallback))
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Vendored raylib headers, relative to the build-script CWD (the package
        // root). nobuild.h resolves raylib.h/raymath.h/rlgl.h through this -I;
        // binding.h's quoted ../raylib/src/ includes resolve file-relative and
        // don't need it.
        .clang_arg("-Iraylib/src")
        .clang_arg("-std=c99")
        .clang_arg(plat)
        // RAYMATH_IMPLEMENTATION makes RMAPI expand to `extern inline` so
        // bindgen sees the raymath functions as non-static external declarations
        // and generates Rust FFI stubs for them.  The matching external symbols
        // are provided by the raymath_shim static library (gen_raymath).
        .clang_arg("-DRAYMATH_IMPLEMENTATION")
        // Emit Rust `extern "C"` declarations for raymath's inline functions.
        // External symbols are provided by the raymath_shim static library.
        .generate_inline_functions(true)
        .parse_callbacks(Box::new(ignored_macros));

    if platform == Platform::Desktop && os == PlatformOS::Windows {
        // odd workaround for booleans being broken
        builder = builder.clang_arg("-D__STDC__");
    }

    // MinGW cross builds: without an explicit `--target`, bindgen's libclang
    // defaults to the *host* triple and mis-parses the MinGW system headers
    // (e.g. `__mingw_ldbl_type_t`). Pin clang to the build target so it uses the
    // right headers/ABI. Scoped to `*-windows-gnu` so host/MSVC/other targets
    // are unaffected. Resolves #324; original workaround by @jgabaut (#325).
    if target.ends_with("-windows-gnu") {
        builder = builder.clang_arg(format!("--target={target}"));
    }

    // nobuild bindings may be consumed cross-target via nobindgen +
    // RAYLIB_BINDGEN_LOCATION (e.g. the no-std CI leg compile-checks a
    // host-generated binding for thumbv7em). bindgen's layout asserts
    // hardcode the generating host's pointer width and break such
    // compile-only cross-checks; hosted default builds keep them.
    #[cfg(feature = "nobuild")]
    {
        builder = builder.layout_tests(false);
    }

    if platform == Platform::Web {
        builder = builder
            .clang_arg("-fvisibility=default")
            .clang_arg("--target=wasm32-emscripten");
    }

    if std::env::var("CARGO_FEATURE_SERDE").is_ok() {
        builder = builder.parse_callbacks(Box::new(SerdeOnMath));
    }

    // Build
    let bindings = builder.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Export the raylib include path so dependent crates (e.g. other raylib C-utility sys-crates)
    // can compose against the same headers without version drift.  Accessible as
    // the `DEP_RAYLIB_INCLUDE` environment variable in the downstream build script.
    println!("cargo::metadata=include={}/include", out_path.display());
}

fn gen_rgui() {
    // Compile the code and link with cc crate.
    //
    // Allocator unification (RAYGUI_MALLOC → RL_MALLOC etc.) is done in
    // binding/rgui_wrapper.c itself, not via `.define()` here, so that the
    // RL_* macros from raylib.h are in scope before the RAYGUI_* defines
    // take effect. See the comment block at the top of rgui_wrapper.c.
    cc::Build::new()
        .files(vec!["binding/rgui_wrapper.c"])
        .include("binding")
        .warnings(false)
        .extra_warnings(false)
        .compile("rgui");
}

fn gen_utils() {
    // Compile the code and link with cc crate
    cc::Build::new()
        .files(vec!["binding/utils_log.c"])
        .include("binding")
        .warnings(false)
        .extra_warnings(false)
        .compile("utils_log");
}

fn gen_raymath() {
    cc::Build::new()
        .files(vec!["binding/raymath_shim.c"])
        .include("binding")
        .warnings(false)
        .extra_warnings(false)
        .compile("raymath_shim");
}

#[cfg(feature = "nobuild")]
fn link(_platform: Platform, _platform_os: PlatformOS) {
    println!("cargo:rustc-link-lib=dylib=raylib");
}

#[cfg(not(feature = "nobuild"))]
fn link(platform: Platform, platform_os: PlatformOS) {
    if platform == Platform::Memory {
        if platform_os == PlatformOS::Windows {
            println!("cargo:rustc-link-lib=dylib=winmm");
        }
        println!("cargo:rustc-link-lib=static=raylib");
        return;
    }
    match platform_os {
        PlatformOS::Windows => {
            println!("cargo:rustc-link-lib=dylib=winmm");
            println!("cargo:rustc-link-lib=dylib=gdi32");
            println!("cargo:rustc-link-lib=dylib=user32");
            println!("cargo:rustc-link-lib=dylib=shell32");
        }
        PlatformOS::Linux => {
            // X11 linking
            #[cfg(not(any(feature = "wayland", target_os = "android", feature = "drm")))]
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
        }
        PlatformOS::Osx => {
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
    } else if platform == Platform::Drm {
        println!("cargo:rustc-link-lib=EGL");
        println!("cargo:rustc-link-lib=drm");
        println!("cargo:rustc-link-lib=gbm");
    } else if platform == Platform::Rpi {
        println!("cargo:rustc-link-search=/opt/vc/lib");
        println!("cargo:rustc-link-lib=bcm_host");
        println!("cargo:rustc-link-lib=brcmEGL");
        println!("cargo:rustc-link-lib=brcmGLESv2");
        println!("cargo:rustc-link-lib=vcos");
    }

    println!("cargo:rustc-link-lib=static=raylib");
}

fn main() {
    let header;
    #[cfg(feature = "nobuild")]
    {
        header = "binding/nobuild.h"
    }
    #[cfg(not(feature = "nobuild"))]
    {
        header = "binding/binding.h"
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={header}");
    //for cross compiling on switch arm
    //https://users.rust-lang.org/t/cross-compiling-arm/96456/10
    if std::env::var("CROSS_SYSROOT").is_ok() {
        unsafe {
            std::env::remove_var("CROSS_SYSROOT");
        }
    }
    let target = env::var("TARGET").expect("Cargo build scripts always have TARGET");

    if target.contains("wasm32-unknown-emscripten") {
        if let Err(e) = env::var("EMCC_CFLAGS") {
            if e == std::env::VarError::NotPresent {
                panic!(
                    "\nYou must have to set EMCC_CFLAGS yourself to compile for WASM.\n{}{}\"\n",
                    {
                        #[cfg(target_family = "windows")]
                        {
                            "set EMCC_CFLAGS="
                        }
                        #[cfg(not(target_family = "windows"))]
                        {
                            "export EMCC_CFLAGS="
                        }
                    },
                    "\"-O3 -sUSE_GLFW=3 -sASSERTIONS=1 -sWASM=1 -sASYNCIFY -sGL_ENABLE_GET_PROC_ADDRESS=1\""
                );
            } else {
                panic!("\nError regarding EMCC_CFLAGS: {e:?}\n");
            }
        }
    }

    let (platform, platform_os) = platform_from_target(&target);

    let raylib_src = "./raylib";
    build_with_cmake(raylib_src);

    // `nobindgen` consumers supply a pregenerated binding via RAYLIB_BINDGEN_LOCATION
    // (see src/lib.rs); skip bindgen entirely — it may not be runnable for the
    // build target (e.g. the no-std CI leg cross-checking thumbv7em-none-eabihf).
    #[cfg(not(feature = "nobindgen"))]
    gen_bindings();

    link(platform, platform_os);

    #[cfg(feature = "raygui")]
    {
        gen_rgui();
    }

    #[cfg(not(feature = "nobuild"))]
    {
        // utils_log.c backs setLogCallbackWrapper(), which is itself
        // nobuild-gated in raylib/src/core/callbacks.rs. So TraceLog-callback
        // registration is intentionally unavailable under nobuild; leaving this
        // gated keeps the link surface minimal for prebuilt-link consumers.
        gen_utils();
    }

    // The raymath shim (binding/raymath_shim.c, compiled with
    // RAYMATH_IMPLEMENTATION) provides Vector2Add/Vector3Add/... as real
    // linkable symbols; raylib's own library exports them only as `static
    // inline`. The safe crate's Vector operators and raylib-sys's
    // raymath_wrappers tests call them, so they must exist even under `nobuild`
    // (linking against a prebuilt raylib). Gate on `not(nobindgen)` rather than
    // `not(nobuild)`: `nobindgen` is the codebase's cross-target-compile-only
    // marker (only ever paired with `nobuild` on thumbv7em), where invoking
    // `cc` to cross-compile the shim would need an arm-none-eabi C toolchain the
    // runner lacks. The shim is self-contained (raymath.h -> math.h only;
    // independent of libraylib), so it compiles on any host target.
    #[cfg(not(feature = "nobindgen"))]
    gen_raymath();

    // ENABLE_UBSAN: the C side is instrumented via cmake's CompilerFlags.cmake
    // (-fsanitize=undefined). The Rust link step needs `-lubsan` explicitly because
    // `-Z build-std` passes `-nodefaultlibs` to gcc, suppressing its normal
    // auto-injection of libubsan from `-fsanitize=undefined`. Emitting
    // `cargo:rustc-link-lib=ubsan` here puts `-lubsan` in the correct position
    // (after the rlibs that reference __ubsan_handle_*).
    #[cfg(feature = "ENABLE_UBSAN")]
    println!("cargo:rustc-link-lib=ubsan");
}

#[must_use]
/// returns false if the directory does not exist
fn is_directory_empty(path: &str) -> bool {
    match std::fs::read_dir(path) {
        Ok(mut dir) => dir.next().is_none(),
        Err(_) => true,
    }
}

fn platform_from_target(target: &str) -> (Platform, PlatformOS) {
    let platform = if cfg!(feature = "software_renderer") {
        Platform::Memory
    } else if cfg!(feature = "drm") {
        Platform::Drm
    } else if cfg!(feature = "legacy_rpi") {
        Platform::Rpi
    } else if target.contains("wasm") {
        Platform::Web
    } else if target.contains("android") {
        Platform::Android
    } else {
        Platform::Desktop
    };

    let platform_os = if matches!(platform, Platform::Desktop | Platform::Memory) {
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
                "FreeBSD" => PlatformOS::Bsd,
                "OpenBSD" => PlatformOS::Bsd,
                "NetBSD" => PlatformOS::Bsd,
                "DragonFly" => PlatformOS::Bsd,
                "Darwin" => PlatformOS::Osx,
                _ => panic!("Unknown platform {}", uname()),
            }
        }
    } else if matches!(platform, Platform::Drm | Platform::Rpi | Platform::Android) {
        let un: &str = &uname();
        if un == "Linux" {
            PlatformOS::Linux
        } else {
            PlatformOS::Unknown
        }
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
    Android,
    Drm,
    Rpi,    // legacy raspberry pi
    Memory, // raylib 6.0 windowless software-render platform (PLATFORM=Memory)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlatformOS {
    Windows,
    Linux,
    Bsd,
    Osx,
    Unknown,
}

/// Copied from https://github.com/raysan5/raylib/wiki/CMake-Build-Options and https://github.com/raysan5/raylib/blob/master/src/config.h
/// You should be copy pasting into both raylib/raylib-sys `Cargo.toml` and here while keeping it as close as possible to raylibs `config.h`` for easy maintenance
#[rustfmt::skip]
fn features_from_env(cmake: &mut Config) {
    let is_android = cfg!(target_os = "android"); // skip linking to x11 & wayland
    cmake.define("ENABLE_ASAN", bstr(cfg!(feature = "ENABLE_ASAN")));
    cmake.define("ENABLE_UBSAN", bstr(cfg!(feature = "ENABLE_UBSAN")));
    cmake.define("ENABLE_MSAN", bstr(cfg!(feature = "ENABLE_MSAN")));
    cmake.define("WITH_PIC", bstr(cfg!(feature = "WITH_PIC")));
    cmake.define("BUILD_SHARED_LIBS", bstr(cfg!(feature = "BUILD_SHARED_LIBS")));
    cmake.define("USE_EXTERNAL_GLFW", bstr(cfg!(feature = "USE_EXTERNAL_GLFW")));
    // PLATFORM=Memory neither builds nor links GLFW, but raylib's top-level CMake still
    // raises a FATAL_ERROR on Linux ("Cannot disable both Wayland and X11") for any
    // non-DRM/non-Web platform when both GLFW backends are off. Force X11 on under
    // software_renderer to satisfy that guard — the GLFW X11 backend is not compiled
    // for the Memory platform.
    let force_x11 = cfg!(feature = "software_renderer");
    cmake.define("GLFW_BUILD_WAYLAND", bstr(cfg!(feature = "GLFW_BUILD_WAYLAND") && !is_android));
    cmake.define(
        "GLFW_BUILD_X11",
        bstr((cfg!(feature = "GLFW_BUILD_X11") || force_x11) && !is_android),
    );
    cmake.define("INCLUDE_EVERYTHING", bstr(cfg!(feature = "INCLUDE_EVERYTHING")));
    cmake.define("USE_AUDIO", bstr(cfg!(feature = "USE_AUDIO")));
    cmake.define("SUPPORT_MODULE_RSHAPES", bstr(cfg!(feature = "SUPPORT_MODULE_RSHAPES")));
    cmake.define("SUPPORT_MODULE_RTEXTURES", bstr(cfg!(feature = "SUPPORT_MODULE_RTEXTURES")));
    cmake.define("SUPPORT_MODULE_RTEXT", bstr(cfg!(feature = "SUPPORT_MODULE_RTEXT")));
    cmake.define("SUPPORT_MODULE_RMODELS", bstr(cfg!(feature = "SUPPORT_MODULE_RMODELS")));
    cmake.define("SUPPORT_MODULE_RAUDIO", bstr(cfg!(feature = "SUPPORT_MODULE_RAUDIO")));
    cmake.define("SUPPORT_BUSY_WAIT_LOOP", bstr(cfg!(feature = "SUPPORT_BUSY_WAIT_LOOP")));
    cmake.define("SUPPORT_CAMERA_SYSTEM", bstr(cfg!(feature = "SUPPORT_CAMERA_SYSTEM")));
    cmake.define("SUPPORT_GESTURES_SYSTEM", bstr(cfg!(feature = "SUPPORT_GESTURES_SYSTEM")));
    cmake.define("SUPPORT_RPRAND_GENERATOR", bstr(cfg!(feature = "SUPPORT_RPRAND_GENERATOR")));
    cmake.define("SUPPORT_MOUSE_GESTURES", bstr(cfg!(feature = "SUPPORT_MOUSE_GESTURES")));
    cmake.define("SUPPORT_SSH_KEYBOARD_RPI", bstr(cfg!(feature = "SUPPORT_SSH_KEYBOARD_RPI")));
    cmake.define("SUPPORT_WINMM_HIGHRES_TIMER", bstr(cfg!(feature = "SUPPORT_WINMM_HIGHRES_TIMER")));
    cmake.define("SUPPORT_PARTIALBUSY_WAIT_LOOP", bstr(cfg!(feature = "SUPPORT_PARTIALBUSY_WAIT_LOOP")));
    cmake.define("SUPPORT_COMPRESSION_API", bstr(cfg!(feature = "SUPPORT_COMPRESSION_API")));
    cmake.define("SUPPORT_AUTOMATION_EVENTS", bstr(cfg!(feature = "SUPPORT_AUTOMATION_EVENTS")));
    cmake.define("SUPPORT_CUSTOM_FRAME_CONTROL", bstr(cfg!(feature = "SUPPORT_CUSTOM_FRAME_CONTROL")));
    cmake.define("SUPPORT_CLIPBOARD_IMAGE", bstr(cfg!(feature = "SUPPORT_CLIPBOARD_IMAGE")));
    cmake.define("SUPPORT_QUADS_DRAW_MODE", bstr(cfg!(feature = "SUPPORT_QUADS_DRAW_MODE")));
    cmake.define("SUPPORT_FILEFORMAT_PNG", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_PNG")));
    cmake.define("SUPPORT_FILEFORMAT_BMP", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_BMP")));
    cmake.define("SUPPORT_FILEFORMAT_TGA", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_TGA")));
    cmake.define("SUPPORT_FILEFORMAT_JPG", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_JPG")));
    cmake.define("SUPPORT_FILEFORMAT_GIF", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_GIF")));
    cmake.define("SUPPORT_FILEFORMAT_QOI", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_QOI")));
    cmake.define("SUPPORT_FILEFORMAT_PSD", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_PSD")));
    cmake.define("SUPPORT_FILEFORMAT_DDS", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_DDS")));
    cmake.define("SUPPORT_FILEFORMAT_HDR", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_HDR")));
    cmake.define("SUPPORT_FILEFORMAT_PIC", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_PIC")));
    cmake.define("SUPPORT_FILEFORMAT_PNM", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_PNM")));
    cmake.define("SUPPORT_FILEFORMAT_KTX", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_KTX")));
    cmake.define("SUPPORT_FILEFORMAT_ASTC", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_ASTC")));
    cmake.define("SUPPORT_FILEFORMAT_PKM", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_PKM")));
    cmake.define("SUPPORT_FILEFORMAT_PVR", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_PVR")));
    cmake.define("SUPPORT_IMAGE_EXPORT", bstr(cfg!(feature = "SUPPORT_IMAGE_EXPORT")));
    cmake.define("SUPPORT_IMAGE_GENERATION", bstr(cfg!(feature = "SUPPORT_IMAGE_GENERATION")));
    cmake.define("SUPPORT_FILEFORMAT_TTF", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_TTF")));
    cmake.define("SUPPORT_FILEFORMAT_FNT", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_FNT")));
    cmake.define("SUPPORT_FILEFORMAT_BDF", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_BDF")));
    cmake.define("SUPPORT_FILEFORMAT_OBJ", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_OBJ")));
    cmake.define("SUPPORT_FILEFORMAT_MTL", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_MTL")));
    cmake.define("SUPPORT_FILEFORMAT_IQM", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_IQM")));
    cmake.define("SUPPORT_FILEFORMAT_GLTF", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_GLTF")));
    cmake.define("SUPPORT_FILEFORMAT_VOX", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_VOX")));
    cmake.define("SUPPORT_FILEFORMAT_M3D", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_M3D")));
    cmake.define("SUPPORT_MESH_GENERATION", bstr(cfg!(feature = "SUPPORT_MESH_GENERATION")));
    cmake.define("SUPPORT_GPU_SKINNING", bstr(cfg!(feature = "SUPPORT_GPU_SKINNING")));
    cmake.define("SUPPORT_FILEFORMAT_WAV", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_WAV")));
    cmake.define("SUPPORT_FILEFORMAT_OGG", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_OGG")));
    cmake.define("SUPPORT_FILEFORMAT_MP3", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_MP3")));
    cmake.define("SUPPORT_FILEFORMAT_QOA", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_QOA")));
    cmake.define("SUPPORT_FILEFORMAT_FLAC", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_FLAC")));
    cmake.define("SUPPORT_FILEFORMAT_XM", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_XM")));
    cmake.define("SUPPORT_FILEFORMAT_MOD", bstr(cfg!(feature = "SUPPORT_FILEFORMAT_MOD")));
    cmake.define("SUPPORT_TRACELOG", bstr(cfg!(feature = "SUPPORT_TRACELOG")));
    cmake.define("SUPPORT_SCREEN_CAPTURE", bstr(cfg!(feature = "SUPPORT_SCREEN_CAPTURE")));
}
#[must_use]
fn bstr(b: bool) -> &'static str {
    if b { "ON" } else { "OFF" }
}
