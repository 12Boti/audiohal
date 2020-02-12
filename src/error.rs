use std::fmt;
use std::result;

use crate::Format;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    /// An out-of-memory occurred while allocating in a C library.
    OutOfMemory,
    /// An unexpected and unrecoverable error.
    Unknown(&'static str),
    Invalid,
    InvalidFramesPerBuffer,
    /// The size of the frame type specified in the stream callback does not match the expected size
    /// of [`Format`].
    InvalidFrameSize {
        expected: usize,
        actual: usize,
    },
    /// The backend requested was either not compiled, or is uninitializable.
    BackendUnavailable,
    /// The requested device was unavailable.
    NoSuchDevice,
    /// The requested format is not compatible with the device in-use.
    IncompatibleFormat(Format),
    /// The requested sample rate is not compatible with the device.
    IncompatibleSampleRate,
    /// The requested number of channels is not compatible with the device.
    IncompatibleNChannels,
    /// ['Stream::start`] called on stream that has already started.
    StreamAlreadyStarted,
}

pub type Result<T> = result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Actually make this nice.
        fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for Error {}
