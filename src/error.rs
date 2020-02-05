use std::result;

use crate::Format;

#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum Error {
    NoMem,
    Unknown,
    Invalid,
    BackendUnavailable,
    NoSuchDevice,
    IncompatibleFormat(Format),
}

pub type Result<T> = result::Result<T, Error>;
