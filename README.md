# How to call C libraries from Rust

English · [中文](./README-zh_CN.md)

This tutorial was inspired by [rust-ffi-to-c](https://github.com/vanjacosic/rust-ffi-to-c)

A step-by-step guide on integrating C libraries with Rust applications.

This repository contains complete working code for both Windows and Linux platforms.
To get started:

## First Run on Linux
```shell
$ ./build_lib_and_run.sh

```
## First Run on Windows
```shell
$ ./build_lib_and_run.ps1

```
## Subsequent Runs
For subsequent runs, simply use `cargo run`.

```shell
$ cargo run

warning: test_ffi@0.1.0: move "./src_lib/lib_build/libdylib_for_rust.so" to "./target/debug/libdylib_for_rust.so"
warning: test_ffi@0.1.0: move "./src_lib/lib_build/dylib_for_rust.dll" to "./target/debug/dylib_for_rust.dll"
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running `target/debug/test_ffi`

[Rust] Calling function in C file
[C] source: Argument a is:{ 1 }, Argument b is:{ 5 }
[C] source: returning the result { 6 } to Rust
[Rust] Result from c file: 6

[Rust] Calling function in static library
[C] static_call: Argument a is:{ 10 }, Argument b is:{ 5 }
[C] static_call: returning the result { 15 } to Rust
[Rust] Result from static library: 15

[Rust] Calling function in dynamic library
[C] dylib_call: Argument a is:{ 100 }, Argument b is:{ 5 }
[C] dylib_call: returning the result { 105 } to Rust
[Rust] Result from dynamic library: 105

```

## Tutorial

This guide covers two primary methods of Rust-C interoperability. The first method demonstrates how to compile C code directly through Cargo and call its functions from Rust. The second method shows how to work with pre-compiled C libraries, both static and dynamic.

### 1. Compiling and Calling C Code through Cargo

Let's start by creating a new project:
```shell
cargo new hello-ffi
```

First, create a C source file `c/add.c` containing a simple addition function that we'll call from Rust:

```c
// add.c
#include <stdio.h>
#include <stdint.h>

int32_t add(int32_t a, int32_t b)
{
    int32_t result = a + b;
    printf("[C] source: Argument a is:{ %i }, Argument b is:{ %i } \n", a, b);
    printf("[C] source: returning the result { %i } to Rust\n", result);
    return result;
}

```
This function takes two arguments of type `int32_t`, computes their sum, and returns it to the caller.




Next, we'll need the [`cc`](https://crates.io/crates/cc) crate to handle C code compilation:


```toml
[build-dependencies]
cc = "1.0"
```

Create a `build.rs` file in the project root to configure C code compilation:

```rust
extern crate cc;

fn main() {
    cc::Build::new().file("c/add.c").compile("add");
}
```

Now we'll define the foreign function interface in `src/main.rs`:

```rust
// This is our entry file for calling both static and dynamic libraries
extern crate core;
use core::ffi::c_int;

extern "C" {
    fn add(a: c_int, b: c_int) -> c_int;
}

fn main() {

    unsafe {
        println!("[Rust] Calling function in C file");
        let result = add(1, 5);
        println!("[Rust] Result from c file: {} \n", result);
    }
}

```


We use [`extern`](https://doc.rust-lang.org/reference/items/external-blocks.html) to reference the `add()` function, which is written in C (`c/add.c`).

In this case we want to add integers, so we import a C-compatible integer type into Rust from `core:ffi`. 

We then define the argument types and return type for our C function as `c_int` (equivalent to `i32` in Rust).

Any use of foreign function is considered unsafe because the Rust compiler can't guarantee memory safety in foreign code. 
So in our main Rust file (`src/main.rs`) we call the function in an `unsafe` block, then pass in two `i32` integers, and print the result.

And now we can use Cargo to build both the C and Rust code and run the program:

```shell
$ cargo clean && cargo run
```

### 2. Working with Pre-compiled C Libraries

Let's explore how to work with pre-existing C libraries (both static and dynamic) in your Rust project.

To work with existing C libraries:

1. Create a `src_lib/lib_build` directory in your project:

2. Place your library files in directory `src_lib/lib_build/`:
    - Static library files (`.lib` on Windows, `.a` on Linux)
    - Dynamic library files (`.dll` on Windows, `.so` on Linux)

Then update `build.rs` to handle library linking:

```rust
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
            println!("cargo:warning=move {path:?} to {dest_path:?}");
            fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}

fn main() {
    let profile = env::var("PROFILE").unwrap();
    let profile_dir = format!("./target/{}/", profile);
    let lib_dir = String::from("./src_lib/lib_build");

    cc::Build::new().file("c/add.c").compile("add");

    let libs_dir = "./src_lib/lib_build";
    println!("cargo:rustc-link-search=native={}", libs_dir);
    println!("cargo:rustc-link-lib=dylib=dylib_for_rust");
    println!("cargo:rustc-link-lib=static=static_for_rust");

    move_dylib_files(&Path::new(&lib_dir), &Path::new(&profile_dir))
        .expect("failed to move dylib files");
}


```
The [rustc-link-search](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-link-search) instruction tells Cargo where to find the libraries. The [rustc-link-lib](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-link-lib) instruction tells Cargo to link the given library.

For dynamic libraries (`dll` files on Windows, `so` files on Linux), we also need to move them to the directory where the executable program resides, so we define the `move_dylib_files()` function to do this.

Modify `src/main.rs` to tell rustc what the library function we want to call looks like. Please note that the two functions `static_call()` and `dylib_call()` are the functions contained in the static and dynamic library files in this repository, and if you use your own library files, please modify the corresponding function signatures.



```rust
extern crate core;
use core::ffi::c_int;

extern "C" {
    fn add(a: c_int, b: c_int) -> c_int;
    fn static_call(a: c_int, b: c_int) -> c_int;
    fn dylib_call(a: c_int, b: c_int) -> c_int;
}

fn main() {

    unsafe {
        println!("[Rust] Calling function in C file");
        let result = add(1, 5);
        println!("[Rust] Result from c file: {} \n", result);

        println!("[Rust] Calling function in static library");
        let result = static_call(10, 5);
        println!("[Rust] Result from static library: {}\n", result);

        println!("[Rust] Calling function in dynamic library");
        let result = dylib_call(100, 5);
        println!("[Rust] Result from dynamic library: {}\n", result);
    }
}

```

To build and run the project:
```shell
$ cargo clean && cargo run
```
