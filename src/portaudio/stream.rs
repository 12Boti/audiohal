use libportaudio_sys as ffi;
use std::ptr::NonNull;

pub type StreamHandle = std::sync::Arc<StreamImpl>;
pub struct Stream(StreamHandle);

struct StreamImpl {
    pa_stream: NonNull<ffi::PaStream>,
    /// Stores a copy of PaStreamInfo, since it is immutable once a stream is created.
    info: ffi::PaStreamInfo,
}

impl StreamImpl {
    fn from_options_and_params() {}
}