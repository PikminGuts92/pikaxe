mod archive;
mod compression;
mod crypt;
mod file;
mod stream;

pub use self::archive::*;
pub use self::compression::*;
pub use self::crypt::*;
pub use self::file::*;
pub use self::stream::*;

pub(crate) fn align_to_multiple_of_four(n: usize) -> usize {
    (n + 3) & !3
}