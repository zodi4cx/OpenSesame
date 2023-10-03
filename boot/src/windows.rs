//! # Credit
//! This definitions are extracted from the [memN0ps/bootkit-rs](https://github.com/memN0ps/bootkit-rs/)
//! project. Additional credit to the [Vergilius Project](https://www.vergiliusproject.com/), for
//! providing the original C structures documentation.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::enum_variant_names)]

use core::ffi::c_void;

//0x10 bytes (sizeof)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LIST_ENTRY {
    pub Flink: *mut LIST_ENTRY, //0x0
    pub Blink: *mut LIST_ENTRY, //0x8
}

//0x48 bytes (sizeof)
#[repr(C)]
pub struct CONFIGURATION_COMPONENT_DATA {
    pub Parent: *mut CONFIGURATION_COMPONENT_DATA,  //0x0
    pub Child: *mut CONFIGURATION_COMPONENT_DATA,   //0x8
    pub Sibling: *mut CONFIGURATION_COMPONENT_DATA, //0x10
    pub ComponentEntry: CONFIGURATION_COMPONENT,    //0x18
    pub ConfigurationData: *mut u8,                 //0x40
}

//0x28 bytes (sizeof)
#[repr(C)]
pub struct CONFIGURATION_COMPONENT {
    Class: CONFIGURATION_CLASS,                // 0x0
    Type: CONFIGURATION_TYPE,                  // 0x4
    Flags: DEVICE_FLAGS,                       // 0x8
    Version: u16,                              // 0xc
    Revision: u16,                             // 0xe
    Key: u32,                                  // 0x10
    AffinityMask: CONFIGURATION_AFFINITY_MASK, // 0x14
    ConfigurationDataLength: u32,              // 0x18
    IdentifierLength: u32,                     // 0x1c
    Identifier: *const i8,                     // 0x20
}

//0x4 bytes (sizeof)
#[repr(C)]
union CONFIGURATION_AFFINITY_MASK {
    pub AffinityMask: u32,
    pub Group: u16,
    pub GroupIndex: u16,
}

//0x4 bytes (sizeof)
#[repr(u32)]
pub enum CONFIGURATION_CLASS {
    SystemClass = 0,
    ProcessorClass = 1,
    CacheClass = 2,
    AdapterClass = 3,
    ControllerClass = 4,
    PeripheralClass = 5,
    MemoryClass = 6,
    MaximumClass = 7,
}

//0x4 bytes (sizeof)
#[repr(u32)]
pub enum CONFIGURATION_TYPE {
    ArcSystem = 0,
    CentralProcessor = 1,
    FloatingPointProcessor = 2,
    PrimaryIcache = 3,
    PrimaryDcache = 4,
    SecondaryIcache = 5,
    SecondaryDcache = 6,
    SecondaryCache = 7,
    EisaAdapter = 8,
    TcAdapter = 9,
    ScsiAdapter = 10,
    DtiAdapter = 11,
    MultiFunctionAdapter = 12,
    DiskController = 13,
    TapeController = 14,
    CdromController = 15,
    WormController = 16,
    SerialController = 17,
    NetworkController = 18,
    DisplayController = 19,
    ParallelController = 20,
    PointerController = 21,
    KeyboardController = 22,
    AudioController = 23,
    OtherController = 24,
    DiskPeripheral = 25,
    FloppyDiskPeripheral = 26,
    TapePeripheral = 27,
    ModemPeripheral = 28,
    MonitorPeripheral = 29,
    PrinterPeripheral = 30,
    PointerPeripheral = 31,
    KeyboardPeripheral = 32,
    TerminalPeripheral = 33,
    OtherPeripheral = 34,
    LinePeripheral = 35,
    NetworkPeripheral = 36,
    SystemMemory = 37,
    DockingInformation = 38,
    RealModeIrqRoutingTable = 39,
    RealModePCIEnumeration = 40,
    MaximumType = 41,
}

#[repr(C)]
struct DEVICE_FLAGS {
    pub Flags: u32, // 0x0
}

impl DEVICE_FLAGS {
    const FAILED: u32 = 0x1;
    const READ_ONLY: u32 = 0x2;
    const REMOVABLE: u32 = 0x4;
    const CONSOLE_IN: u32 = 0x8;
    const CONSOLE_OUT: u32 = 0x10;
    const INPUT: u32 = 0x20;
    const OUTPUT: u32 = 0x40;

    pub fn is_failed(&self) -> bool {
        self.Flags & Self::FAILED != 0
    }

    pub fn is_read_only(&self) -> bool {
        self.Flags & Self::READ_ONLY != 0
    }

    pub fn is_removable(&self) -> bool {
        self.Flags & Self::REMOVABLE != 0
    }

    pub fn is_console_in(&self) -> bool {
        self.Flags & Self::CONSOLE_IN != 0
    }

    pub fn is_console_out(&self) -> bool {
        self.Flags & Self::CONSOLE_OUT != 0
    }

