use libportaudio_sys as ffi;
use std::os::raw::{c_ulong, c_void};
use std::pin::Pin;
use std::sync::{Arc, Weak};

use crate::error::{Error, Result};
use crate::portaudio::device::DeviceHandle;
use crate::portaudio::error::PaErrorAsResult;
use crate::portaudio::{global_lock, RawPtr};
use crate::stream_options::Callback;
use crate::StreamOptions;

type StreamHandle<Frame> = Arc<StreamImpl<Frame>>;

/// A stream represents the flow of data in and out of an audio device. It's defined by its audio
/// data format, the number of channels, and whether it is an input stream (e.g. a microphone) or
/// an output stream (e.g. speakers).
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

struct StreamImpl<Frame> {
    pa_stream: RawPtr<ffi::PaStream>,
    /// Stores a copy of PaStreamInfo, since it is immutable once a stream is created.
    info: ffi::PaStreamInfo,
    callback: Callback<Frame>,
    /// Handle back to the parent device.
    _parent_device: DeviceHandle,
}

/// Convenience structure to collect data needed for stream creation.
pub struct StreamOpenParams<Frame> {
    pub user_options: StreamOptions<Frame>,
    pub pa_params: ffi::PaStreamParameters,
    pub sample_rate: i32,
}

impl<Frame> Drop for StreamImpl<Frame> {
    fn drop(&mut self) {
        // Immediately stop execution.
        self.abort()
            .expect("Could not abort stream while dropping.");
    }
}

impl<Frame> StreamImpl<Frame> {
    fn new_outstream(
        params: StreamOpenParams<Frame>,
        device: DeviceHandle,
    ) -> Result<StreamHandle<Frame>> {
        let _guard = global_lock();
        // Create (but not initialize) a weak reference from the callback back into the stream. Once
        // the into_raw api is stabilized, this won't have to be such a hack. This weak reference
        // will prevent any race conditions if the StreamImpl is dropped while in the middle of a
        // callback.
        //let callback_ref = Arc::new(params.user_options.callback);
        let mut pa_stream: *const ffi::PaStream = std::ptr::null();
        unsafe {
            ffi::Pa_OpenStream(
                &mut pa_stream as *const _ as *mut _,
                std::ptr::null(),
                &params.pa_params,
                params.sample_rate.into(),
                params
                    .user_options
                    .frames_per_buffer
                    .unwrap_or(ffi::paFramesPerBufferUnspecified as i32) as c_ulong,
                ffi::PaStreamFlags::PaNoFlag, // No flags
                Some(outstream_callback),
                Box::as_ref(&params.user_options.callback) as *const _ as *mut _,
            )
        }
        .as_result()?;
        debug_assert!(!pa_stream.is_null());

        // Get the stream info.
        let stream_info = *(unsafe { ffi::Pa_GetStreamInfo(pa_stream as *mut _).as_ref() }
            .ok_or(Error::Unknown("Could not get stream info after creation."))?);

        let stream_handle = Arc::new(StreamImpl {
            pa_stream: RawPtr::new(pa_stream).unwrap(),
            info: stream_info,
            callback: params.user_options.callback,
            _parent_device: device,
        });
        Ok(stream_handle)
    }

    fn start(&mut self) -> Result<()> {
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

    /// Aborts the execution of the stream, throwing away any data already buffered. Does nothing
    /// if the stream is not running.
    fn abort(&mut self) -> Result<()> {
        let _guard = global_lock();
        match unsafe { ffi::Pa_AbortStream(self.pa_stream.as_ptr() as *mut _) }.into() {
            Err(ffi::PaErrorCode::paStreamIsStopped) => Ok(()),
            Err(code) => Err(code.into()),
            _ => Ok(()),
        }
    }
}

pub fn new_outstream<Frame>(
    params: StreamOpenParams<Frame>,
    device: DeviceHandle,
) -> Result<Stream<Frame>> {
    Ok(Stream(StreamImpl::new_outstream(params, device)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portaudio::test_prelude::*;
    use crate::SampleRate;
    use std::sync::{Condvar, Mutex};
    use std::thread;
    use std::time::Duration;

    fn make_stream_with(options: StreamOptions<[f32; 2]>) -> Result<Stream<[f32; 2]>> {
        let mut device = Host::with_default_backend()?.default_output_device()?;
        device.open_outstream(options)
    }

    #[test]
    fn stream_is_send() {
        assert_send::<Stream<[f32; 2]>>();
    }

    #[test]
    fn creates_outstream() -> Result<()> {
        let mut device = Host::with_default_backend()?.default_output_device()?;
        let stream = device.open_outstream(StreamOptions::<[f32; 2]>::default())?;
        Ok(())
    }

    //    #[test]
    //    fn can_start_stream() -> Result<()> {
    //        let (mutex, cond) = (Mutex::new(()), Condvar::new());
    //        let (mutex2, cond2) = (mutex.clone(), cond2.clone());
    //        let cb = move |_: &mut [[f32; 2]]| {
    //            // Wait for a little bit.
    //            thread::sleep(Duration::from_millis(100));
    //            cond2.notify_one();
    //        };
    //        let stream = make_stream_with(StreamOptions {
    //            callback: Box::new(cb),
    //        })?;
    //    }

    #[test]
    fn errors_if_invalid_sample_rate() {
        let stream = make_stream_with(StreamOptions {
            sample_rate: Some(SampleRate::Exact(-100)),
            ..Default::default()
        });
        assert_that!(&stream, maybe_err(eq(Error::IncompatibleSampleRate)));
    }

    #[test]
    fn errors_if_incompatible_sample_rate() {
        assert_that!(
            &make_stream_with(StreamOptions {
                sample_rate: Some(SampleRate::Exact(1)),
                ..Default::default()
            }),
            maybe_err(eq(Error::IncompatibleSampleRate))
        );
    }

    #[test]
    fn errors_if_invalid_frames_per_buffer() {
        assert_that!(
            &make_stream_with(StreamOptions {
                frames_per_buffer: Some(-100),
                ..Default::default()
            }),
            maybe_err(eq(Error::InvalidFramesPerBuffer))
        );
    }

    #[test]
    fn errors_if_unsupported_n_channels() {
        assert_that!(
            &make_stream_with(StreamOptions {
                n_channels: 100_000,
                ..Default::default()
            }),
            maybe_err(eq(Error::IncompatibleNChannels))
        );
        assert_that!(
            &make_stream_with(StreamOptions {
                n_channels: -100,
                ..Default::default()
            }),
            maybe_err(eq(Error::IncompatibleNChannels))
        );
    }
}
