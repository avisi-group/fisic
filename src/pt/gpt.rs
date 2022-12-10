use std::mem::size_of;

use crate::image::Image;
use crate::pt::mbr::MBR;
use memmap::MmapMut;
use uuid::{uuid, Uuid};

use super::mbr::PartitionEntry;
use super::raw::{RawGPTHeader, RawGPTPartitionEntry};

const PARTITION_ENTRY_SIZE: usize = 128;
const GPT_HEADER_SIZE: usize = 0x5c;
const BLOCK_SIZE: usize = 512;

#[inline]
fn u32_to_le(v: u32) -> [u8; 4] {
    [v as u8, (v >> 8) as u8, (v >> 16) as u8, (v >> 24) as u8]
}

#[inline]
fn u64_to_le(v: u64) -> [u8; 8] {
    [
        v as u8,
        (v >> 8) as u8,
        (v >> 16) as u8,
        (v >> 24) as u8,
        (v >> 32) as u8,
        (v >> 40) as u8,
        (v >> 48) as u8,
        (v >> 56) as u8,
    ]
}

fn compute_crc32(data: &[u8]) -> u32 {
    let mut crc = crc_any::CRC::crc32();
    crc.digest(data);

    crc.get_crc() as u32
}

#[derive(Clone)]
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

pub struct GPT {
    partitions: Vec<Partition>,
    disk_guid: Uuid,
}

#[derive(Clone)]
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
        let mut mbr = MBR::new_protective(image.len() / super::mbr::MBR_SECTOR_SIZE);
        mbr.write(image);
    }

    /*
    fn write_header(&self, m: &mut MmapMut) {
        let nr_blocks = m.len() / BLOCK_SIZE;
        let lba1 = &mut m[BLOCK_SIZE..BLOCK_SIZE + BLOCK_SIZE];

        lba1.fill(0);

        // Magic Numbers
        lba1[0..8].copy_from_slice(&[0x45, 0x46, 0x49, 0x20, 0x50, 0x41, 0x52, 0x54]);

        // Revision
        lba1[8..12].copy_from_slice(&u32_to_le(0x010000));

        // Header Size
        lba1[12..16].copy_from_slice(&u32_to_le(GPT_HEADER_SIZE as u32));

        // Current LBA
        lba1[24..32].copy_from_slice(&u64_to_le(1));

        // Backup LBA
        lba1[32..40].copy_from_slice(&u64_to_le((nr_blocks - 1) as u64));

        let nr_blocks_for_pe =
            ((self.partitions.len() * PARTITION_ENTRY_SIZE) + (BLOCK_SIZE - 1)) / BLOCK_SIZE;

        // First usable LBA
        lba1[40..48].copy_from_slice(&u64_to_le(0x0800)); //3 + nr_blocks_for_pe as u64));

        // Last usable LBA
        lba1[48..56].copy_from_slice(&u64_to_le((nr_blocks - 1 - nr_blocks_for_pe) as u64));

        // Disk GUID
        lba1[56..72].copy_from_slice(&self.disk_guid.to_bytes_le());

        // Starting LBA of partition entries
        lba1[72..80].copy_from_slice(&u64_to_le(2));

        // Number of partition entries
        lba1[80..84].copy_from_slice(&u32_to_le(self.partitions.len() as u32));

        // Size of partition entry
        lba1[84..88].copy_from_slice(&u32_to_le(PARTITION_ENTRY_SIZE as u32));
    }

    pub fn write_entries(&self, m: &mut MmapMut) {}

    pub fn write_checksums(&self, m: &mut MmapMut) {
        let entries_start = BLOCK_SIZE * 2;
        let nr_blocks_for_pe =
            ((self.partitions.len() * PARTITION_ENTRY_SIZE) + (BLOCK_SIZE - 1)) / BLOCK_SIZE;
        let entries_end = entries_start + (nr_blocks_for_pe * BLOCK_SIZE);

        let entries = &m[entries_start..entries_end];
        let pt_checksum = compute_crc32(entries);

        let header_start = BLOCK_SIZE;
        let header_end = header_start + GPT_HEADER_SIZE;
        let header = &mut m[header_start..header_end];

        // Write the entries checksum first
        header[88..92].copy_from_slice(&u32_to_le(pt_checksum));

        // Compute and write the header checksum
        let header_checksum = compute_crc32(&header[..GPT_HEADER_SIZE]);
        header[16..20].copy_from_slice(&u32_to_le(header_checksum));
    }

    pub fn copy_backup_header(&self, m: &mut MmapMut) {
        let nr_blocks = m.len() / BLOCK_SIZE;
        let primary_header = &m[BLOCK_SIZE..BLOCK_SIZE + BLOCK_SIZE];
        let primary_header_data = primary_header.to_vec();

        let backup_header = &mut m[(BLOCK_SIZE * (nr_blocks - 1))..];

        backup_header.copy_from_slice(&primary_header_data);
    }*/

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
        hdr.header_checksum = compute_crc32(hdr.as_bytes());

        let block = image.get_blocks_mut(this_block_idx, 1);
        block.fill(0);

        block
            .split_at_mut(std::mem::size_of::<RawGPTHeader>())
            .0
            .copy_from_slice(hdr.as_bytes());
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

        /*self.write_header(m);
        self.write_entries(m);
        self.write_checksums(m);
        self.copy_backup_header(m);*/
    }
}
