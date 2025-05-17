# Rust 与 C 交互实践教程

[English](./README.md) · 中文



本教程系统介绍如何在 Rust 工程中集成 C 代码、如何用 Rust 编译出遵循 C 语言二进制接口（ABI）的动态库和静态库，以及如何在 Rust 中调用这些库。适合有一定 Rust/C 基础的开发者。


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
    // ...已有代码...
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

## 常见问题与提示

- 路径问题：确保 `build.rs` 中的库搜索路径正确。
- Windows 下动态库需在可执行文件同目录或 PATH 路径下。
- Rust/C 交互时注意类型匹配，推荐用 `std::os::raw` 类型。
- 若遇到链接错误，检查库名、路径、crate-type 配置。

---

## 总结

通过本教程，你可以：

1. 在 Rust 工程中集成并调用 C 代码；
2. 用 Rust 编译出遵循 C ABI 的动态库和静态库；
3. 在 Rust 中调用这些库，实现跨语言互操作。

如需完整示例代码，可参考本项目结构和上述代码片段。
