use crate::{
    image::Image,
    pt::{gpt::GPT, mbr::MBR, PartitionTableType},
};

use super::Action;

pub struct InitActionArgs {
    pub pt_type: PartitionTableType,
}

#[derive(Debug, displaydoc::Display, thiserror::Error)]
pub enum InitActionError {
    /// Generic Error
    GenericError,
}

pub struct InitAction {}

impl Action<InitActionArgs, InitActionError> for InitAction {
    fn invoke(image: &mut Image, args: InitActionArgs) -> Result<(), InitActionError> {
        match args.pt_type {
            PartitionTableType::MBR => {
                let mbr = MBR::new();
                mbr.write(image);
                Ok(())
            }
            PartitionTableType::GPT => {
                let gpt = GPT::new();
                gpt.write(image);
                Ok(())
            }
        }
    }
}
