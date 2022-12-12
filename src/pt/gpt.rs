use std::fmt::Display;

use super::{
    mbr::PartitionType as MBRPartitionType,
    raw::{RawGPTHeader, RawGPTPartitionEntry},
};
use crate::image::Image;
use crate::pt::mbr::MBR;
use uuid::{uuid, Uuid};

const BLOCK_SIZE: usize = 512;

fn compute_crc32(data: &[u8]) -> u32 {
    let mut crc = crc_any::CRC::crc32();
    crc.digest(data);

    crc.get_crc() as u32
}

#[derive(Clone, Debug)]
pub struct Partition {
    partition_type: PartitionType,
}

impl Partition {
    pub fn new() -> Self {
        Partition {
            partition_type: PartitionType::Unused,
        }
    }
}

#[derive(Debug)]
pub struct GPT {
    partitions: Vec<Partition>,
    disk_guid: Uuid,
}

#[derive(Clone, Debug)]
pub enum PartitionType {
    Unused,
    MBR,
    EFISystem,
    BIOSBoot,
    LinuxFilesystem,
}

impl From<PartitionType> for Uuid {
    fn from(t: PartitionType) -> Self {
        match t {
            PartitionType::Unused => uuid! {"00000000-0000-0000-0000-000000000000"},
            PartitionType::MBR => uuid! {"024DEE41-33E7-11D3-9D69-0008C781F39F"},
            PartitionType::EFISystem => uuid! {"C12A7328-F81F-11D2-BA4B-00A0C93EC93B"},
            PartitionType::BIOSBoot => uuid! {"21686148-6449-6E6F-744E-656564454649"},
            PartitionType::LinuxFilesystem => uuid! {"0FC63DAF-8483-4772-8E79-3D69D8477DE4"},
        }
    }
}

impl GPT {
    pub fn new() -> Self {
        GPT {
            partitions: vec![Partition::new(); 128],
            disk_guid: Uuid::new_v4(),
        }
    }

    pub fn add_partition(&mut self, t: PartitionType) -> () {
        self.partitions.push(Partition { partition_type: t })
    }

    fn write_protective_mbr(&self, image: &mut Image) {
        let mbr = MBR::new_protective(image.len() / super::mbr::MBR_SECTOR_SIZE);
        mbr.write(image);
    }

    fn write_entries(&self, image: &mut Image, entries_start_idx: usize) -> u32 {
        let nr_entries = self.partitions.len();
        let entries_size = nr_entries * std::mem::size_of::<RawGPTPartitionEntry>();
        let nr_entry_blocks = (entries_size + (BLOCK_SIZE - 1)) / BLOCK_SIZE;

        let blocks = image.get_blocks_mut(entries_start_idx, nr_entry_blocks);
        blocks.fill(0);

        // TODO: Restrict to just the entries?
        compute_crc32(blocks)
    }

    fn write_table(
        &self,
        image: &mut Image,
        this_block_idx: usize,
        alternative_block_idx: usize,
        entries_start_idx: usize,
        valid_range: (usize, usize),
    ) {
        let entries_checksum = self.write_entries(image, entries_start_idx);

        let mut hdr = RawGPTHeader::new();

        hdr.this_header_lba = this_block_idx as u64;
        hdr.other_header_lba = alternative_block_idx as u64;
        hdr.first_usable_lba = valid_range.0 as u64;
        hdr.last_usable_lba = valid_range.1 as u64;
        hdr.disk_guid = self.disk_guid.to_bytes_le();
        hdr.partition_entries_lba = entries_start_idx as u64;
        hdr.nr_partition_entries = self.partitions.len() as u32;
        hdr.partition_entries_checksum = entries_checksum;
        hdr.header_checksum = hdr.compute_checksum();

        image.write(this_block_idx * BLOCK_SIZE, hdr);
    }

    pub fn write(&self, image: &mut Image) {
        self.write_protective_mbr(image);

        let nr_blocks = image.len() / BLOCK_SIZE;

        let primary_header_block = 1;
        let alt_header_block = nr_blocks - 1;

        let nr_entries = self.partitions.len();
        let entries_size = nr_entries * std::mem::size_of::<RawGPTPartitionEntry>();
        let nr_entry_blocks = (entries_size + (BLOCK_SIZE - 1)) / BLOCK_SIZE;

        let valid_range = (
            primary_header_block + nr_entry_blocks + 1,
            alt_header_block - nr_entry_blocks - 1,
        );

        self.write_table(
            image,
            primary_header_block,
            alt_header_block,
            primary_header_block + 1,
            valid_range,
        );

        self.write_table(
            image,
            alt_header_block,
            primary_header_block,
            alt_header_block - nr_entry_blocks,
            valid_range,
        );
    }

    pub fn read(image: &Image) -> Option<GPT> {
        let mbr = MBR::read(image).unwrap();

        match mbr.partition_table[0].ptype {
            MBRPartitionType::ProtectiveMBR => {
                let gpt = image.read::<RawGPTHeader>(BLOCK_SIZE);
                if gpt.signature == [0x45, 0x46, 0x49, 0x20, 0x50, 0x41, 0x52, 0x54] {
                    Some(GPT {
                        partitions: vec![],
                        disk_guid: Uuid::from_bytes(gpt.disk_guid),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Display for GPT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("GUID: {}", self.disk_guid))
    }
}
