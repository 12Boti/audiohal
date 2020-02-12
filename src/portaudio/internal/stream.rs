use libportaudio_sys as ffi;
use std::os::raw::{c_ulong, c_void};

use crate::error::{Error, Result};
use crate::portaudio::device::DeviceHandle;
use crate::portaudio::error::PaErrorAsResult;
use crate::portaudio::{global_lock, LockGuard, RawPtr};
use crate::stream_options::{Callback, StreamOptions};

/// Convenience structure to collect data needed for stream creation.
pub struct StreamOpenParams<Frame> {
    pub user_options: StreamOptions<Frame>,
    pub pa_params: ffi::PaStreamParameters,
    pub sample_rate: i32,
}

/// Internal stream implementation. Deals with the Portaudio boilerplate.
pub struct StreamImpl<Frame> {
    pa_stream: RawPtr<ffi::PaStream>,
    cb_wrapper: Box<CallbackWrapper<Frame>>,
    _sample_rate: i32,
    /// Handle back to the parent device.
    _parent_device: DeviceHandle,
}

impl<Frame> StreamImpl<Frame> {
    pub fn new_outstream(
        params: StreamOpenParams<Frame>,
        device: DeviceHandle,
    ) -> Result<StreamImpl<Frame>> {
        let _guard = global_lock();
        // Wrap the callback into a thin pointer.
        let callback = Box::new(CallbackWrapper(params.user_options.callback));
        // Create the Portaudio stream.
        let mut stream = StreamImpl {
            pa_stream: RawPtr::dangling(),
            _sample_rate: 0,
            cb_wrapper: callback,
            _parent_device: device,
        };
        unsafe {
            ffi::Pa_OpenStream(
                &mut stream.pa_stream as *const _ as *mut _,
                std::ptr::null(),
                &params.pa_params,
                params.sample_rate.into(),
                params
                    .user_options
                    .frames_per_buffer
                    .unwrap_or(ffi::paFramesPerBufferUnspecified as i32) as c_ulong,
                ffi::PaStreamFlags::PaNoFlag, // No flags
                Some(outstream_callback::<Frame>),
                Box::as_ref(&stream.cb_wrapper) as *const _ as *mut _,
            )
        }
        .as_result()?;
        debug_assert!(!stream.pa_stream.is_null());
        // Verify the frame size.
        is_frame_size_valid::<Frame>(
            params.pa_params.sampleFormat,
            params.user_options.n_channels,
            &_guard,
        )?;
        // Get the stream info.
        let stream_info =
            *(unsafe { ffi::Pa_GetStreamInfo(stream.pa_stream.as_ptr_mut()).as_ref() }
                .ok_or(Error::Unknown("Could not get stream info after creation."))?);
        // TODO: Do something with this sample rate.
        stream._sample_rate = stream_info.sampleRate as i32;
        Ok(stream)
    }

    /// Stream is inactive (i.e. no callback) until this method is called.
    pub fn start(&mut self) -> Result<()> {
        let _guard = global_lock();
        // Make sure the stream isn't actually running.
        match unsafe { ffi::Pa_IsStreamStopped(self.pa_stream.as_ptr() as *mut _) }.into() {
            Ok(0) => Err(Error::StreamAlreadyStarted),
            Ok(_) => Ok(()),
            Err(error) => Err(error.into()),
        }?;
        // Now, open the stream.
        unsafe { ffi::Pa_StartStream(self.pa_stream.as_ptr() as *mut _) }
            .as_result()
            .and(Ok(()))
    }

    /// Closes the stream and deallocates any associated data.
    pub fn close(&mut self) -> Result<()> {
        let _guard = global_lock();
        unsafe { ffi::Pa_CloseStream(self.pa_stream.as_ptr_mut() as *mut _) }.as_result()?;
        Ok(())
    }
}

impl<Frame> Drop for StreamImpl<Frame> {
    fn drop(&mut self) {
        // Immediately stop execution and close.
        self.close()
            .expect("Could not close stream while dropping.");
    }
}

/// Wraps Callback in order to avoid dealing with fat closure pointers.
struct CallbackWrapper<Frame>(Callback<Frame>);

extern "C" fn outstream_callback<Frame>(
    input: *const c_void,
    output: *mut c_void,
    frame_count: c_ulong,
    time_info: *const ffi::PaStreamCallbackTimeInfo,
    status_flags: ffi::PaStreamCallbackFlags,
    user_data: *mut c_void,
) -> i32 {
    let callback = unsafe { (user_data as *mut CallbackWrapper<Frame>).as_mut() }
        .expect("Could not create CallbackWrapper from user_data.");

    let output =
        unsafe { std::slice::from_raw_parts_mut(output as *mut Frame, frame_count as usize) };
    (callback.0)(output);
    0
}

#[must_use]
fn is_frame_size_valid<Frame>(
    pa_format: ffi::PaSampleFormat,
    n_channels: i32,
    _guard: &LockGuard,
) -> Result<()> {
    if n_channels <= 0 {
        return Err(Error::IncompatibleNChannels);
    }
    let pa_sample_size = unsafe { ffi::Pa_GetSampleSize(pa_format) }.as_result()?;
    let pa_frame_size = (pa_sample_size * n_channels) as usize;
    if std::mem::size_of::<Frame>() != pa_frame_size {
        return Err(Error::InvalidFrameSize {
            expected: pa_frame_size,
            actual: std::mem::size_of::<Frame>(),
        });
    }
    Ok(())
}

#[must_use]
fn is_stream_spec_supported<F>(
    params: StreamOpenParams<F>,
    is_output: bool,
    _guard: &LockGuard,
) -> Result<()> {
    if is_output {
        unsafe {
            ffi::Pa_IsFormatSupported(
                std::ptr::null(),
                &params.pa_params,
                params.sample_rate.into(),
            )
        }
    } else {
        unsafe {
            ffi::Pa_IsFormatSupported(
                &params.pa_params,
                std::ptr::null(),
                params.sample_rate.into(),
            )
        }
    }
    .as_result()?;
    Ok(())
}
