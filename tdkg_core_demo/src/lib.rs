use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern "C" fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

#[no_mangle]
pub extern "C" fn get_version() -> *mut c_char {
    let version = CString::new("tdkg_core v0.1.0").unwrap();
    version.into_raw()
}

#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn calculate_sum(array: *const i32, length: usize) -> i32 {
    if array.is_null() || length == 0 {
        return 0;
    }
    
    let slice = unsafe { std::slice::from_raw_parts(array, length) };
    slice.iter().sum()
}

#[no_mangle]
pub extern "C" fn create_hello_message(name: *const c_char) -> *mut c_char {
    if name.is_null() {
        let message = CString::new("Hello, World!").unwrap();
        return message.into_raw();
    }
    
    let name_str = unsafe { CStr::from_ptr(name).to_string_lossy() };
    let message = CString::new(format!("Hello, {}!", name_str)).unwrap();
    message.into_raw()
} 