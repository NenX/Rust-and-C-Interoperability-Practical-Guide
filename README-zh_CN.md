# Rust 与 C 交互实践教程

[English](./README.md) · 中文



本教程系统介绍以下几个方面:
- 如何在 Rust 工程中集成 C 代码
- 如何用 Rust 编译出遵循 C 语言二进制接口（ABI）的动态库和静态库
- 如何在 Rust 中链接和调用静态库和动态库
- 如何在 Rust 中动态加载、绑定并调用动态库


这个存储库包含本教程的最终代码，它在Windows和Linux上都能很好地工作。

克隆它并使用 `cargo run` 运行：
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

## 1. 在 Rust 工程中集成 C 代码

本节介绍如何在 Rust 项目中直接集成和调用 C 语言源码。

### 1.1 新建 Workspace

首先，新建一个 workspace，并配置成员：

```shell
cargo new my_workspace --lib
cd my_workspace
```

编辑根目录下 `Cargo.toml`：

```toml
[workspace]
members = ["packages/*"]
```

新建 `packages` 目录并进入：

```shell
mkdir packages
cd packages
```

创建一个 bin crate 用于集成 C 代码：

```shell
cargo new call_libs
cd call_libs
```

### 1.2 添加 C 源码

在 call_libs 下新建 C 源码文件，实现你需要的 C 函数。

新建 `c` 目录，并添加 `clib.c` 文件：

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

### 1.3 配置 Cargo.toml

为项目添加 `cc` 依赖，用于自动编译 C 源码。

在 `packages/call_libs/Cargo.toml` 添加 `cc` 作为 build-dependencies：

```toml
[build-dependencies]
cc = "1.0"
```

### 1.4 编写 build.rs

通过 build 脚本自动编译 C 代码并链接到 Rust 工程。

在 `packages/call_libs` 下新建 `build.rs`：

```rust
fn main() {
    cc::Build::new().file("c/clib.c").compile("clib");
}
```

### 1.5 在 Rust 中调用 C 函数

通过 FFI 声明 C 函数，并在 Rust 代码中安全调用。

编辑 `src/main.rs`：

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
        println!("[Rust] 调用 {}", $desc);
        let result = $fn($a, $b, b.as_mut_ptr());
        let msg = unsafe { CStr::from_ptr(b.as_ptr()).to_str().unwrap() };
        println!("{}", msg);
        println!("[Rust] {} 返回: {}\n", $desc, result);
    }};
}

fn main() {
    unsafe {
        call_lib_fn!(add, 1, 2, buf("Lucy", 1024), "C 源码");
    }
}
```

执行 `cargo run`



输出：

```
[Rust] 调用 C 源码
[C source] Hello Lucy
[C source] The result (1 + 2) is 3!
[Rust] C 源码 返回: 3
```

> **提示**：如遇到链接错误，检查 C 文件路径、build.rs 配置和依赖项。

---

## 2. 用 Rust 编译出遵循 C ABI 的动态库和静态库

本节介绍如何用 Rust 生成可被 C 语言等调用的动态库和静态库。

### 2.1 编译动态库（cdylib）

新建库 crate，配置 crate-type 为 `cdylib`，实现导出函数。

在 `packages` 目录下创建库 crate：

```shell
cd ..
cargo new cdylib_gen --lib
cd cdylib_gen
```

编辑 `Cargo.toml`，添加 crate-type：

```toml
[lib]
crate-type = ["cdylib"]
```

编辑 `src/lib.rs`：

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

编译：

```shell
cargo build
```

编译完成后，在 `target/debug/` 下会生成 `cdylib_gen.dll`（Windows 下, 在 Linux 下是 libcdylib_gen.so）。

### 2.2 编译静态库（staticlib）

新建库 crate，配置 crate-type 为 `staticlib`，实现导出函数。

同样在 `packages` 目录下，创建静态库 crate：

```shell
cd ..
cargo new staticlib_gen --lib
cd staticlib_gen
```

编辑 `Cargo.toml`，添加 crate-type：

```toml
[lib]
crate-type = ["staticlib"]
```

编辑 `src/lib.rs`：

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

编译：

```shell
cargo build
```

编译完成后，在 `target/debug/` 下会生成 `staticlib_gen.lib`（Windows 下, 在 Linux 下是 libstaticlib_gen.a）。

---

## 3. 在 Rust 中调用遵循 C ABI 的动态库和静态库

本节介绍如何在 Rust 工程中链接和调用动态库和静态库。

回到 `call_libs` 目录：

```shell
cd ../call_libs
```

### 3.1 配置 build.rs 链接库

编辑 `build.rs`，添加如下内容：

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

> **注意**：动态库和静态库需位于 `target/{profile}` 目录下，或通过 `cargo:rustc-link-search` 指定路径。

### 3.2 在 main.rs 调用库函数

编辑 `src/main.rs`：

```rust
extern "C" {
    fn add(a: c_int, b: c_int, result: *mut c_char) -> c_int;
    fn cdylib_add(a: c_int, b: c_int, result: *mut c_char) -> c_int;
    fn staticlib_add(a: c_int, b: c_int, result: *mut c_char) -> c_int;
}

