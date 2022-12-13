use crate::pt::{read_partition_table, PartitionTable};

use super::Action;

pub struct ListPartitionsArgs {}

#[derive(Debug, displaydoc::Display, thiserror::Error)]
pub enum ListPartitionsError {
    /// Generic Error
    GenericError,
}

pub struct ListPartitionsAction {}

impl Action<ListPartitionsArgs, ListPartitionsError> for ListPartitionsAction {
    fn invoke(
        image: &mut crate::image::Image,
        args: ListPartitionsArgs,
    ) -> Result<(), ListPartitionsError> {
        // Determine partition table type
        let pt = read_partition_table(image);

        match pt {
            Some(PartitionTable::MBR(mbr)) => {
                println!("found mbr:");
                println!("{}", mbr);
            }
            Some(PartitionTable::GPT(gpt)) => {
                println!("found gpt:");
                println!("{}", gpt);
            }
            None => {
                println!("no partition table found");
            }
        }

        Ok(())
    }
}
