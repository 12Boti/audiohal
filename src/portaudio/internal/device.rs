use libportaudio_sys as ffi;
use std::convert::TryInto;

use crate::error::{Error, Result};
use crate::portaudio::device::DeviceHandle;
use crate::portaudio::host::HostHandle;
use crate::portaudio::internal::stream::StreamOpenParams;
use crate::portaudio::stream::{new_outstream, Stream};
use crate::portaudio::{LockGuard, RawPtr};
use crate::stream_options::StreamOptions;
use crate::SampleRate;

pub struct Device {
    pub name: String,
    info: RawPtr<ffi::PaDeviceInfo>,

    index: i32,
    /// Handle to parent host.
    _parent_host: HostHandle,
}

impl Device {
    /// Creates a portaudio Device from a given device index.
    pub fn from_device_index(
        index: i32,
        host_handle: HostHandle,
        _guard: &LockGuard,
    ) -> Result<Device> {
        debug_assert_ge!(index, 0);
        let device_info = unsafe {
            ffi::Pa_GetDeviceInfo(index)
                .as_ref()
                .ok_or(Error::NoSuchDevice)?
        };
        let name = unsafe { std::ffi::CStr::from_ptr(device_info.name) }
            .to_str()
            .or(Err(Error::Unknown(
                "Could not convert device name to UTF-8.",
            )))?
            .to_string();
        Ok(Device {
            name,
            info: RawPtr::new(device_info as *const _).unwrap(),
            index,
            _parent_host: host_handle,
        })
    }

    pub fn open_outstream<Frame>(
        &self,
        options: StreamOptions<Frame>,
        device_handle: DeviceHandle,
    ) -> Result<Stream<Frame>> {
        // Early-out if the stream spec is not supported?
        // self.is_stream_spec_supported(&options, true, &global_lock())?;
        let (params, sample_rate) = self.options_to_stream_params(&options)?;
        let open_params = StreamOpenParams {
            user_options: options,
            pa_params: params,
            sample_rate,
        };
        new_outstream(open_params, device_handle)
    }

    fn options_to_stream_params<F>(
        &self,
        options: &StreamOptions<F>,
    ) -> Result<(ffi::PaStreamParameters, i32)> {
        let info = unsafe { self.info.as_ref().unwrap() };
        let sample_rate = match options.sample_rate {
            SampleRate::Exact(rate) => rate,
            SampleRate::DeviceDefault | SampleRate::NearestTo(_) => info.defaultSampleRate as i32,
            _ => panic!("Non-exhaustive sample rate."),
        };
        let latency = if let Some(frames_per_buffer) = options.frames_per_buffer {
            if frames_per_buffer <= 0 {
                return Err(Error::InvalidFramesPerBuffer);
            }
            frames_per_buffer_to_latency(frames_per_buffer, sample_rate)
        } else {
            info.defaultHighOutputLatency
        };
        Ok((
            ffi::PaStreamParameters {
                device: self.index,
                channelCount: options.n_channels,
                sampleFormat: options.format.try_into()?,
                suggestedLatency: latency,
                hostApiSpecificStreamInfo: std::ptr::null_mut(),
            },
            sample_rate,
        ))
    }
}

fn frames_per_buffer_to_latency(frames_per_buffer: i32, sample_rate: i32) -> f64 {
    f64::from(frames_per_buffer) / f64::from(sample_rate)
}
