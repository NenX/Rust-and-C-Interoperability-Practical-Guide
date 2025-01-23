// 这是我们的构建脚本
// This is our build script
use std::ffi::OsStr;
use std::path::Path;
use std::{env, fs};

extern crate cc;

fn move_dylib_files(from_dir: &Path, to_dir: &Path) -> std::io::Result<()> {
    if !to_dir.exists() {
        fs::create_dir_all(to_dir)?;
    }

    for entry in fs::read_dir(from_dir)? {
        let entry = entry?;
        let path = entry.path();
        let ext = path.extension();
        if path.is_file() && (ext == Some(OsStr::new("dll")) || ext == Some(OsStr::new("so"))) {
            let dest_path = to_dir.join(path.file_name().unwrap());
            println!("cargo::warning=move file from {path:?} to {dest_path:?}");
            fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}

// src 不能包含 lib.rs 否则编译不成功
fn main() {
    let profile = env::var("PROFILE").unwrap();
    let profile_dir = format!("./target/{}/", profile);
    let lib_dir = String::from("./src_lib/lib_build");

    cc::Build::new().file("c/add.c").compile("add");

    let libs_dir = "./src_lib/lib_build";
    println!("cargo::rustc-link-search=native={}", libs_dir);
    println!("cargo::rustc-link-lib=dylib=dylib_for_rust");
    println!("cargo::rustc-link-lib=static=static_for_rust");

    move_dylib_files(Path::new(&lib_dir), Path::new(&profile_dir))
        .expect("failed to move dylib files");
}