    pub fn is_input(&self) -> bool {
        self.Flags & Self::INPUT != 0
    }

    pub fn is_output(&self) -> bool {
        self.Flags & Self::OUTPUT != 0
    }
}

//0x18 bytes (sizeof)
#[repr(C)]
pub struct NLS_DATA_BLOCK {
    pub AnsiCodePageData: *mut u8,     // 0x0
    pub OemCodePageData: *mut u8,      // 0x8
    pub UnicodeCaseTableData: *mut u8, // 0x10
}

//0x10 bytes (sizeof)
#[repr(C)]
pub struct ARC_DISK_INFORMATION {
    pub DiskSignatures: LIST_ENTRY, // 0x0
}

// 0x14 bytes (sizeof)
#[repr(C)]
pub struct LOADER_BLOCK {
    pub I386: Option<I386_LOADER_BLOCK>, // x86 specific loader block
    pub Arm: Option<ARM_LOADER_BLOCK>,   // ARM specific loader block
}

// 0x10 bytes (sizeof)
#[repr(C)]
pub struct I386_LOADER_BLOCK {
    pub CommonDataArea: *mut c_void, // Pointer to common data area
    pub MachineType: u32,            // Machine type
    pub VirtualBias: u32,            // Virtual bias
}

// 0x4 bytes (sizeof)
#[repr(C)]
pub struct ARM_LOADER_BLOCK {
    pub PlaceHolder: u32, // Placeholder
}

// 0x40 bytes (sizeof)
#[repr(C)]
pub struct FIRMWARE_INFORMATION_LOADER_BLOCK {
    pub FirmwareTypeUefi: u32,                      // UEFI firmware type
    pub EfiRuntimeUseIum: u32,                      // EFI runtime use IUM
    pub EfiRuntimePageProtectionSupported: u32,     // EFI runtime page protection supported
    pub Reserved: u32,                              // Reserved
    pub u: FIRMWARE_INFORMATION_LOADER_BLOCK_Union, // Union for firmware information
}

#[repr(C)]
pub struct FIRMWARE_INFORMATION_LOADER_BLOCK_Union {
    pub EfiInformation: EFI_FIRMWARE_INFORMATION, // EFI firmware information
    pub PcatInformation: PCAT_FIRMWARE_INFORMATION, // PCAT firmware information
}

// 0x38 bytes (sizeof)
#[repr(C)]
pub struct EFI_FIRMWARE_INFORMATION {
    pub FirmwareVersion: u32, // Firmware version
    pub VirtualEfiRuntimeServices: *mut VIRTUAL_EFI_RUNTIME_SERVICES, // Pointer to virtual EFI runtime services
    pub SetVirtualAddressMapStatus: i32, // Status of virtual address map
    pub MissedMappingsCount: u32,        // Count of missed mappings
    pub FirmwareResourceList: LIST_ENTRY, // Firmware resource list
    pub EfiMemoryMap: *mut u8,           // Pointer to EFI memory map
    pub EfiMemoryMapSize: u32,           // Size of EFI memory map
    pub EfiMemoryMapDescriptorSize: u32, // Size of EFI memory map descriptor
}

//0x70 bytes (sizeof)
#[repr(C)]
pub struct VIRTUAL_EFI_RUNTIME_SERVICES {
    pub GetTime: u64,                   // 0x0
    pub SetTime: u64,                   // 0x8
    pub GetWakeupTime: u64,             // 0x10
    pub SetWakeupTime: u64,             // 0x18
    pub SetVirtualAddressMap: u64,      // 0x20
    pub ConvertPointer: u64,            // 0x28
    pub GetVariable: u64,               // 0x30
    pub GetNextVariableName: u64,       // 0x38
    pub SetVariable: u64,               // 0x40
    pub GetNextHighMonotonicCount: u64, // 0x48
    pub ResetSystem: u64,               // 0x50
    pub UpdateCapsule: u64,             // 0x58
    pub QueryCapsuleCapabilities: u64,  // 0x60
    pub QueryVariableInfo: u64,         // 0x68
}

//0x4 bytes (sizeof)
#[repr(C)]
pub struct PCAT_FIRMWARE_INFORMATION {
    pub PlaceHolder: u32, // 0x0
}

//0x10 bytes (sizeof)
#[repr(C)]
pub struct RTL_RB_TREE {
    pub Root: *mut RTL_BALANCED_NODE, // 0x0
    pub Encoded: u8,                  // 0x8 (1 bit)
    pub Min: *mut RTL_BALANCED_NODE,  // 0x8
}

//0x18 bytes (sizeof)
#[repr(C)]
pub struct RTL_BALANCED_NODE {
    pub Children: [*mut RTL_BALANCED_NODE; 2], // 0x0
    pub Left: *mut RTL_BALANCED_NODE,          // 0x0
    pub Right: *mut RTL_BALANCED_NODE,         // 0x8
    pub Red: u8,                               // 0x10 (1 bit)
    pub Balance: u8,                           // 0x10 (2 bits)
    pub ParentValue: u64,                      // 0x10
}