fn main() {
    unsafe {
        call_lib_fn!(add, 1, 2, buf("Lucy", 1024), "C 源码");
        call_lib_fn!(cdylib_add, 1, 2, buf("Lee", 1024), "动态库");
        call_lib_fn!(staticlib_add, 3, 4, buf("Chen", 1024), "静态库");
    }
}
```

执行 `cargo run`



输出：

```
[Rust] 调用 C 源码
[C source] Hello Lucy
[C source] The result (1 + 2) is 3!
[Rust] C 源码 返回: 3

[Rust] 调用 动态库
[Rust cdylib] Hello Lee
[Rust cdylib] The result (1 + 2) is 3!
[Rust] 动态库 返回: 3

[Rust] 调用 静态库
[Rust staticlib] Hello Chen
[Rust staticlib] The result (3 + 4) is 7!
[Rust] 静态库 返回: 7
```

---

## 4. 利用 libloading 动态加载并绑定动态链接库

本节介绍如何使用 [libloading](https://crates.io/crates/libloading) crate，在运行时动态加载动态链接库（如 `.so` 或 `.dll`），并绑定其中的函数。

### 4.1 制作我们的外部动态链接库

进入 `external_lib` 目录执行脚本:


```shell
./build.sh # 如果你使用 windows 系统, 请换成 .\build.ps1
```
我可以看到在 `external_lib/lib_build` 出现了 `libexternal_dy.so`(在 windows 上是 `external_dy.dll`)
### 4.2 添加依赖

在 `packages/call_libs/Cargo.toml` 中添加 `libloading` 依赖：

```toml
[dependencies]
libloading = "0.8"
```

### 4.3 示例：动态加载 cdylib_gen 动态库



编辑 `src/main.rs`，添加如下代码：

```rust
use libloading::{Library, Symbol};
use std::ffi::{CStr, CString, c_char, c_int};

fn dynamic_load_bind() {
    #[cfg(target_os = "linux")]
    let lib_file = "libexternal_dy.so";
    #[cfg(target_os = "windows")]
    let lib_file = "external_dy.dll";
    let lib_path = format!("external_lib/lib_build/{}",lib_file);

    unsafe {
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
    }
    dynamic_load_bind()
}

```

### 4.4 运行效果

执行 `cargo run`，你会看到类似如下输出：

```
[Rust] 动态加载调用 cdylib_add
[Rust cdylib] Hello Dylan
[Rust cdylib] The result (10 + 20) is 30!
[Rust] 动态加载 cdylib_add 返回: 30
```

---

> **提示**：  
> - 使用 libloading 可以让你的程序在运行时灵活加载/卸载动态库，适合插件机制等场景。  
> - 注意符号名需与库中导出的函数名一致，且类型声明要完全匹配。  
> - 路径需指向已编译好的动态库文件。

---

你可以根据实际项目结构调整路径和调用方式。

---

## 总结

通过本教程，你可以：

1. 在 Rust 工程中集成并调用 C 代码；
2. 用 Rust 编译出遵循 C ABI 的动态库和静态库；
3. 在 Rust 中调用这些库，实现跨语言互操作。

如需完整示例代码，可参考本项目结构和上述代码片段。
