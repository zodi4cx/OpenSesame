#![no_main]
#![no_std]

extern crate alloc;

use log::info;
use uefi::prelude::*;
use uefi::proto::device_path::build::media::FilePath;
use uefi::proto::device_path::build::DevicePathBuilder;
use uefi::proto::device_path::text::AllowShortcuts;
use uefi::proto::device_path::text::DisplayOnly;
use uefi::proto::device_path::DevicePath;
use uefi::proto::device_path::DeviceSubType;
use uefi::proto::device_path::DeviceType;
use uefi::proto::device_path::LoadedImageDevicePath;
use uefi::proto::media::file::File;
use uefi::proto::media::file::FileAttribute;
use uefi::proto::media::file::FileMode;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::HandleBuffer;
use uefi::table::boot::SearchType;
use uefi::{CStr16, Identify};

use alloc::boxed::Box;
use alloc::vec::Vec;

const WINDOWS_BOOTMGR_PATH: &CStr16 = cstr16!("\\efi\\microsoft\\boot\\bootmgfw.efi");

#[entry]
fn efi_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    let boot_services = system_table.boot_services();
    info!("Searching Windows EFI bootmgr");
    windows_bootmgr_device_path(&boot_services);

    // Cleanup
    info!("Done!");
    system_table.boot_services().stall(10_000_000);
    Status::SUCCESS
}

fn windows_bootmgr_device_path(boot_services: &BootServices) -> Option<()> {
    let handles: HandleBuffer = boot_services
        .locate_handle_buffer(SearchType::ByProtocol(&SimpleFileSystem::GUID))
        .unwrap();
    for handle in handles.iter() {
        if let Ok(mut file_system) =
            boot_services.open_protocol_exclusive::<SimpleFileSystem>(*handle)
        {
            if let Ok(mut volume) = file_system.open_volume() {
                if let Ok(_) = volume.open(
                    &WINDOWS_BOOTMGR_PATH,
                    FileMode::Read,
                    FileAttribute::READ_ONLY,
                ) {
                    info!("Found Windows!");
                    let loaded_image_device_path = boot_services
                        .open_protocol_exclusive::<LoadedImageDevicePath>(
                            boot_services.image_handle(),
                        )
                        .unwrap();

                    let mut storage = Vec::new();
                    let mut builder = DevicePathBuilder::with_vec(&mut storage);

                    for node in loaded_image_device_path.node_iter() {
                        if node.full_type() == (DeviceType::MEDIA, DeviceSubType::MEDIA_FILE_PATH) {
                            break;
                        }

                        builder = builder.push(&node).unwrap();
                    }

                    builder = builder
                        .push(&FilePath {
                            path_name: WINDOWS_BOOTMGR_PATH,
                        })
                        .unwrap();

                    let new_image_path = builder.finalize().unwrap();
                    info!(
                        "{}",
                        new_image_path
                            .to_string(boot_services, DisplayOnly(false), AllowShortcuts(false))
                            .unwrap()
                            .unwrap()
                    );
                }
            }
        }
    }
    return None;
}
