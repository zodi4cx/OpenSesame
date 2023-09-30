#![no_main]
#![no_std]

extern crate alloc;

mod hook;

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::slice;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::u8;

use hook::{Hook, ImgArchStartBootApplication};
use uefi::prelude::*;
use uefi::proto::device_path::{
    build::{media::FilePath, DevicePathBuilder},
    DevicePath,
};
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::media::file::{File, FileAttribute, FileMode};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::{HandleBuffer, LoadImageSource, SearchType};
use uefi::{CStr16, Identify};

const WINDOWS_BOOTMGR_PATH: &CStr16 = cstr16!("\\efi\\microsoft\\boot\\bootmgfw.efi");
const IMG_ARCH_START_BOOT_APPLICATION_SIGNATURE: &str = "48 8B C4 48 89 58 20 44 89 40 18 48 89 50 10 48 89 48 08 55 56 57 41 54 41 55 41 56 41 57 48 8D 68 A9";

static mut IMG_ARCH_START_BOOT_APPLICATION: Option<Hook<ImgArchStartBootApplication>> = None;

#[entry]
fn efi_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    let boot_services = system_table.boot_services();
    log::info!("[*] Searching Windows EFI bootmgr");
    let bootmgr_device_path = windows_bootmgr_device_path(boot_services)
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
    let offset = find_pattern(IMG_ARCH_START_BOOT_APPLICATION_SIGNATURE, bootmgr_data)
        .expect("Unable to match ImgArchStartBootApplication signature");
    unsafe {
        IMG_ARCH_START_BOOT_APPLICATION = Some(Hook::new(
            image_base.add(offset) as *mut _,
            img_arch_start_boot_application_hook as *const _,
        ));
    };
}

fn img_arch_start_boot_application_hook(
    _app_entry: *mut c_void,
    _image_base: *mut c_void,
    _image_size: u32,
    _boot_option: u8,
    _return_arguments: *mut c_void,
) {
    panic!("It worked!");
}

fn windows_bootmgr_device_path(boot_services: &BootServices) -> Option<Box<DevicePath>> {
    let handles: HandleBuffer = boot_services
        .locate_handle_buffer(SearchType::ByProtocol(&SimpleFileSystem::GUID))
        .unwrap();
    for handle in handles.iter() {
        if let Ok(mut file_system) =
            boot_services.open_protocol_exclusive::<SimpleFileSystem>(*handle)
        {
            if let Ok(mut root) = file_system.open_volume() {
                if root
                    .open(
                        WINDOWS_BOOTMGR_PATH,
                        FileMode::Read,
                        FileAttribute::READ_ONLY,
                    )
                    .is_ok()
                {
                    let device_path = boot_services
                        .open_protocol_exclusive::<DevicePath>(*handle)
                        .unwrap();
                    let mut storage = Vec::new();
                    let boot_path = device_path
                        .node_iter()
                        .fold(
                            DevicePathBuilder::with_vec(&mut storage),
                            |builder, item| builder.push(&item).unwrap(),
                        )
                        .push(&FilePath {
                            path_name: WINDOWS_BOOTMGR_PATH,
                        })
                        .unwrap()
                        .finalize()
                        .expect("Error creating bootmgfw.efi device path");
                    return Some(boot_path.to_owned());
                }
            }
        }
    }
    None
}

fn pattern_to_hex(pattern: &str) -> Vec<Option<u8>> {
    let mut result = Vec::new();
    pattern
        .split_ascii_whitespace()
        .for_each(|char| match char {
            "?" => result.push(None),
            other => result.push(Some(
                u8::from_str_radix(other, 16).expect("Invalid signature"),
            )),
        });
    result
}

fn find_pattern(pattern: &str, data: &[u8]) -> Option<usize> {
    let pattern = pattern_to_hex(pattern);
    data.windows(pattern.len()).position(|window| {
        window
            .iter()
            .zip(pattern.iter())
            .all(|(byte, pattern_byte)| pattern_byte.map_or(true, |b| *byte == b))
    })
}
