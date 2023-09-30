#![no_main]
#![no_std]

extern crate alloc;

mod hook;

use core::u8;

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::vec::Vec;

use uefi::prelude::*;
use uefi::proto::device_path::{
    build::{media::FilePath, DevicePathBuilder},
    DevicePath,
};
use uefi::proto::media::file::{File, FileAttribute, FileMode};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::{HandleBuffer, LoadImageSource, SearchType};
use uefi::{CStr16, Identify};

const WINDOWS_BOOTMGR_PATH: &CStr16 = cstr16!("\\efi\\microsoft\\boot\\bootmgfw.efi");

#[entry]
fn efi_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    let boot_services = system_table.boot_services();
    log::info!("[*] Searching Windows EFI bootmgr");
    let bootmgr_device_path = windows_bootmgr_device_path(boot_services)
        .expect("Failed to find Windows Boot Manager. Is Windows installed?");
    log::info!("[+] Found! Loading Boot Manager into memory");
    let bootmgr_image = boot_services
        .load_image(
            image_handle,
            LoadImageSource::FromDevicePath {
                device_path: &bootmgr_device_path,
                from_boot_manager: false,
            },
        )
        .unwrap();
    log::info!("[+] Starting Windows Boot Manager");
    system_table.boot_services().stall(2_000_000);
    boot_services
        .start_image(bootmgr_image)
        .expect("Failed to start Windows Boot Manager");
    Status::SUCCESS
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
        .into_iter()
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
