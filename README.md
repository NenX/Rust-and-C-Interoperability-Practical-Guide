# Practical Tutorial on Rust and C Interoperability

English · [中文](./README-zh_CN.md)

This tutorial systematically introduces the following aspects:
- How to integrate C code into a Rust project
- How to compile dynamic and static libraries in Rust that conform to the C ABI
- How to link and call static and dynamic libraries in Rust
- How to dynamically load, bind, and call dynamic libraries in Rust

This repository contains the final code for this tutorial, which works well on both Windows and Linux.

Clone it and run with `cargo run`:
```shell
[Rust] Calling function in C source code
[C source] Hello Lucy
[C source] The result (1 + 2) is 3!
[Rust] Result from C source code: 3

[Rust] Calling function in dynamic library
[Rust cdylib] Hello Lee
[Rust cdylib] The result (1 + 2) is 3!
[Rust] Result from dynamic library: 3

[Rust] Calling function in static library
[Rust staticlib] Hello Chen
[Rust staticlib] The result (3 + 4) is 7!
[Rust] Result from static library: 7

[Rust] Calling function in dynamic loading library
[External dyloading] Hello Jack
[External dyloading] The result (8 + 9) is 17!
[Rust] Result from dynamic loading library: 17
```

---

## 1. Integrating C Code in a Rust Project

This section explains how to directly integrate and call C source code in a Rust project.

### 1.1 Create a Workspace

First, create a workspace and configure its members:

```shell
cargo new my_workspace --lib
cd my_workspace
```

Edit the root `Cargo.toml`:

```toml
[workspace]
members = ["packages/*"]
```

Create a `packages` directory and enter it:

```shell
mkdir packages
cd packages
```

Create a binary crate for integrating C code:

```shell
cargo new call_libs
cd call_libs
```

### 1.2 Add C Source Code

Create a C source file under `call_libs` to implement the required C functions.

Create a `c` directory and add a `clib.c` file:

```c
#include <stdio.h>
#include <stdint.h>

int32_t add(int32_t a, int32_t b, char *result) {
    printf("[C source] Hello %s\n", result);
    int32_t sum = a + b;
    sprintf(result, "[C source] The result (%d + %d) is %d!", a, b, sum);
    return sum;
}
```

### 1.3 Configure Cargo.toml

Add the `cc` dependency to automatically compile C source code.

In `packages/call_libs/Cargo.toml`, add `cc` as a build-dependency:

```toml
[build-dependencies]
cc = "1.0"
```

### 1.4 Write build.rs

Use a build script to automatically compile the C code and link it to the Rust project.

Create `build.rs` under `packages/call_libs`:

```rust
fn main() {
    cc::Build::new().file("c/clib.c").compile("clib");
}
```

### 1.5 Call C Functions in Rust

Declare the C function via FFI and safely call it in Rust code.

Edit `src/main.rs`:

```rust
use std::ffi::{CStr, c_char, c_int};

extern "C" {
    fn add(a: c_int, b: c_int, result: *mut c_char) -> c_int;
}

fn buf(label: &str, capacity: usize) -> Vec<i8> {
    let mut b = label.as_bytes().to_vec();
    b.resize(capacity, 0);
    b.iter().map(|&i| i as i8).collect()
}

macro_rules! call_lib_fn {
    ($fn:expr, $a:expr, $b:expr, $buf:expr, $desc:expr) => {{
        let mut b = $buf;
        println!("[Rust] Calling {}", $desc);
        let result = $fn($a, $b, b.as_mut_ptr());
        let msg = unsafe { CStr::from_ptr(b.as_ptr()).to_str().unwrap() };
        println!("{}", msg);
        println!("[Rust] {} returned: {}\n", $desc, result);
    }};
}

fn main() {
    unsafe {
        call_lib_fn!(add, 1, 2, buf("Lucy", 1024), "C source code");
    }
}
```

Run `cargo run`

Output:

```
[Rust] Calling C source code
[C source] Hello Lucy
[C source] The result (1 + 2) is 3!
[Rust] C source code returned: 3
```

> **Tip**: If you encounter linking errors, check the C file path, build.rs configuration, and dependencies.

---

## 2. Compiling Dynamic and Static Libraries with Rust (C ABI)

This section introduces how to generate dynamic and static libraries in Rust that can be called from C and other languages.

### 2.1 Compile a Dynamic Library (cdylib)

Create a library crate, set crate-type to `cdylib`, and implement the exported function.

In the `packages` directory, create a library crate:

```shell
cd ..
cargo new cdylib_gen --lib
cd cdylib_gen
```

Edit `Cargo.toml` to add crate-type:

```toml
[lib]
crate-type = ["cdylib"]
```

Edit `src/lib.rs`:

```rust
use std::ffi::{CStr, CString, c_char, c_int};
use std::ptr;

#[no_mangle]
pub extern "C" fn cdylib_add(a: c_int, b: c_int, result: *mut c_char) -> c_int {
    let sum = a + b;
    unsafe {
        let name = CStr::from_ptr(result).to_str().unwrap();
        println!("[Rust cdylib] Hello {name}");
        let msg = format!("[Rust cdylib] The result ({a} + {b}) is {sum}!");
        let msg = CString::new(msg).unwrap();
        ptr::copy_nonoverlapping(msg.as_ptr(), result, msg.as_bytes().len() + 1);
    }
    sum
}
```

Build:

```shell
cargo build
```

After building, `cdylib_gen.dll` will be generated under `target/debug/` (on Windows, on Linux it is `libcdylib_gen.so`).

