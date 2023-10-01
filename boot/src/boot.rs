extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::vec::Vec;

use uefi::proto::device_path::{
    build::{media::FilePath, DevicePathBuilder},
    DevicePath,
};
use uefi::proto::media::file::{File, FileAttribute, FileMode};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::{HandleBuffer, SearchType};
use uefi::{prelude::*, CStr16, Identify};

const WINDOWS_BOOTMGR_PATH: &CStr16 = cstr16!("\\efi\\microsoft\\boot\\bootmgfw.efi");

pub fn windows_bootmgr_device_path(boot_services: &BootServices) -> Option<Box<DevicePath>> {
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
