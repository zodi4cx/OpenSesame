use alloc::string::{String, FromUtf16Error};
use winapi::shared::{
    basetsd::SIZE_T,
    ntdef::{HANDLE, PVOID, ULONG},
};

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LOCK_OPERATION {
    IoReadAccess = 0,
    IoWriteAccess = 1,
    IoModifyAccess = 2,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MEMORY_CACHING_TYPE {
    MmNonCached = 0,
    MmCached = 1,
    MmWriteCombined = 2,
    MmHardwareCoherentCached = 3,
    MmNonCachedUnordered = 4,
    MmUSWCCached = 5,
    MmMaximumCacheType = 6,
    MmNotMapped = -1,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MM_PAGE_PRIORITY {
    LowPagePriority = 0,
    NormalPagePriority = 16,
    HighPagePriority = 32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IMAGE_INFO {
    pub Properties: ULONG,
    pub ImageBase: PVOID,
    pub ImageSelector: ULONG,
    pub ImageSize: SIZE_T,
    pub ImageSectionNumber: ULONG,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct UNICODE_STRING {
    pub Length: u16,        // Length of the string
    pub MaximumLength: u16, // Maximum length of the string
    pub Buffer: *mut u16,   // Pointer to the string buffer
}

impl UNICODE_STRING {
    pub fn as_str(&self) -> Result<String, FromUtf16Error> {
        // Convert the UTF-16 buffer to a UTF-8 slice
        let utf16_slice =
            unsafe { core::slice::from_raw_parts(self.Buffer, self.Length as usize / 2) };
        // Convert UTF-16 to UTF-8
        String::from_utf16(utf16_slice)
    }
}

pub type LOAD_IMAGE_NOTIFY_ROUTINE = unsafe extern "C" fn(
    FullImageName: *const UNICODE_STRING,
    ProcessId: HANDLE,
    ImageInfo: *mut IMAGE_INFO,
);
