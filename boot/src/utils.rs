use alloc::vec::Vec;
use core::{ffi::c_void, mem};

pub const CALL_SIZE: usize = 5;

fn pattern_to_hex(pattern: &str) -> Vec<Option<u8>> {
    let mut result = Vec::new();
    pattern
        .split_ascii_whitespace()
        .for_each(|char| match char {
            "?" => result.push(None),
            other => result.push(Some(
                u8::from_str_radix(other, 16).expect("Invalid signature"),
            )),
        });
    result
}

pub fn find_pattern(pattern: &str, data: &[u8]) -> Option<usize> {
    let pattern = pattern_to_hex(pattern);
    data.windows(pattern.len()).position(|window| {
        window
            .iter()
            .zip(pattern.iter())
            .all(|(byte, pattern_byte)| pattern_byte.map_or(true, |b| *byte == b))
    })
}

pub unsafe fn relative_address(address: *const c_void, size: usize) -> *const c_void {
    assert!(size >= mem::size_of::<i32>());
    let mut buffer = [0_u8; mem::size_of::<i32>()];
    core::ptr::copy_nonoverlapping(
        address.add(size - mem::size_of::<i32>()) as *const _,
        buffer.as_mut_ptr(),
        mem::size_of::<i32>(),
    );
    address.add(size).offset(i32::from_le_bytes(buffer) as _)
}
