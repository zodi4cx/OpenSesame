pub use crate::global::JMP_SIZE;
use crate::{global::LEA_SIZE, windows::LOADER_PARAMETER_BLOCK};
use core::{ffi::c_void, ptr};

pub type ImgArchStartBootApplication = fn(
    app_entry: *mut c_void,
    image_base: *mut c_void,
    image_size: u32,
    boot_option: u8,
    return_arguments: *mut c_void,
) -> uefi::Status;

pub type OslFwpKernelSetupPhase1 = fn(loader_block: *mut LOADER_PARAMETER_BLOCK) -> uefi::Status;

pub type BlImgAllocateBuffer = fn(
    image_buffer: *mut *mut c_void,
    image_size: u64,
    memory_type: u32,
    attributes: u32,
    reserved: *mut c_void,
    flags: u32,
) -> uefi::Status;

pub struct Hook<T> {
    original_func: *mut T,
    hooked_bytes: [u8; JMP_SIZE],
}

impl<T> Hook<T> {
    pub unsafe fn new(original_func: *mut T, hook_func: *const T) -> Hook<T> {
        let hooked_bytes = trampoline_hook(original_func, hook_func);
        Hook {
            original_func,
            hooked_bytes,
        }
    }

    pub unsafe fn unhook(&mut self) -> T {
        ptr::copy_nonoverlapping(
            self.hooked_bytes.as_ptr(),
            self.original_func as *mut _,
            JMP_SIZE,
        );
        core::mem::transmute_copy::<_, T>(&self.original_func)
    }

    pub unsafe fn hook(&mut self, hook_func: *const T) {
        self.hooked_bytes = trampoline_hook(self.original_func, hook_func);
    }
}

unsafe fn trampoline_hook<T>(src: *mut T, dst: *const T) -> [u8; JMP_SIZE] {
    let mut original = [0_u8; JMP_SIZE];
    let mut trampoline: [u8; JMP_SIZE] = [
        0xFF, 0x25, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    // Save original data
    ptr::copy_nonoverlapping(src as *const _, original.as_mut_ptr(), JMP_SIZE);
    // Complete trampoline jmp with destination address
    ptr::copy_nonoverlapping(
        &dst as *const _ as *const u8,
        trampoline.as_mut_ptr().offset(6),
        core::mem::size_of::<*const u8>(),
    );
    // Install hook
    ptr::copy_nonoverlapping(trampoline.as_ptr(), src as *mut _, JMP_SIZE);
    original
}

pub unsafe fn hook_driver(target_entry: *const c_void, original_entry: *mut c_void) {
    // lea r8, [rip - 7]
    let lea_instruction: [u8; LEA_SIZE] = [0x4C, 0x8D, 0x05, 0xF9, 0xFF, 0xFF, 0xFF];
    ptr::copy_nonoverlapping(
        lea_instruction.as_ptr() as _,
        original_entry,
        lea_instruction.len(),
    );
    trampoline_hook(original_entry.add(LEA_SIZE), target_entry);
}
