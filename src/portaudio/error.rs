use libportaudio_sys as ffi;
use std::os::raw::c_int;

use crate::error::{Error, Result};

impl From<ffi::PaErrorCode> for Error {
    fn from(error: ffi::PaErrorCode) -> Error {
        use ffi::PaErrorCode::*;
        use Error::*;

        match error {
            paInsufficientMemory => OutOfMemory,
            paInvalidHostApi => BackendUnavailable,
            paInternalError => Unknown("Portaudio internal error."),
            paHostApiNotFound => BackendUnavailable,
            paInvalidSampleRate => IncompatibleSampleRate,
            paInvalidChannelCount => IncompatibleNChannels,
            // Not actually sure how to handle paNotInitialized. Should never happen
            // under normal circumstances.
            paNotInitialized => Unknown("Portaudio not initialized."),
            _ => panic!("Figure out error mapping for {:?}", error),
        }
    }
}

pub trait PaErrorAsResult: Sized {
    fn as_result(self) -> Result<c_int>;
}

impl PaErrorAsResult for ffi::PaError {
    fn as_result(self) -> Result<c_int> {
        match self.into() {
            Err(err) => Err(err.into()),
            Ok(val) => Ok(val),
        }
    }
}
