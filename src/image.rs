use std::{
    fs::{File, OpenOptions},
    path::Path,
};

use memmap::{Mmap, MmapMut};

const BLOCK_SIZE: usize = 512;

/// Error during creation of disk image.
#[derive(Debug, displaydoc::Display, thiserror::Error)]
pub enum ImageError {
    /// Unable to open image file
    OpenError,
    /// Unable to map image file
    MapError,
}

pub struct Image {
    block_size: usize,
    mem: MmapMut,
}

impl Image {
    pub fn from_file(file: File) -> Result<Self, ImageError> {
        let mem = unsafe { Mmap::map(&file).map_err(|_| ImageError::MapError)? }
            .make_mut()
            .map_err(|_| ImageError::MapError)?;

        Ok(Image {
            block_size: BLOCK_SIZE,
            mem,
        })
    }

    pub fn open<P>(path: P) -> Result<Self, ImageError>
    where
        P: AsRef<Path>,
    {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .truncate(false)
            .append(false)
            .open(path)
            .map_err(|_| ImageError::OpenError)?;

        Self::from_file(file)
    }

    pub fn get_blocks(&self, block_index: usize, block_count: usize) -> &[u8] {
        let block_start = block_index * self.block_size;
        let block_end = block_start + (block_count * self.block_size);

        &self.mem[block_start..block_end]
    }

    pub fn get_blocks_mut(&mut self, block_index: usize, block_count: usize) -> &mut [u8] {
        let block_start = block_index * self.block_size;
        let block_end = block_start + (block_count * self.block_size);

        &mut self.mem[block_start..block_end]
    }

    pub fn read<T>(&self, offset: usize) -> T {
        unsafe { std::ptr::read(self.mem[offset..].as_ptr() as *const _) }
    }

    pub fn write<T>(&mut self, offset: usize, obj: T) {
        unsafe { std::ptr::write(self.mem[offset..].as_mut_ptr() as *mut _, obj) }
    }

    pub fn len(&self) -> usize {
        self.mem.len()
    }
}
