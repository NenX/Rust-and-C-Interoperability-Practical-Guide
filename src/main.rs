// 这是我们的入口文件，用来调用静态库和动态库
// This is our entry file for calling both static and dynamic libraries
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