### 2.2 Compile a Static Library (staticlib)

Create a library crate, set crate-type to `staticlib`, and implement the exported function.

Similarly, in the `packages` directory, create a static library crate:

```shell
cd ..
cargo new staticlib_gen --lib
cd staticlib_gen
```

Edit `Cargo.toml` to add crate-type:

```toml
[lib]
crate-type = ["staticlib"]
```

Edit `src/lib.rs`:

```rust
use std::ffi::{CStr, CString, c_char, c_int};
use std::ptr;

#[no_mangle]
pub extern "C" fn staticlib_add(a: c_int, b: c_int, result: *mut c_char) -> c_int {
    let sum = a + b;
    unsafe {
        let name = CStr::from_ptr(result).to_str().unwrap();
        println!("[Rust staticlib] Hello {name}");
        let msg = format!("[Rust staticlib] The result ({a} + {b}) is {sum}!");
        let msg = CString::new(msg).unwrap();
        ptr::copy_nonoverlapping(msg.as_ptr(), result, msg.as_bytes().len() + 1);
    }
    sum
}
```

Build:

```shell
cargo build
```

After building, `staticlib_gen.lib` will be generated under `target/debug/` (on Windows, on Linux it is `libstaticlib_gen.a`).

---

## 3. Linking and Calling C ABI Dynamic and Static Libraries in Rust

This section introduces how to link and call dynamic and static libraries in a Rust project.

Go back to the `call_libs` directory:

```shell
cd ../call_libs
```

### 3.1 Configure build.rs to Link Libraries

Edit `build.rs` and add the following:

```rust
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
```

> **Note**: The dynamic and static libraries must be in the `target/{profile}` directory, or specify the path via `cargo:rustc-link-search`.

### 3.2 Call Library Functions in main.rs

Edit `src/main.rs`:

```rust
extern "C" {
    fn add(a: c_int, b: c_int, result: *mut c_char) -> c_int;
    fn cdylib_add(a: c_int, b: c_int, result: *mut c_char) -> c_int;
    fn staticlib_add(a: c_int, b: c_int, result: *mut c_char) -> c_int;
}

fn main() {
    unsafe {
        call_lib_fn!(add, 1, 2, buf("Lucy", 1024), "C source code");
        call_lib_fn!(cdylib_add, 1, 2, buf("Lee", 1024), "dynamic library");
        call_lib_fn!(staticlib_add, 3, 4, buf("Chen", 1024), "static library");
    }
}
```

Run `cargo run`

Output:

```
[Rust] Calling C source code
[C source] Hello Lucy
[C source] The result (1 + 2) is 3!
[Rust] C source code returned: 3

[Rust] Calling dynamic library
[Rust cdylib] Hello Lee
[Rust cdylib] The result (1 + 2) is 3!
[Rust] dynamic library returned: 3

[Rust] Calling static library
[Rust staticlib] Hello Chen
[Rust staticlib] The result (3 + 4) is 7!
[Rust] static library returned: 7
```

---

## 4. Dynamically Loading and Binding Dynamic Libraries with libloading

This section introduces how to use the [libloading](https://crates.io/crates/libloading) crate to dynamically load dynamic libraries (such as `.so` or `.dll`) at runtime and bind their functions.

### 4.1 Build an External Dynamic Library

Go to the `external_lib` directory and run `./build.sh` (on Windows use `.\build.ps1`).

You should see `external_dy.dll` (on Linux it is `libexternal_dy.so`) in `external_lib/lib_build`.

### 4.2 Add Dependency

Add `libloading` to `packages/call_libs/Cargo.toml`:

```toml
[dependencies]
libloading = "0.8"
```

### 4.3 Example: Dynamically Load cdylib_gen Dynamic Library

Edit `src/main.rs` and add the following code:

```rust
use libloading::{Library, Symbol};
use std::ffi::{CStr, CString, c_char, c_int};

unsafe fn dynamic_load_bind() {
    #[cfg(target_os = "linux")]
    let lib_file = "libexternal_dy.so";
    #[cfg(target_os = "windows")]
    let lib_file = "external_dy.dll";
    let lib_path = format!("external_lib/lib_build/{}", lib_file);
    
    if Path::new(&lib_path).exists() {
        let lib = Library::new(lib_path).expect("Failed to load the dynamic library.");
        type CdylibAdd = unsafe extern "C" fn(c_int, c_int, *mut c_char) -> c_int;
        let dyloading_add: Symbol<CdylibAdd> = lib
            .get(b"dyloading_add")
            .expect("Failed to find the symbol.");

        CallLibFn! { dyloading_add, 8, 9, buf("Jack", 1024), "dynamic loading library" };
    }
}
fn main() {
    unsafe {
        CallLibFn! { add, 1, 2, buf("Lucy", 1024), "C source code" };
        CallLibFn! { cdylib_add, 1, 2, buf("Lee", 1024), "dynamic library" };
        CallLibFn! { staticlib_add, 3, 4, buf("Chen", 1024), "static library" };
        dynamic_load_bind()
    }
}
```

### 4.4 Running Result

Run `cargo run`, and you will see output similar to:

```
[Rust] Calling function in dynamic loading library
[External dyloading] Hello Jack
[External dyloading] The result (8 + 9) is 17!
[Rust] Result from dynamic loading library: 17
```

---

## Summary

Through this tutorial, you can learn how Rust and C interact. For complete example code, refer to the project structure and code snippets above.
