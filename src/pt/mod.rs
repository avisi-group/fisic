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
    if mbr::MBR::check(image) {
        if gpt::GPT::check(image) {
            Some(PartitionTableType::GPT)
        } else {
            Some(PartitionTableType::MBR)
        }
    } else {
        None
    }
}
