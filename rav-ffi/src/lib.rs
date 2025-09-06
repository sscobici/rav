use std::{ffi::CStr, os::raw::{c_char, c_void}, ptr::null_mut};

use rav::{data::Packet, format::FormatContext};

/// # Safety
/// This function should receive a pointer to c string.
#[no_mangle]
pub unsafe extern "C" fn rav_open_input(format_ctx: *mut *mut c_void, path: *mut c_char) -> i32 {
    if format_ctx.is_null() || path.is_null() {
        return -1;
    }

    let path = CStr::from_ptr(path).to_string_lossy().into_owned();
    let format = FormatContext::open_input(&path);
    *format_ctx = Box::into_raw(Box::new(format)) as *mut c_void;
    0
}


/// # Safety
/// This function should receive a pointer to FormatContext and pointer to Packet.
#[no_mangle]
pub unsafe extern "C" fn rav_read_packet(format_ctx: *mut c_void, packet: *mut c_void) -> i32 {
    if format_ctx.is_null() || packet.is_null() {
        return -1;
    }

    let format = Box::leak(Box::from_raw(format_ctx as *mut Box<FormatContext>));
    let packet = Box::leak(Box::from_raw(format_ctx as *mut Box<Packet>));
    match format.read_packet(packet) {
        Ok(()) => 0,
        Err(_) => -2,
    }
}
