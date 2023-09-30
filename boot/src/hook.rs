use core::{ffi::c_void, ptr};

pub const JMP_SIZE: usize = 14;

pub type ImgArchStartBootApplication = fn(
    app_entry: *mut c_void,
    image_base: *mut c_void,
    image_size: u32,
    boot_option: u8,
    return_arguments: *mut c_void,
);

pub unsafe fn trampoline_hook<T>(src: *mut T, dst: *const T) -> [u8; JMP_SIZE] {
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

pub unsafe fn trampoline_unhook<T>(src: *mut T, original: [u8; JMP_SIZE]) {
    ptr::copy_nonoverlapping(original.as_ptr(), src as *mut _, JMP_SIZE);
}