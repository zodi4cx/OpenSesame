use core::ffi::c_void;
use core::ptr;
use windows_sys::Win32::System::{
    Diagnostics::Debug::IMAGE_NT_HEADERS64,
    SystemServices::{IMAGE_DOS_HEADER, IMAGE_DOS_SIGNATURE, IMAGE_NT_SIGNATURE},
};

/// Maps the driver manually into memory within winload context.
pub unsafe fn map_driver(
    driver_base: *mut c_void,
    _ntoskrnl_base: *const c_void,
    _target_function: *mut c_void,
) {
    let driver_buffer = core::include_bytes!("../../target/x86_64-pc-windows-msvc/sesame.sys");

    log::info!("[*] Mapping headers");
    let driver_nt_headers =
        nt_headers(driver_buffer.as_ptr() as _).expect("Failed to parse NT Headers");
    ptr::copy_nonoverlapping(
        driver_buffer.as_ptr(),
        driver_base as _,
        driver_nt_headers.OptionalHeader.SizeOfHeaders as _,
    );
}

unsafe fn dos_header(module_base: *const c_void) -> Option<IMAGE_DOS_HEADER> {
    let dos_header = *(module_base as *const IMAGE_DOS_HEADER);
    (dos_header.e_magic == IMAGE_DOS_SIGNATURE).then_some(dos_header)
}

unsafe fn nt_headers(module_base: *const c_void) -> Option<IMAGE_NT_HEADERS64> {
    let nt_headers_offset = dos_header(module_base)
        .expect("Failed to parse DOS Header")
        .e_lfanew;
    let nt_headers = *(module_base.offset(nt_headers_offset as _) as *const IMAGE_NT_HEADERS64);
    (nt_headers.Signature == IMAGE_NT_SIGNATURE).then_some(nt_headers)
}
