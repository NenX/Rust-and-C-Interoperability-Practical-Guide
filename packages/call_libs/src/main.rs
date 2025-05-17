// 这是我们的入口文件，用来调用静态库和动态库
// This is our entry file for calling both static and dynamic libraries

use std::{
    ffi::{self, c_char, c_int}
};

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
fn main() {
    unsafe {
        CallLibFn! { add, 1, 2, buf("Lucy", 1024), "C source code" };
        CallLibFn! { cdylib_add, 1, 2, buf("Lee", 1024), "dynamic library" };
        CallLibFn! { staticlib_add, 3, 4, buf("Chen", 1024), "static library" };
    }
}
