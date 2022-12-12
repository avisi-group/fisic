use crate::image::Image;

pub mod gpt;
pub mod mbr;
pub mod raw;

#[derive(Debug)]
pub enum PartitionTableType {
    MBR,
    GPT,
}

#[derive(Debug)]
pub enum PartitionTable {
    MBR(mbr::MBR),
    GPT(gpt::GPT),
}

pub fn read_partition_table(image: &Image) -> Option<PartitionTable> {
    let mbr = mbr::MBR::read(image);

    match mbr {
        Some(mbr) => {
            let gpt = gpt::GPT::read(image);

            match gpt {
                Some(gpt) => Some(PartitionTable::GPT(gpt)),
                None => Some(PartitionTable::MBR(mbr)),
            }
        }
        None => None,
    }
}
