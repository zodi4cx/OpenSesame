#![no_main]
#![no_std]

extern crate alloc;

mod boot;
mod hook;
mod utils;

use alloc::slice;
use core::ffi::c_void;
use core::u8;

use hook::{Hook, ImgArchStartBootApplication};
use uefi::prelude::*;
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::LoadImageSource;

const IMG_ARCH_START_BOOT_APPLICATION_SIGNATURE: &str = "48 8B C4 48 89 58 20 44 89 40 18 48 89 50 10 48 89 48 08 55 56 57 41 54 41 55 41 56 41 57 48 8D 68 A9";

static mut IMG_ARCH_START_BOOT_APPLICATION: Option<Hook<ImgArchStartBootApplication>> = None;

#[entry]
fn efi_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    let boot_services = system_table.boot_services();
    log::info!("[*] Searching Windows EFI bootmgr");
    let bootmgr_device_path = boot::windows_bootmgr_device_path(boot_services)
        .expect("Failed to find Windows Boot Manager. Is Windows installed?");
    log::info!("[+] Found! Loading Boot Manager into memory");
    let bootmgr_handle = boot_services
        .load_image(
            image_handle,
            LoadImageSource::FromDevicePath {
                device_path: &bootmgr_device_path,
                from_boot_manager: false,
            },
        )
        .unwrap();
    setup_hooks(&bootmgr_handle, boot_services);
    log::info!("[+] Starting Windows Boot Manager");
    system_table.boot_services().stall(2_000_000);
    boot_services
        .start_image(bootmgr_handle)
        .expect("Failed to start Windows Boot Manager");
    Status::SUCCESS
}

fn setup_hooks(bootmgr_handle: &Handle, boot_services: &BootServices) {
    let bootmgr_image = boot_services
        .open_protocol_exclusive::<LoadedImage>(*bootmgr_handle)
        .unwrap();
    let (image_base, image_size) = bootmgr_image.info();
    let bootmgr_data = unsafe { slice::from_raw_parts(image_base as *const _, image_size as _) };
    let offset = utils::find_pattern(IMG_ARCH_START_BOOT_APPLICATION_SIGNATURE, bootmgr_data)
        .expect("Unable to match ImgArchStartBootApplication signature");
    unsafe {
        IMG_ARCH_START_BOOT_APPLICATION = Some(Hook::new(
            image_base.add(offset) as *mut _,
            img_arch_start_boot_application_hook as *const _,
        ));
    };
}

fn img_arch_start_boot_application_hook(
    app_entry: *mut c_void,
    image_base: *mut c_void,
    image_size: u32,
    boot_option: u8,
    return_arguments: *mut c_void,
) {
    let img_arch_start_boot_application =
        unsafe { IMG_ARCH_START_BOOT_APPLICATION.take().unwrap().unhook() };
    img_arch_start_boot_application(
        app_entry,
        image_base,
        image_size,
        boot_option,
        return_arguments,
    )
}
