// 这是我们的构建脚本
// This is our build script

extern crate cc;

// src 不能包含 lib.rs 否则编译不成功
fn main() {
    cc::Build::new().file("c/clib.c").compile("clib");

    let profile = std::env::var("PROFILE").unwrap();
    let search_dir = format!("target/{}", profile);
    println!("cargo::rustc-link-search=native={}", search_dir);
    if cfg!(target_os = "windows") {
        println!("cargo::rustc-link-lib=dylib=cdylib_gen.dll");
        println!("cargo::rustc-link-lib=static=staticlib_gen");
    } else {
        println!("cargo::rustc-link-lib=dylib=cdylib_gen");
        println!("cargo::rustc-link-lib=static=staticlib_gen");
    }
}
