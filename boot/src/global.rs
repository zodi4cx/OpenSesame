use crate::hook::{
    BlImgAllocateBuffer, Hook, ImgArchStartBootApplication, OslFwpKernelSetupPhase1,
};
use core::ffi::c_void;

pub const IMG_ARCH_START_BOOT_APPLICATION_SIGNATURE: &str = "48 8B C4 48 89 58 20 44 89 40 18 48 89 50 10 48 89 48 08 55 56 57 41 54 41 55 41 56 41 57 48 8D 68 A9";
pub const OSL_EXECUTE_TRANSITION_SIGNATURE: &str = "E8 ? ? ? ? 8B D8 E8 ? ? ? ? 48 83 3D ? ? ? ? ?";
pub const OSL_FWP_KERNEL_SETUP_PHASE1_SIGNATURE: &str = "E8 ? ? ? ? 8B F0 85 C0 79 ?";
pub const BL_IMG_ALLOCATE_BUFFER_SIGNATURE: &str = "48 8B D6 E8 ? ? ? ? 48 8B 7C 24 ?";

pub static mut IMG_ARCH_START_BOOT_APPLICATION: Option<Hook<ImgArchStartBootApplication>> = None;
pub static mut OSL_FWP_KERNEL_SETUP_PHASE1: Option<Hook<OslFwpKernelSetupPhase1>> = None;
pub static mut BL_IMG_ALLOCATE_BUFFER: Option<Hook<BlImgAllocateBuffer>> = None;

pub const BL_MEMORY_TYPE_APPLICATION: u32 = 0xE0000012;
pub const BL_MEMORY_ATTRIBUTE_RWX: u32 = 0x424000;
pub static mut DRIVER_ALLOCATED_BUFFER: *mut c_void = core::ptr::null_mut();
pub static mut DRIVER_SIZE: u64 = 0x100;
