use std::path::Path;

use nix::{
    fcntl::{FallocateFlags, OFlag},
    sys::stat::Mode,
};

use crate::pt::gpt::GPT;
use crate::{image::Image, pt::mbr::MBR};

use super::init::PartitionTableType;

pub struct CreateActionArgs {
    pub size: i64,
    pub overwrite: bool,
    pub initial_pt_type: PartitionTableType,
}

/// Error during creation of disk image.
#[derive(Debug, displaydoc::Display, thiserror::Error)]
pub enum CreateError {
    /// Unable to open output file.
    OpenError,
    /// Unable to map output file.
    MapError,
    /// Unable to allocate space for output file.
    AllocationFailedError,
    /// The image file already exists, and force overwrite was not specified.
    FileAlreadyExistsError,
}

pub fn invoke(image_file: &String, ca: CreateActionArgs) -> Result<(), CreateError> {
    let p = Path::new(image_file);

    // Check for the existence of the image file
    if p.exists() {
        if ca.overwrite {
            println!("Image file already exists, but --overwrite was specified so re-creating!");
        } else {
            return Err(CreateError::FileAlreadyExistsError);
        }
    } else if ca.overwrite {
        println!("Warning: overwrite was specified, but the image file does not already exist!");
    }

    println!("Creating a disk image of size {}", ca.size);

    // We need to use the *nix APIs to create a sparse file.
    let fd = nix::fcntl::open(
        p,
        OFlag::O_CREAT | OFlag::O_TRUNC | OFlag::O_RDWR,
        Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH,
    )
    .map_err(|_| CreateError::OpenError)?;

    nix::fcntl::fallocate(fd, FallocateFlags::empty(), 0, ca.size)
        .map_err(|_| CreateError::AllocationFailedError)?;

    // A bit hacky, but this will clear any existing MBR.
    nix::unistd::write(fd, &[0; 512]);

    nix::unistd::close(fd).unwrap();

    // OK - and now, see if we're also creating an initial partition table.

    let mut image = Image::open(p).map_err(|_| CreateError::OpenError)?;

    match ca.initial_pt_type {
        PartitionTableType::None => Ok(()),
        PartitionTableType::MBR => {
            let mbr = MBR::new();
            mbr.write(&mut image);
            Ok(())
        }
        PartitionTableType::GPT => {
            let gpt = GPT::new();
            gpt.write(&mut image);
            Ok(())
        }
    }
}
