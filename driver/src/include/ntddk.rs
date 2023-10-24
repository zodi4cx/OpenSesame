use super::types::*;
use winapi::{
    km::{
        ndis::PMDL,
        wdm::{KPROCESSOR_MODE, PIRP},
    },
    shared::ntdef::{BOOLEAN, NTSTATUS, PVOID, ULONG},
};

#[link(name = "ntoskrnl")]
extern "system" {
    pub fn IoAllocateMdl(
        VirtualAddress: PVOID,
        Length: ULONG,
        SecondaryBuffer: BOOLEAN,
        ChargeQuota: BOOLEAN,
        Irp: PIRP,
    ) -> PMDL;

    pub fn MmProbeAndLockPages(
        MemoryDescriptorList: PMDL,
        AccessMode: KPROCESSOR_MODE,
        Operation: LOCK_OPERATION,
    );

    pub fn MmMapLockedPagesSpecifyCache(
        MemoryDescriptorList: PMDL,
        AccessMode: KPROCESSOR_MODE,
        CacheType: MEMORY_CACHING_TYPE,
        RequestedAddress: PVOID,
        BugCheckOnFailure: ULONG,
        Priority: MM_PAGE_PRIORITY,
    ) -> PVOID;

    pub fn MmUnlockPages(MemoryDescriptorList: PMDL);

    pub fn IoFreeMdl(Mdl: PMDL);

    pub fn MmUnmapLockedPages(BaseAddress: PVOID, MemoryDescriptorList: PMDL);

    #[must_use]
    pub fn PsSetLoadImageNotifyRoutine(NotifyRoutine: LOAD_IMAGE_NOTIFY_ROUTINE) -> NTSTATUS;
}
