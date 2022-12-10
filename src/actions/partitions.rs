use crate::pt::determine_pt_type;

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
        let pt_type = determine_pt_type(image);
        dbg!("Found PT {}", pt_type);

        Ok(())
    }
}
