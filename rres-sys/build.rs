use std::{env, path::PathBuf};

fn gen_rres() {
    let mut comp = &mut cc::Build::new();
    // Compile the code and link with cc crate
    #[cfg(target_os = "windows")]
    {
        comp = comp.files(vec!["binding/rres_wrapper.cpp"]);
    }
    #[cfg(not(target_os = "windows"))]
    {
        comp = comp.files(vec!["binding/rres_wrapper.c"]);
    }

    comp = comp
        .include("binding")
        .warnings(false)
        // .flag("-std=c99")
        .extra_warnings(false);

    comp.compile("rres");
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=bindings/*.c");
    println!("cargo:rerun-if-changed=bindings/*.h");
    println!("cargo:rerun-if-changed=bindings/*.cpp");
    let builder = bindgen::builder()
        .header("binding/binding.h")
        .rustified_enum(".+")
        // generate nothing from Raylib, since we're linking it to raylib_sys anyways.
        .blocklist_file("binding/raylib.h")
        .clang_arg("-std=c99");

    // Build
    let bindings = builder.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    gen_rres();
    Ok(())
}
