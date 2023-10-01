use core::{ffi::c_void, ptr};

pub const JMP_SIZE: usize = 14;

pub type ImgArchStartBootApplication = fn(
    app_entry: *mut c_void,
    image_base: *mut c_void,
    image_size: u32,
    boot_option: u8,
    return_arguments: *mut c_void,
);

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
