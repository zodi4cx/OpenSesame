#![no_std]

use core::{
    ffi::{c_void, CStr},
    slice,
};
use windows_sys::Win32::System::{
    Diagnostics::Debug::IMAGE_NT_HEADERS64,
    SystemServices::{
        IMAGE_DOS_HEADER, IMAGE_DOS_SIGNATURE, IMAGE_EXPORT_DIRECTORY, IMAGE_NT_SIGNATURE,
    },
};

pub enum ImageDirectoryEntry {
    Export = 0,
    Import = 1,
    BaseReloc = 5,
}

pub unsafe fn image_dos_header(module_base: *const c_void) -> Option<*const IMAGE_DOS_HEADER> {
    let dos_header = module_base.cast::<IMAGE_DOS_HEADER>();
    ((*dos_header).e_magic == IMAGE_DOS_SIGNATURE).then_some(dos_header)
}

pub unsafe fn image_nt_headers(module_base: *const c_void) -> Option<*const IMAGE_NT_HEADERS64> {
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

pub unsafe fn get_export(base: *const c_void, export: &CStr) -> Option<*const c_void> {
    let nt_headers = image_nt_headers(base).expect("Failed to parse NT Headers");
    let exports_rva = (*nt_headers).OptionalHeader.DataDirectory
        [ImageDirectoryEntry::Export as usize]
        .VirtualAddress;
    if exports_rva == 0 {
        return None;
    }
    let exports = *base.add(exports_rva as _).cast::<IMAGE_EXPORT_DIRECTORY>();
    let names_rva = slice::from_raw_parts(
        base.add(exports.AddressOfNames as _).cast::<u32>(),
        exports.NumberOfNames as _,
    );
    for (i, &name_rva) in names_rva.iter().enumerate() {
        let func = CStr::from_ptr(base.add(name_rva as _) as _);
        if export == func {
            let func_rva = slice::from_raw_parts(
                base.add(exports.AddressOfFunctions as _).cast::<u32>(),
                exports.NumberOfFunctions as _,
            );
            let ordinal_rva = slice::from_raw_parts(
                base.add(exports.AddressOfNameOrdinals as _).cast::<u16>(),
                exports.NumberOfNames as _,
            );
            return Some(base.add(func_rva[ordinal_rva[i] as usize] as usize));
        }
    }
    None
}
