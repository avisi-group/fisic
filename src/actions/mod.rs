use crate::image::Image;

pub mod create;
pub mod init;
pub mod partitions;

pub trait Action<T, E> {
    fn invoke(image: &mut Image, args: T) -> Result<(), E>;
}