//TODO (too big and not required for now)
#[repr(C)]
pub struct LOADER_PARAMETER_EXTENSION;

//0x170 bytes (sizeof)
#[repr(C)]
pub struct LOADER_PARAMETER_BLOCK {
    pub OsMajorVersion: u32,                                    //0x00
    pub OsMinorVersion: u32,                                    //0x4
    pub Size: u32,                                              //0x8
    pub OsLoaderSecurityVersion: u32,                           //0xc
    pub LoadOrderListHead: LIST_ENTRY,                          //0x10
    pub MemoryDescriptorListHead: LIST_ENTRY,                   //0x20
    pub BootDriverListHead: LIST_ENTRY,                         //0x30
    pub EarlyLaunchListHead: LIST_ENTRY,                        //0x40
    pub CoreDriverListHead: LIST_ENTRY,                         //0x50
    pub CoreExtensionsDriverListHead: LIST_ENTRY,               //0x60
    pub TpmCoreDriverListHead: LIST_ENTRY,                      //0x70
    pub KernelStack: u64,                                       //0x80
    pub Prcb: u64,                                              //0x88
    pub Process: u64,                                           //0x90
    pub Thread: u64,                                            //0x98
    pub KernelStackSize: u32,                                   //0xa0
    pub RegistryLength: u32,                                    //0xa4
    pub RegistryBase: *mut u8,                                  //0xa8
    pub ConfigurationRoot: *mut CONFIGURATION_COMPONENT_DATA,   //0xb0
    pub ArcBootDeviceName: *const i8,                           //0xb8
    pub ArcHalDeviceName: *const i8,                            //0xc0
    pub NtBootPathName: *const i8,                              //0xc8
    pub NtHalPathName: *const i8,                               //0xd0
    pub LoadOptions: *const i8,                                 //0xd8
    pub NlsData: *mut NLS_DATA_BLOCK,                           //0xe0
    pub ArcDiskInformation: *mut ARC_DISK_INFORMATION,          //0xe8
    pub Extension: *mut LOADER_PARAMETER_EXTENSION,             //0xf0
    pub u: LOADER_BLOCK,                                        //0xf8
    pub FirmwareInformation: FIRMWARE_INFORMATION_LOADER_BLOCK, //0x108
    pub OsBootstatPathName: *const i8,                          //0x148
    pub ArcOSDataDeviceName: *const i8,                         //0x150
    pub ArcWindowsSysPartName: *const i8,                       //0x158
    pub MemoryDescriptorTree: RTL_RB_TREE,                      //0x160
}

// 0xa0 bytes (sizeof)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct KLDR_DATA_TABLE_ENTRY {
    pub InLoadOrderLinks: LIST_ENTRY,                 // 0x0
    pub ExceptionTable: *const c_void,                // 0x10
    pub ExceptionTableSize: u32,                      // 0x18
    pub GpValue: *const c_void,                       // 0x20
    pub NonPagedDebugInfo: *mut NON_PAGED_DEBUG_INFO, // 0x28
    pub DllBase: *const c_void,                       // 0x30
    pub EntryPoint: *const c_void,                    // 0x38
    pub SizeOfImage: u32,                             // 0x40
    pub FullDllName: UNICODE_STRING,                  // 0x48
    pub BaseDllName: UNICODE_STRING,                  // 0x58
    pub Flags: u32,                                   // 0x68
    pub LoadCount: u16,                               // 0x6c
    pub SignatureLevel: u16,                          // 0x6e
    pub SectionPointer: *const c_void,                // 0x70
    pub CheckSum: u32,                                // 0x78
    pub CoverageSectionSize: u32,                     // 0x7c
    pub CoverageSection: *const c_void,               // 0x80
    pub LoadedImports: *const c_void,                 // 0x88
    pub Spare: *const c_void,                         // 0x90
    pub SizeOfImageNotRounded: u32,                   // 0x98
    pub TimeDateStamp: u32,                           // 0x9c
}

// Unicode string structure
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UNICODE_STRING {
    pub Length: u16,        // Length of the string
    pub MaximumLength: u16, // Maximum length of the string
    pub Buffer: *mut u16,   // Pointer to the string buffer
}

//0x20 bytes (sizeof)
#[repr(C)]
pub struct NON_PAGED_DEBUG_INFO {
    pub Signature: u16,       // 0x0
    pub Flags: u16,           // 0x2
    pub Size: u32,            // 0x4
    pub Machine: u16,         // 0x8
    pub Characteristics: u16, // 0xa
    pub TimeDateStamp: u32,   // 0xc
    pub CheckSum: u32,        // 0x10
    pub SizeOfImage: u32,     // 0x14
    pub ImageBase: u64,       // 0x18
}
