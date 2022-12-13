pub const GPT_PTYPE_EMPTY: &str = "00000000-0000-0000-0000-000000000000";
pub const GPT_PTYPE_MBR: &str = "024DEE41-33E7-11D3-9D69-0008C781F39F";
pub const GPT_PTYPE_EFI_SYSTEM: &str = "C12A7328-F81F-11D2-BA4B-00A0C93EC93B";
pub const GPT_PTYPE_BIOS_BOOT: &str = "21686148-6449-6E6F-744E-656564454649";
pub const GPT_PTYPE_LINUX_FS: &str = "0FC63DAF-8483-4772-8E79-3D69D8477DE4";

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct RawGPTPartitionEntry {
    pub ptype: [u8; 16],
    pub ident: [u8; 16],
    pub starting_lba: u64,
    pub ending_lba: u64,
    pub attributes: u64,
    pub name: [u8; 72],
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct RawGPTHeader {
    pub signature: [u8; 8],
    pub revision: u32,
    pub header_size: u32,
    pub header_checksum: u32,
    pub reserved: u32,
    pub this_header_lba: u64,
    pub other_header_lba: u64,
    pub first_usable_lba: u64,
    pub last_usable_lba: u64,
    pub disk_guid: [u8; 16],
    pub partition_entries_lba: u64,
    pub nr_partition_entries: u32,
    pub partition_entry_size: u32,
    pub partition_entries_checksum: u32,
}

impl RawGPTHeader {
    pub fn new() -> Self {
        RawGPTHeader {
            signature: [0x45, 0x46, 0x49, 0x20, 0x50, 0x41, 0x52, 0x54],
            revision: 0x00010000,
            header_size: std::mem::size_of::<Self>() as u32,
            header_checksum: 0,
            reserved: 0,
            this_header_lba: 0,
            other_header_lba: 0,
            first_usable_lba: 0,
            last_usable_lba: 0,
            disk_guid: [0; 16],
            partition_entries_lba: 0,
            nr_partition_entries: 0,
            partition_entry_size: std::mem::size_of::<RawGPTPartitionEntry>() as u32,
            partition_entries_checksum: 0,
        }
    }

    pub fn compute_checksum(&self) -> u32 {
        let mut crc = crc_any::CRC::crc32();

        let data = unsafe {
            ::std::slice::from_raw_parts(
                (self as *const _) as *const u8,
                ::std::mem::size_of::<Self>(),
            )
        };

        crc.digest(data);

        crc.get_crc() as u32
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct RawMBRPartitionEntry {
    pub status: u8,
    pub first_sector_chs: [u8; 3],
    pub ptype: u8,
    pub last_sector_chs: [u8; 3],
    pub first_sector_lba: u32,
    pub nr_sectors: u32,
}

impl RawMBRPartitionEntry {
    pub fn new() -> Self {
        RawMBRPartitionEntry {
            status: 0,
            first_sector_chs: [0, 0, 0],
            ptype: 0,
            last_sector_chs: [0, 0, 0],
            first_sector_lba: 0,
            nr_sectors: 0,
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct RawMBR {
    pub bootstrap: [u8; 0x01be],
    pub partition_entries: [RawMBRPartitionEntry; 4],
    pub signature: [u8; 2],
}

impl RawMBR {
    pub fn new() -> Self {
        RawMBR {
            bootstrap: [0; 0x01be],
            partition_entries: [RawMBRPartitionEntry::new(); 4],
            signature: [0x55, 0xaa],
        }
    }
}
