use libportaudio_sys as ffi;
use std::os::raw::{c_ulong, c_void};
use std::ptr::NonNull;
use std::sync::{Arc, Weak};

use crate::error::{Error, Result};
use crate::portaudio::device::DeviceHandle;
use crate::portaudio::error::PaErrorAsResult;
use crate::portaudio::global_lock;
use crate::StreamOptions;

pub type StreamHandle<Frame> = Arc<StreamImpl<Frame>>;
pub struct Stream<Frame>(StreamHandle<Frame>);

extern "C" fn outstream_callback(
    input: *const c_void,
    output: *mut c_void,
    frame_count: c_ulong,
    time_info: *const ffi::PaStreamCallbackTimeInfo,
    status_flags: ffi::PaStreamCallbackFlags,
    user_data: *mut c_void,
) -> i32 {
    0.into()
}

/// Convenience structure to collect data needed for stream creation.
pub struct StreamOpenParams<Frame> {
    pub user_options: StreamOptions<Frame>,
    pub pa_params: ffi::PaStreamParameters,
    pub sample_rate: i32,
}

struct StreamImpl<Frame> {
    pa_stream: NonNull<ffi::PaStream>,
    /// Stores a copy of PaStreamInfo, since it is immutable once a stream is created.
    info: ffi::PaStreamInfo,
    /// StreamOptions used to create this stream. Kept for callback.
    options: StreamOptions<Frame>,
    /// The location of the

    /// Handle back to the parent device.
    _parent_device: DeviceHandle,
}

impl<Frame> Drop for StreamImpl<Frame> {
    fn drop(&mut self) {}
}

impl<Frame> StreamImpl<Frame> {
    fn new_outstream(
        params: StreamOpenParams<Frame>,
        device: DeviceHandle,
    ) -> Result<StreamHandle<Frame>> {
        let _guard = global_lock();

        let frames_per_buffer = params
            .user_options
            .frames_per_buffer
            .unwrap_or(ffi::paFramesPerBufferUnspecified as i32);
        // Initialize the StreamImpl now so that we can get a reference to it.
        let mut stream_handle = Arc::new(StreamImpl {
            pa_stream: NonNull::dangling(),
            info: ffi::PaStreamInfo {
                structVersion: 0,
                inputLatency: 0.0,
                outputLatency: 0.0,
                sampleRate: 0.0,
            },
            options: params.user_options,
            _parent_device: device,
        });
        // Create (but not initialize) a weak reference from the callback back into the stream. Once
        // the into_raw api is
        // stabilized, this won't have to be such a hack. This weak reference will prevent any race
        // conditions if the StreamImpl is dropped while in the middle of a callback.
        let weak_ref: Box<Weak<StreamImpl<Frame>>> = Box::new(Arc::downgrade(&stream_handle));

        let mut pa_stream = std::ptr::null_mut();
        unsafe {
            ffi::Pa_OpenStream(
                &mut pa_stream,
                std::ptr::null(),
                &params.pa_params,
                params.sample_rate.into(),
                frames_per_buffer as c_ulong,
                ffi::PaStreamFlags::PaNoFlag, // No flags
                Some(outstream_callback),
                weak_ref.as_ref() as *const _ as *mut _,
            )
        }
        .as_result()?;
        debug_assert!(!pa_stream.is_null());

        // Be a little unsafe to set "info" in stream_handle, even though Arc does not allow
        // interior mutability.
        unsafe {
            (*(stream_handle.as_ref() as *const _ as *mut StreamImpl<Frame>)).info =
                (*unsafe { ffi::Pa_GetStreamInfo(pa_stream).as_ref() }.ok_or(Error::Unknown)?);
        }

        // Finally, leak the weak_ref to give ownership to the callback.
        Box::leak(weak_ref);
        Ok(stream_handle)
    }
}

pub fn new_outstream<Frame>(
    params: StreamOpenParams<Frame>,
    device: DeviceHandle,
) -> Result<Stream<Frame>> {
    Ok(Stream(StreamImpl::new_outstream(params, device)?))
}
