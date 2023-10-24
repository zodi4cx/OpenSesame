use crate::global::{DRIVER_EXPORT_NAME, DRIVER_EXPORT_SIZE};
use common::ImageDirectoryEntry;
use core::{
    ffi::{c_void, CStr},
    mem, ptr, slice,
};
use windows_sys::Win32::System::SystemServices::{IMAGE_BASE_RELOCATION, IMAGE_IMPORT_BY_NAME};
use windows_sys::Win32::System::{
    Diagnostics::Debug::IMAGE_SECTION_HEADER, SystemServices::IMAGE_IMPORT_DESCRIPTOR,
    WindowsProgramming::IMAGE_THUNK_DATA64,
};

#[repr(u16)]
enum ImageRel {
    BasedAbsolute,
    BasedDir64,
    Unknown,
}

impl From<u16> for ImageRel {
    fn from(value: u16) -> Self {
        match value {
            0 => ImageRel::BasedAbsolute,
            10 => ImageRel::BasedDir64,
            _ => ImageRel::Unknown,
        }
    }
}

/// Maps the driver manually into memory within winload context. Returns driver's entrypoint.
pub unsafe fn map_driver(
    driver_base: *mut c_void,
    ntoskrnl_base: *const c_void,
    target_function: *mut c_void,
) -> *const c_void {
    let driver_buffer = crate::global::DRIVER_DATA;

    log::info!("[*] Mapping headers");
    let driver_nt_headers =
        common::image_nt_headers(driver_buffer.as_ptr() as _).expect("Failed to parse NT Headers");
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
        [ImageDirectoryEntry::Import as usize]
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
                let import = common::get_export(ntoskrnl_base, export_name)
                    .expect("Failed to resolve all imports");
                (*thunk).u1.Function = import as _;
                thunk = thunk.add(1);
                thunk_original = thunk_original.add(1);
            }
            import_descriptor = import_descriptor.add(1);
        }
    }

    log::info!("[*] Resolving relocations");
    let base_reloc_dir =
        (*driver_nt_headers).OptionalHeader.DataDirectory[ImageDirectoryEntry::BaseReloc as usize];
    if base_reloc_dir.VirtualAddress != 0 {
        let mut reloc =
            driver_base.add(base_reloc_dir.VirtualAddress as _) as *const IMAGE_BASE_RELOCATION;
        let mut current_size = 0;
        while current_size < base_reloc_dir.Size {
            let reloc_length = ((*reloc).SizeOfBlock as usize
                - mem::size_of::<IMAGE_BASE_RELOCATION>())
                / mem::size_of::<u16>();
            let reloc_address = reloc
                .cast::<c_void>()
                .add(mem::size_of::<IMAGE_BASE_RELOCATION>())
                as *const u16;
            let reloc_data = slice::from_raw_parts(reloc_address, reloc_length);
            let reloc_base = driver_base.add((*reloc).VirtualAddress as _);
            for &data in reloc_data {
                let reloc_type: ImageRel = (data >> 12).into();
                let reloc_offset = data & 0xFFF;
                match reloc_type {
                    ImageRel::BasedAbsolute => (),
                    ImageRel::BasedDir64 => {
                        let rva = reloc_base.add(reloc_offset as _).cast::<u64>();
                        *rva = driver_base.offset(
                            *rva as isize - (*driver_nt_headers).OptionalHeader.ImageBase as isize,
                        ) as u64;
                    }
                    ImageRel::Unknown => panic!("Unsupported relocation type"),
                }
            }
            current_size += (*reloc).SizeOfBlock;
            reloc = reloc_address.add(reloc_length) as _;
        }
    }

    log::info!("[*] Copying restore data to driver export: \"{DRIVER_EXPORT_NAME}\"");
    let driver_data = common::get_export(
        driver_base,
        CStr::from_ptr(DRIVER_EXPORT_NAME.as_ptr() as _),
    )
    .expect("Unable to find target driver export");
    ptr::copy_nonoverlapping(target_function, driver_data as _, DRIVER_EXPORT_SIZE);

    driver_base.add((*driver_nt_headers).OptionalHeader.AddressOfEntryPoint as _)
}
