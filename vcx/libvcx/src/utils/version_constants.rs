use std::os::raw::c_char;
pub const VERSION_STRING: &str = "0.11.2+a26ad4f";
pub const VERSION_STRING_CSTR: *const c_char = "0.11.2+a26ad4f\0".as_ptr().cast();
