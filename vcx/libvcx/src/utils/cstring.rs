use libc::c_char;

use std::ffi::CStr;
use std::str::Utf8Error;
use std::ffi::CString;

// TODO: refactor this into free functions
#[allow(non_snake_case)]
pub mod CStringUtils {
    use super::*;

    pub fn c_str_to_string(cstr: *const c_char) -> Result<Option<String>, Utf8Error> {
        c_str_to_str(cstr).map(|opt| opt.map(String::from))
    }
    pub fn c_str_to_opt_string(cstr: *const c_char) -> Option<String> {
        c_str_to_str(cstr).ok().flatten().map(String::from)
    }
    pub fn c_str_to_str<'a>(cstr: *const c_char) -> Result<Option<&'a str>, Utf8Error> {
        if cstr.is_null() {
            Ok(None)
        } else {
            // SAFETY: the pointer is non-null; we assume the foreign code has
            // upheld the other invariants
            unsafe {
                CStr::from_ptr(cstr).to_str().map(Some)
            }
        }

    }
    pub fn string_to_cstring(s: String) -> CString {
        CString::new(s).unwrap()
    }
}

// TODO: decide on a better place to put this
pub fn raw_slice_to_vec(ptr: *const u8, len: u32) -> Vec<u8> {
    if ptr.is_null() {
        Vec::new()
    } else {
        // SAFETY: the pointer is non-null; we assume the foreign code has
        // upheld the other invariants
        unsafe { std::slice::from_raw_parts(ptr, len as usize).to_vec() }
    }
}

//TODO DOCUMENT WHAT THIS DOES
macro_rules! check_useful_c_str {
    ($x:ident, $e:expr) => {
        let $x = match CStringUtils::c_str_to_string($x) {
            Ok(Some(val)) => val,
            _ => return VcxError::from_msg($e, "Invalid pointer has been passed").into()
        };

        if $x.is_empty() {
            return VcxError::from_msg($e, "Empty string has been passed").into()
        }
    }
}

macro_rules! check_useful_opt_c_str {
    ($x:ident, $e:expr) => {
        let $x = match CStringUtils::c_str_to_string($x) {
            Ok(opt_val) => opt_val,
            Err(_) => return VcxError::from_msg($e, "Invalid pointer has been passed").into()
        };
    }
}

/// Vector helpers
macro_rules! check_useful_c_byte_array {
    ($ptr:ident, $len:expr, $err1:expr, $err2:expr) => {
        if $ptr.is_null() {
            return VcxError::from_msg($err1, "Invalid pointer has been passed").into()
        }

        if $len <= 0 {
            return VcxError::from_msg($err2, "Array length must be greater than 0").into()
        }

        let $ptr = unsafe { $crate::std::slice::from_raw_parts($ptr, $len as usize) };
        let $ptr = $ptr.to_vec();
    }
}

//Returnable pointer is valid only before first vector modification
pub fn vec_to_pointer(v: &[u8]) -> (*const u8, u32) {
    let len = v.len() as u32;
    (v.as_ptr() as *const u8, len)
}
