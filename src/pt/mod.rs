use crate::image::Image;

pub mod gpt;
pub mod mbr;
pub mod raw;

#[derive(Debug)]
pub enum PartitionTableType {
    MBR,
    GPT,
}

pub fn determine_pt_type(image: &Image) -> Option<PartitionTableType> {
    None
}
