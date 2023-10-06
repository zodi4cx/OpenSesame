#![no_std]

#[allow(unused_imports)]
use core::panic::PanicInfo;

extern crate alloc;
use kernel_log::KernelLogger;
use log::LevelFilter;
use winapi::{
    km::wdm::DRIVER_OBJECT,
    shared::{
        ntdef::{NTSTATUS, UNICODE_STRING},
        ntstatus::STATUS_SUCCESS,
    },
};

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

#[no_mangle]
pub extern "system" fn driver_entry(
    _driver_object: &mut DRIVER_OBJECT,
    _registry_path: &UNICODE_STRING,
) -> NTSTATUS {
    KernelLogger::init(LevelFilter::Info).expect("Failed to initialize logger");
    log::info!("Hello, world!");
    return STATUS_SUCCESS;
}
