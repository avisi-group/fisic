use super::raw::{RawMBR, RawMBRPartitionEntry};
use crate::image::Image;

pub const MBR_SECTOR_SIZE: usize = 512;

pub enum MBRError {
    MBRFormatError,
}

#[inline]
fn u32_to_le(v: u32) -> [u8; 4] {
    [v as u8, (v >> 8) as u8, (v >> 16) as u8, (v >> 24) as u8]
}

pub enum EntryStatus {
    Bootable,
    NotBootable,
}

pub enum PartitionType {
    Empty,
    ProtectiveMBR,
    Unknown,
}

pub struct CHS {
    head: usize,
    sector: usize,
    cylinder: usize,
}

const HEADS_PER_CYLINDER: usize = 16;
const SECTORS_PER_TRACK: usize = 63;

impl CHS {
    pub fn new(head: usize, sector: usize, cylinder: usize) -> Self {
        CHS {
            head,
            sector,
            cylinder,
        }
    }

    pub fn new_zero() -> Self {
        CHS {
            head: 0,
            sector: 0,
            cylinder: 0,
        }
    }

    pub fn new_max() -> Self {
        CHS {
            head: 0xff,
            sector: 0x3f,
            cylinder: 0x3ff,
        }
    }

    pub fn from_raw(bytes: &[u8; 3]) -> Self {
        CHS {
            head: 0,
            sector: 0,
            cylinder: 0,
        }
    }

    fn saturate<T>(v: T, max: T) -> T
    where
        T: PartialOrd,
    {
        if v > max {
            max
        } else {
            v
        }
    }

    pub fn from_lba(lba: usize) -> Self {
        let cylinder = Self::saturate(lba / (HEADS_PER_CYLINDER * SECTORS_PER_TRACK), 0x3ff);
        let head = Self::saturate((lba / SECTORS_PER_TRACK) % HEADS_PER_CYLINDER, 0xff);
        let sector = Self::saturate((lba % SECTORS_PER_TRACK) + 1, 0x3f);

        CHS {
            head,
            sector,
            cylinder,
        }
    }

    pub fn to_bytes(&self) -> [u8; 3] {
        [
            self.head as u8,
            ((self.sector & 0x3f) | ((self.cylinder & 0x300) >> 2)) as u8,
            self.cylinder as u8,
        ]
    }
}

pub struct PartitionEntry {
    pub status: EntryStatus,
    pub ptype: PartitionType,
    pub first_sector: CHS,
    pub last_sector: CHS,
    pub first_sector_lba: usize,
    pub nr_sectors: usize,
}

pub struct MBR {
    pub partition_table: [PartitionEntry; 4],
}

impl PartitionEntry {
    pub fn new_empty() -> Self {
        PartitionEntry {
            status: EntryStatus::NotBootable,
            ptype: PartitionType::Empty,
            first_sector: CHS::new_zero(),
            last_sector: CHS::new_zero(),
            first_sector_lba: 0,
            nr_sectors: 0,
        }
    }

    pub fn new(status: EntryStatus, ptype: PartitionType, first: usize, last: usize) -> Self {
        PartitionEntry {
            status,
            ptype,
            first_sector: CHS::from_lba(first),
            last_sector: CHS::from_lba(last),
            first_sector_lba: first,
            nr_sectors: last - first,
        }
    }

    pub fn to_raw(&self) -> RawMBRPartitionEntry {
        RawMBRPartitionEntry {
            status: match self.status {
                EntryStatus::Bootable => 0x80,
                EntryStatus::NotBootable => 0x00,
            },
            first_sector_chs: self.first_sector.to_bytes(),
            ptype: match self.ptype {
                PartitionType::Empty => 0,
                PartitionType::ProtectiveMBR => 0xee,
                _ => 0xff,
            },
            last_sector_chs: self.last_sector.to_bytes(),
            first_sector_lba: self.first_sector_lba as u32,
            nr_sectors: self.nr_sectors as u32,
        }
    }

    pub fn from_raw(raw: RawMBRPartitionEntry) -> PartitionEntry {
        PartitionEntry {
            status: if (raw.status & 0x80) == 0x80 {
                EntryStatus::Bootable
            } else {
                EntryStatus::NotBootable
            },
            ptype: match raw.ptype {
                0 => PartitionType::Empty,
                0xee => PartitionType::ProtectiveMBR,
                _ => PartitionType::Unknown,
            },
            first_sector: CHS::from_raw(&raw.first_sector_chs),
            last_sector: CHS::from_raw(&raw.last_sector_chs),
            first_sector_lba: raw.first_sector_lba as usize,
            nr_sectors: raw.nr_sectors as usize,
        }
    }
}

impl MBR {
    pub fn new() -> Self {
        MBR {
            partition_table: [
                PartitionEntry::new_empty(),
                PartitionEntry::new_empty(),
                PartitionEntry::new_empty(),
                PartitionEntry::new_empty(),
            ],
        }
    }

    pub fn new_protective(nr_blocks: usize) -> Self {
        let mut pe = PartitionEntry::new(
            EntryStatus::NotBootable,
            PartitionType::ProtectiveMBR,
            1,
            nr_blocks,
        );

        pe.last_sector = CHS::new_max();

        MBR {
            partition_table: [
                pe,
                PartitionEntry::new_empty(),
                PartitionEntry::new_empty(),
                PartitionEntry::new_empty(),
            ],
        }
    }

    pub fn set_entry(&mut self, index: usize, e: PartitionEntry) -> () {
        self.partition_table[index] = e;
    }

    pub fn to_raw(&self) -> RawMBR {
        let mut mbr = RawMBR::new();

        for i in 0..4 {
            mbr.partition_entries[i] = self.partition_table[i].to_raw();
        }

        mbr
    }

    pub fn write(&self, image: &mut Image) {
        let mbr = self.to_raw();
        let block0 = image.get_blocks_mut(0, 1);

        block0.copy_from_slice(mbr.as_bytes());
    }

    pub fn read(image: &Image) -> Option<Self> {
        let block0 = image.get_blocks(0, 1);
        let raw = RawMBR::from_bytes(block0);

        if raw.signature != [0x55, 0xaa] {
            return None;
        }

        Some(MBR {
            partition_table: [
                PartitionEntry::from_raw(raw.partition_entries[0]),
                PartitionEntry::from_raw(raw.partition_entries[1]),
                PartitionEntry::from_raw(raw.partition_entries[2]),
                PartitionEntry::from_raw(raw.partition_entries[3]),
            ],
        })
    }

    pub fn check(image: &Image) -> bool {
        let block0 = image.get_blocks(0, 1);
        let raw = RawMBR::from_bytes(block0);

        raw.signature == [0x55, 0xaa]
    }
}
