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
