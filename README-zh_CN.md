# 如何从Rust调用C库

[English](./README.md) · 中文

本教程的灵感来自 [rust-ffi-to-c](https://github.com/vanjacosic/rust-ffi-to-c)

这是一个关于如何从Rust调用C库的保姆教程。

这个存储库包含本教程的最终代码，它在Windows和Linux上都能很好地工作。

克隆它并使用 `cargo run` 运行.

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

## 教程

在本教程中，Rust 与 C 的交互采用两种形式。第一种，由 Cargo 编译C源码程序，Rust 调用其中的函数。第二种，我们需要处理静态和动态的C库。


### 1. 通过 Cargo 编译 C 源代码，并在 Rust 中调用函数。

首先，我们需要执行 `cargo new hello-ffi` 来创建一个全新的项目来执行我们的实验。

现在，假设我们有一些 C 源代码，创建一个新的 `c/add.c` 文件，我们需要在Rust中调用 `add()` 这个函数。

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
这个函数接受 `int32_t` 类型的两个参数，计算它们的和并将其返回给调用者。


下一步的工作，我们需要安装 [`cc`](https://crates.io/crates/cc) crate.



```toml
[build-dependencies]
cc = "1.0"
```

然后创建一个 `build.rs` 文件，并编写编译脚本，它告诉 Cargo 如何正确编译 C 源代码。

```rust
extern crate cc;

fn main() {
    cc::Build::new().file("c/add.c").compile("add");
}
```

在 rustc 开始编译我们的 Rust 程序之前，`build.rs` 将会被调用来编译 C 源代码。

但是还需要告诉 rustc 我们的 C 函数长什么样子。修改 `src/main.rs`:

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


我们使用 [`extern`](https://doc.rust-lang.org/reference/items/external-blocks.html) 来引用 C 语言编写的 `add()` 函数，
从 `core:ffi` 导入兼容 C 的整数类型 `c_int` 到 Rust 中。

任何外部函数的使用都被 rustc 认为是不安全的，因为 Rust 编译器不能保证外部代码中的内存安全。
因此，在的 `src/main.rs` 中，我们在一个 `unsafe` 块中调用该函数，然后传入两个 `i32` 整数，并打印结果。

现在我们可以使用 Cargo 来构建 C 和 Rust 代码并运行程序：

```shell
$ cargo clean && cargo run
```

### 2. 链接 C 库并调用其中的函数。

假设你从其他地方获得了一些动态和静态 C 库，现在你需要在 Rust 中调用这些库函数。


创建一个新的 `src_lib/lib_build` 文件夹来存储 C 库文件(当然，您可以在此存储库中复制相应的文件以供实验使用)。修改的 `build.rs` 告诉 Cargo 如何链接这些库文件。

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
[rustc-link-search](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-link-search) 指令告诉 Cargo 从哪里搜索库文件。 [rustc-link-lib](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-link-lib) 指令告诉 Cargo 哪些库文件需要链接。


对于动态库(Windows上的 `dll` 文件，Linux上的 `so` 文件)，我们还需要将它们移动到可执行程序所在的目录，因此我们定义 `move_dylib_files()` 函数来完成此操作。


修改 `src/main.rs`  告诉 rustc 我们要调用的库函数是什么样子的。请注意，这两个函数 `static_call()` 和 `dylib_call()` 是包含在此存储库中的静态和动态库文件中的函数，如果您使用自己的库文件，请修改相应的函数签名。


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
现在我们可以使用 Cargo 来链接 C 库并运行程序:
```shell
$ cargo clean && cargo run
```
