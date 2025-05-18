// 这是我们的入口文件，用来调用静态库和动态库
// This is our entry file for calling both static and dynamic libraries

use std::ffi::{self, c_char, c_int};

use libloading::{Library, Symbol};

extern "C" {
    fn add(a: c_int, b: c_int, result: *mut c_char) -> c_int;
    fn cdylib_add(a: c_int, b: c_int, result: *mut c_char) -> c_int;
    fn staticlib_add(a: c_int, b: c_int, result: *mut c_char) -> c_int;
}
fn buf(label: &str, capacity: usize) -> Vec<i8> {
    let mut b = label.as_bytes().to_vec();
    let len = b.len();
    if len < capacity {
        b.extend_from_slice(&vec![u8::MIN; capacity - len]);
    };

    b.iter().map(|&i| i as i8).collect()
}

macro_rules! CallLibFn {
    ($call_fn:expr, $arg1:expr, $arg2:expr, $buf:expr, $t:expr) => {
        let mut b = $buf;

        println!("[Rust] Calling function in {}", $t);
        let result = $call_fn($arg1, $arg2, b.as_mut_ptr());
        let msg = ffi::CStr::from_ptr(b.as_ptr()).to_str().unwrap();
        println!("{}", msg);
        println!("[Rust] Result from {}: {}\n", $t, result);
    };
}
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
