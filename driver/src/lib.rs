#![no_std]

mod include;

#[allow(unused_imports)]
use core::panic::PanicInfo;

extern crate alloc;
use core::ptr;
use include::{
    ntddk::{IoAllocateMdl, MmMapLockedPagesSpecifyCache, MmProbeAndLockPages, MmUnmapLockedPages},
    types::{LOCK_OPERATION, MEMORY_CACHING_TYPE, MM_PAGE_PRIORITY},
};
use kernel_log::KernelLogger;
use log::LevelFilter;
use winapi::{
    ctypes::c_void,
    km::wdm::{DRIVER_OBJECT, KPROCESSOR_MODE},
    shared::ntdef::{NTSTATUS, UNICODE_STRING},
};

use crate::include::ntddk::{IoFreeMdl, MmUnlockPages};

const JMP_SIZE: usize = 14;
const LEA_SIZE: usize = 7;
const RESTORE_DATA_SIZE: usize = JMP_SIZE + LEA_SIZE;

#[no_mangle]
#[export_name = "RestoreData"]
pub static restore_data: [u8; RESTORE_DATA_SIZE] = [0; RESTORE_DATA_SIZE];

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
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

type DriverEntry =
    fn(driver_object: &mut DRIVER_OBJECT, registry_path: &UNICODE_STRING) -> NTSTATUS;

#[no_mangle]
pub extern "system" fn driver_entry(
    driver_object: &mut DRIVER_OBJECT,
    registry_path: &UNICODE_STRING,
    target_entry: *mut c_void,
) -> NTSTATUS {
    KernelLogger::init(LevelFilter::Info).expect("Failed to initialize logger");
    log::info!("[+] Driver entry called! Welcome back");
    log::info!("[*] Restoring original entry point");
    unsafe {
        copy_data(&restore_data, target_entry);
    }

    log::info!("[*] Executing unhooked DriverEntry of target driver");
    let original_driver_entry = unsafe {
        core::mem::transmute::<*mut c_void, DriverEntry>(target_entry)
    };
    original_driver_entry(driver_object, registry_path)
}

unsafe fn copy_data(src: &[u8], dst: *mut c_void) {
    let mdl = IoAllocateMdl(dst, src.len() as _, 0, 0, ptr::null_mut());
    if mdl.is_null() {
        log::error!("[!] IoAllocateMdl failed");
        panic!();
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
        log::error!("[!] MmMapLockedPagesSpecifyCache failed");
        MmUnlockPages(mdl);
        IoFreeMdl(mdl);
        panic!()
    }
    ptr::copy_nonoverlapping(src.as_ptr(), mapped as _, src.len());
    MmUnmapLockedPages(mapped, mdl);
    MmUnlockPages(mdl);
    IoFreeMdl(mdl);
}
