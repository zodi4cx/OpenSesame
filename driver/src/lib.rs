#![no_std]
#![feature(panic_info_message)]

mod include;

#[allow(unused_imports)]
use core::panic::PanicInfo;

extern crate alloc;
use core::ptr;
use include::{
    ntddk::{IoAllocateMdl, MmMapLockedPagesSpecifyCache, MmProbeAndLockPages, MmUnmapLockedPages},
    types::{IMAGE_INFO, LOCK_OPERATION, MEMORY_CACHING_TYPE, MM_PAGE_PRIORITY, UNICODE_STRING},
};
use kernel_log::KernelLogger;
use log::LevelFilter;
use winapi::{
    ctypes::c_void,
    km::wdm::{DRIVER_OBJECT, KPROCESSOR_MODE},
    shared::{ntdef::{HANDLE, NTSTATUS}, ntstatus::STATUS_SUCCESS},
};

use crate::include::{ntddk::{IoFreeMdl, MmUnlockPages, PsSetLoadImageNotifyRoutine}, types::LOAD_IMAGE_NOTIFY_ROUTINE};

const JMP_SIZE: usize = 14;
const LEA_SIZE: usize = 7;
const RESTORE_DATA_SIZE: usize = JMP_SIZE + LEA_SIZE;

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
    loop {}
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
    if PsSetLoadImageNotifyRoutine(load_image_callback as LOAD_IMAGE_NOTIFY_ROUTINE) == STATUS_SUCCESS {
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
        LOCK_OPERATION::IoModifyAccess,
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
    _image_info: *mut IMAGE_INFO,
) {
    log::info!(
        "[D] Module '{}' loaded for process {:?}",
        (*full_image_name).as_str().unwrap(),
        process_id
    );
}
