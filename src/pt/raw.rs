#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct RawGPTPartitionEntry {
    ptype: [u8; 16],
    ident: [u8; 16],
    starting_lba: u64,
    ending_lba: u64,
    attributes: u64,
    name: [u8; 72],
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
