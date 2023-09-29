#![no_main]
#![no_std]

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use uefi::prelude::*;
use uefi::proto::device_path::build::media::FilePath;
use uefi::proto::device_path::build::DevicePathBuilder;
use uefi::proto::device_path::text::AllowShortcuts;
use uefi::proto::device_path::text::DisplayOnly;
use uefi::proto::device_path::DevicePath;
use uefi::proto::media::file::File;
use uefi::proto::media::file::FileAttribute;
use uefi::proto::media::file::FileMode;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::HandleBuffer;
use uefi::table::boot::LoadImageSource;
use uefi::table::boot::OpenProtocolAttributes;
use uefi::table::boot::OpenProtocolParams;
use uefi::table::boot::SearchType;
use uefi::{CStr16, Identify};

use alloc::vec::Vec;

const WINDOWS_BOOTMGR_PATH: &CStr16 = cstr16!("\\efi\\microsoft\\boot\\bootmgfw.efi");

#[entry]
fn efi_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    let boot_services = system_table.boot_services();
    log::info!("Searching Windows EFI bootmgr");
    if let Some(bootmgr_device_path) = windows_bootmgr_device_path(boot_services) {
        log::info!(
            "Windows bootmgfw.efi path: {}",
            bootmgr_device_path
                .to_string(boot_services, DisplayOnly(false), AllowShortcuts(false))
                .unwrap()
                .unwrap()
        );
        let bootmgr_image = boot_services
            .load_image(
                image_handle,
                LoadImageSource::FromDevicePath {
                    device_path: &bootmgr_device_path,
                    from_boot_manager: false,
                },
            )
            .unwrap();
        log::info!("[+] Starting Windows EFI Boot Manager (bootmgfw.efi)");
        system_table.boot_services().stall(5_000_000);
        boot_services
            .start_image(bootmgr_image)
            .expect("[-] Failed to start Windows EFI Boot Manager");
    }
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
            if let Ok(mut volume) = file_system.open_volume() {
                if volume
                    .open(
                        WINDOWS_BOOTMGR_PATH,
                        FileMode::Read,
                        FileAttribute::READ_ONLY,
                    )
                    .is_ok()
                {
                    let device_path = unsafe {
                        boot_services.open_protocol::<DevicePath>(
                            OpenProtocolParams {
                                handle: *handle,
                                agent: boot_services.image_handle(),
                                controller: None,
                            },
                            OpenProtocolAttributes::Exclusive,
                        )
                    };
                    let device_path = device_path.unwrap();
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
                        .unwrap();
                    return Some(boot_path.to_owned());
                }
            }
        }
    }
    None
}
