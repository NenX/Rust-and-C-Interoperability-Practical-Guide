use std::ffi;
use std::ptr;

#[no_mangle]
pub extern "C" fn cdylib_add(a: ffi::c_int, b: ffi::c_int, result: *mut ffi::c_char) -> ffi::c_int {
    let sum = a + b;
    unsafe {
        let name = ffi::CStr::from_ptr(result).to_str().unwrap();

        println!("[Rust cdylib] Hello {name}");

        let msg = format!("[Rust cdylib] The result ({a} + {b}) is {sum}!");

        let msg = ffi::CString::new(msg).unwrap();

        ptr::copy_nonoverlapping(msg.as_ptr(), result, msg.as_bytes().len() + 1);
        return sum;
    }
}
