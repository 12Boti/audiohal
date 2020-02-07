use libportaudio_sys as ffi;

use crate::error::{Error, Result};

impl From<ffi::PaError> for Error {
    fn from(error: ffi::PaError) -> Error {
        use ffi::PaErrorCode::*;
        use Error::*;

        match error {
            paInsufficientMemory => OutOfMemory,
            paInvalidHostApi => BackendUnavailable,
            paInternalError => Unknown,
            paHostApiNotFound => BackendUnavailable,
            // Not actually sure how to handle paNotInitialized. Should never happen
            // under normal circumstances.
            paNotInitialized => Unknown, 
            _ => panic!("Figure out error mapping for {:?}", error),
        }
    }
}

pub trait PaErrorAsResult: Sized {
    fn as_result(self) -> Result<Self>;
}

impl PaErrorAsResult for ffi::PaErrorCode {
    fn as_result(self) -> Result<Self> {
        match self {
            ffi::PaErrorCode::paNoError => Ok(self),
            _ => Err(self.into()),
        }
    }
}
