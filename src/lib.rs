extern crate core;

pub trait Required {
    fn args() -> Self;
}

impl<T: Default> Required for T {
    fn args() -> Self {
        <Self as Default>::default()
    }
}

pub mod read;
pub mod error;
pub mod endian;
pub mod write;
pub mod io;
pub mod ext;

pub use error::*;
pub use endian::*;
pub use read::*;
pub use write::*;
pub use ext::strings::*;

pub type BinResult<T> = std::result::Result<T, Error>;
