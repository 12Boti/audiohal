use crate::error::Result;
use crate::portaudio::device::DeviceHandle;

use crate::portaudio::internal::stream as internal;

/// A stream represents the flow of data in and out of an audio device. It's defined by its audio
/// data format, the number of channels, and whether it is an input stream (e.g. a microphone) or
/// an output stream (e.g. speakers).
pub struct Stream<Frame>(internal::StreamImpl<Frame>);

// impl<Frame> StreamImpl<Frame> {

//     /// Aborts the execution of the stream, throwing away any data already buffered. Does nothing
//     /// if the stream is not running.
//     fn abort(&mut self) -> Result<()> {
//         let _guard = global_lock();
//         match unsafe { ffi::Pa_AbortStream(self.pa_stream.as_ptr() as *mut _) }.into() {
//             Err(ffi::PaErrorCode::paStreamIsStopped) => Ok(()),
//             Err(code) => Err(code.into()),
//             _ => Ok(()),
//         }
//     }

// }

impl<Frame> Stream<Frame> {
    pub fn start(&mut self) -> Result<()> {
        self.0.start()
    }

    pub fn close(mut self) {
        self.0
            .close()
            .expect("Could not close stream. No obvious way to cleanly handle error.")
    }
}

pub fn new_outstream<Frame>(
    params: internal::StreamOpenParams<Frame>,
    device: DeviceHandle,
) -> Result<Stream<Frame>> {
    Ok(Stream(internal::StreamImpl::new_outstream(params, device)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::portaudio::test_prelude::*;
    use crate::SampleRate;
    use std::sync::Arc;
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
        begin!();
        let mut device = Host::with_default_backend()?.default_output_device()?;
        device.open_outstream(StreamOptions::<[f32; 2]>::default())?;
        Ok(())
    }

    #[test]
    fn can_start_stream() -> Result<()> {
        begin!();
        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        let pair2 = Arc::clone(&pair);
        let cb = move |buffer: &mut [[f32; 2]]| {
            for val in buffer.iter_mut() {
                *val = [0.0, 0.0];
            }
            // Wait for a little bit.
            thread::sleep(Duration::from_millis(100));
            let (lock, cvar) = &*pair2;
            *lock.lock().unwrap() = true;
            cvar.notify_one();
        };
        let mut stream = make_stream_with(StreamOptions {
            callback: Box::new(cb),
            ..Default::default()
        })?;
        stream.start()?;
        let (lock, cvar) = &*pair;
        // Wait for our notification.
        let (guard, _) = cvar
            .wait_timeout(lock.lock().unwrap(), Duration::from_secs(20))
            .unwrap();
        assert_eq!(*guard, true);
        Ok(())
    }

    #[test]
    fn errors_if_invalid_sample_rate() {
        begin!();
        let stream = make_stream_with(StreamOptions {
            sample_rate: SampleRate::Exact(-100),
            ..Default::default()
        });
        assert_that!(&stream, maybe_err(eq(Error::IncompatibleSampleRate)));
    }

    #[test]
    fn errors_if_incompatible_sample_rate() {
        begin!();
        assert_that!(
            &make_stream_with(StreamOptions {
                sample_rate: SampleRate::Exact(1),
                ..Default::default()
            }),
            maybe_err(eq(Error::IncompatibleSampleRate))
        );
    }

    #[test]
    fn errors_if_invalid_frames_per_buffer() {
        begin!();
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
        begin!();
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
