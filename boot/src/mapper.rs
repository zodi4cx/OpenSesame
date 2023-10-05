use core::{ffi::c_void, slice};
use core::{ffi::CStr, ptr};
use windows_sys::Win32::System::SystemServices::IMAGE_IMPORT_BY_NAME;
use windows_sys::Win32::System::{
    Diagnostics::Debug::{IMAGE_NT_HEADERS64, IMAGE_SECTION_HEADER},
    SystemServices::{
        IMAGE_DOS_HEADER, IMAGE_DOS_SIGNATURE, IMAGE_EXPORT_DIRECTORY, IMAGE_IMPORT_DESCRIPTOR,
        IMAGE_NT_SIGNATURE,
    },
    WindowsProgramming::IMAGE_THUNK_DATA64,
};

enum ImageDirectory {
    EntryExport = 0,
    EntryImport = 1,
    _EntryBaseReloc = 5,
}

/// Maps the driver manually into memory within winload context.
pub unsafe fn map_driver(
    driver_base: *mut c_void,
    ntoskrnl_base: *const c_void,
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
    let sections = slice::from_raw_parts(
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

    log::info!("[*] Resolving ntoskrnl imports");
    let imports_rva = (*driver_nt_headers).OptionalHeader.DataDirectory
        [ImageDirectory::EntryImport as usize]
        .VirtualAddress;
    if imports_rva != 0 {
        let mut import_descriptor =
            driver_base.add(imports_rva as _) as *const IMAGE_IMPORT_DESCRIPTOR;
        while (*import_descriptor).FirstThunk != 0 {
            let mut thunk = driver_base
                .add((*import_descriptor).FirstThunk as _)
                .cast::<IMAGE_THUNK_DATA64>();
            let mut thunk_original = driver_base
                .add((*import_descriptor).Anonymous.OriginalFirstThunk as _)
                as *const IMAGE_THUNK_DATA64;
            while (*thunk).u1.AddressOfData != 0 {
                let export_data = driver_base.add((*thunk_original).u1.AddressOfData as _)
                    as *const IMAGE_IMPORT_BY_NAME;
                let export_name = CStr::from_ptr((*export_data).Name.as_ptr() as _);
                let import =
                    get_export(ntoskrnl_base, export_name).expect("Failed to resolve all imports");
                (*thunk).u1.Function = import as _;
                thunk = thunk.add(1);
                thunk_original = thunk_original.add(1);
            }
            import_descriptor = import_descriptor.add(1);
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

unsafe fn get_export(base: *const c_void, export: &CStr) -> Option<*const c_void> {
    let nt_headers = image_nt_headers(base).expect("Failed to parse NT Headers");
    let exports_rva = (*nt_headers).OptionalHeader.DataDirectory
        [ImageDirectory::EntryExport as usize]
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
        log::debug!("[D] Current symbol: {func:?}");
        if export == func {
            let func_rva = slice::from_raw_parts(
                base.add(exports.AddressOfFunctions as _).cast::<u32>(),
                exports.NumberOfFunctions as _,
            );
            let ordinal_rva = slice::from_raw_parts(
                base.add(exports.AddressOfNameOrdinals as _).cast::<u16>(),
                exports.NumberOfNames as _,
            );
            return Some(base.add(func_rva[ordinal_rva[i as usize] as usize] as usize));
        }
    }
    None
}