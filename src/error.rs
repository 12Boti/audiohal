use std::result;

use crate::Format;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    /// An out-of-memory occured while allocating in a C library.
    OutOfMemory,
    /// An unexpected and unrecoverable error.
    Unknown,
    Invalid,
    /// The backend requested was either not compiled, or is not initializable.
    BackendUnavailable,
    /// The requested device was unavailable.
    NoSuchDevice,
    /// The requested format is not compatible with the device in-use.
    IncompatibleFormat(Format),
}

pub type Result<T> = result::Result<T, Error>;
