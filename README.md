# Rust and C Interoperability Practical Guide

English · [中文](./README-zh_CN.md)



This tutorial systematically introduces how to integrate C code into a Rust project, how to use Rust to compile dynamic and static libraries that conform to the C ABI, and how to call these libraries from Rust. It is suitable for developers with some Rust/C experience.

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

Create the `packages` directory and enter it:

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

In the call_libs crate, create the C source file implementing the required C function.

Create a `c` directory and add `clib.c`:

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

Add the `cc` dependency to automatically compile the C source code.

In `packages/call_libs/Cargo.toml` add `cc` as a build-dependency:

```toml
[build-dependencies]
cc = "1.0"
```

### 1.4 Write build.rs

Use a build script to automatically compile the C code and link it to the Rust project.

In `packages/call_libs`, create `build.rs`:

```rust
fn main() {
    cc::Build::new().file("c/clib.c").compile("clib");
}
```

### 1.5 Call C Functions from Rust

Declare the C function via FFI and call it safely in Rust code.

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
        call_lib_fn!(add, 1, 2, buf("Lucy", 1024), "C source");
    }
}
```

Run:

```shell
cargo run
```

Sample output:

```
[Rust] Calling C source
[C source] Hello Lucy
[C source] The result (1 + 2) is 3!
[Rust] C source returned: 3
```

> **Tip**: If you encounter linking errors, check the C file path, build.rs configuration, and dependencies.

---

## 2. Compiling Dynamic and Static Libraries with Rust (C ABI)

This section explains how to use Rust to generate dynamic and static libraries callable from C and other languages.

### 2.1 Compile a Dynamic Library (cdylib)

Create a library crate, set crate-type to `cdylib`, and implement the exported function.

In the `packages` directory, create the library crate:

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

After building, `cdylib_gen.dll` (on Windows) will be generated in `target/debug/`.

### 2.2 Compile a Static Library (staticlib)

Create a library crate, set crate-type to `staticlib`, and implement the exported function.

Also in the `packages` directory, create the static library crate:

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

After building, `staticlib_gen.lib` (on Windows) will be generated in `target/debug/`.

---

## 3. Calling C ABI Dynamic and Static Libraries from Rust

This section explains how to link and call dynamic and static libraries in a Rust project.

Return to the `call_libs` directory:

```shell
cd ../call_libs
```

### 3.1 Configure build.rs to Link Libraries

Edit `build.rs` and add the following:

```rust
fn main() {
    // ...existing code...
    let profile = std::env::var("PROFILE").unwrap();
    let search_dir = format!("../../target/{}", profile); // Note the path
    println!("cargo:rustc-link-search=native={}", search_dir);
    println!("cargo:rustc-link-lib=dylib=cdylib_gen");
    println!("cargo:rustc-link-lib=static=staticlib_gen");
}
```

> **Note**: `cdylib_gen.dll` and `staticlib_gen.lib` must be in the `target/{profile}` directory, or specify the path via `cargo:rustc-link-search`.

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
        call_lib_fn!(add, 1, 2, buf("Lucy", 1024), "C source");
        call_lib_fn!(cdylib_add, 1, 2, buf("Lee", 1024), "dynamic library");
        call_lib_fn!(staticlib_add, 3, 4, buf("Chen", 1024), "static library");
    }
}
```

Run:

```shell
cargo run
```

Sample output:

```
[Rust] Calling C source
[C source] Hello Lucy
[C source] The result (1 + 2) is 3!
[Rust] C source returned: 3

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

## Common Issues and Tips

- Path issues: Ensure the library search path in `build.rs` is correct.
- On Windows, dynamic libraries must be in the same directory as the executable or in the PATH.
- When using Rust/C FFI, pay attention to type matching; it is recommended to use types from `std::os::raw`.
- If you encounter linking errors, check the library name, path, and crate-type configuration.

---

## Summary

With this tutorial, you can:

1. Integrate and call C code in a Rust project;
2. Use Rust to compile dynamic and static libraries conforming to the C ABI;
3. Call these libraries from Rust, enabling cross-language interoperability.

For complete example code, refer to the project structure and code snippets above.
