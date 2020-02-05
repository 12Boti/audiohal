use libportaudio_sys as ffi;

use crate::error::{Error, Result};

impl From<ffi::PaError> for Error {
    fn from(error: ffi::PaError) -> Error {
        use ffi::PaErrorCode::*;
        use Error::*;

        match error {
            paInsufficientMemory => NoMem,
            paInvalidHostApi => Invalid,
            paInternalError => Unknown,
            paHostApiNotFound => BackendUnavailable,
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
