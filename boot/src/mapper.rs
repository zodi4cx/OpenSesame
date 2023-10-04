use core::ffi::c_void;
use core::ptr;
use windows_sys::Win32::System::{
    Diagnostics::Debug::{IMAGE_NT_HEADERS64, IMAGE_SECTION_HEADER},
    SystemServices::{IMAGE_DOS_HEADER, IMAGE_DOS_SIGNATURE, IMAGE_NT_SIGNATURE},
};

enum ImageDirectory {
    EntryExport = 0,
    EntryImport = 1,
    EntryBaseReloc = 5,
}

/// Maps the driver manually into memory within winload context.
pub unsafe fn map_driver(
    driver_base: *mut c_void,
    _ntoskrnl_base: *const c_void,
    _target_function: *mut c_void,
) {
    let driver_buffer = crate::global::DRIVER_DATA;

    log::info!("[*] Mapping headers");
    let driver_nt_headers =
        image_nt_headers(driver_buffer.as_ptr() as _).expect("Failed to parse NT Headers");
    ptr::copy_nonoverlapping(
        driver_buffer.as_ptr(),
        driver_base as _,
        (*driver_nt_headers).OptionalHeader.SizeOfHeaders as _,
    );

    log::info!("[*] Mapping sections");
    let sections_header = ptr::addr_of!((*driver_nt_headers).OptionalHeader)
        .cast::<c_void>()
        .add((*driver_nt_headers).FileHeader.SizeOfOptionalHeader as _)
        as *const IMAGE_SECTION_HEADER;
    let sections = core::slice::from_raw_parts(
        sections_header,
        (*driver_nt_headers).FileHeader.NumberOfSections as _,
    );
    for &section in sections {
        if section.SizeOfRawData != 0 {
            ptr::copy_nonoverlapping(
                driver_buffer.as_ptr().add(section.PointerToRawData as _),
                driver_base.add(section.VirtualAddress as _) as _,
                section.SizeOfRawData as _,
            );
        }
    }
}

unsafe fn image_dos_header(module_base: *const c_void) -> Option<*const IMAGE_DOS_HEADER> {
    let dos_header = module_base.cast::<IMAGE_DOS_HEADER>();
    ((*dos_header).e_magic == IMAGE_DOS_SIGNATURE).then_some(dos_header)
}

unsafe fn image_nt_headers(module_base: *const c_void) -> Option<*const IMAGE_NT_HEADERS64> {
    let nt_headers_offset =
        (*image_dos_header(module_base).expect("Failed to parse DOS Header")).e_lfanew;
    let nt_headers = module_base
        .offset(nt_headers_offset as _)
        .cast::<IMAGE_NT_HEADERS64>();
    ((*nt_headers).Signature == IMAGE_NT_SIGNATURE).then_some(nt_headers)
}

pub unsafe fn size_of_image(module_base: *const c_void) -> u32 {
    (*image_nt_headers(module_base).expect("Failed to parse NT Headers"))
        .OptionalHeader
        .SizeOfImage
}
