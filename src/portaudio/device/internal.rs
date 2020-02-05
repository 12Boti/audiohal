use libportaudio_sys as ffi;
use std::ptr::NonNull;

use crate::error::{Error, Result};
use crate::portaudio::error::PaErrorAsResult;
use crate::portaudio::host::HostHandle;
use crate::portaudio::LockGuard;
use crate::stream_options::StreamOptions;
use crate::{Format, SampleRate};
use std::convert::TryInto;

use std::os::raw::{c_ulong, c_void};
extern "C" fn dummy_callback(
    input: *const c_void,
    output: *mut c_void,
    frame_count: c_ulong,
    time_info: *const ffi::PaStreamCallbackTimeInfo,
    status_flags: ffi::PaStreamCallbackFlags,
    user_data: *mut c_void,
) -> i32 {
    0
}

pub struct Device {
    pub name: String,
    pub info: NonNull<ffi::PaDeviceInfo>,

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
        let device_info: NonNull<ffi::PaDeviceInfo> =
            NonNull::new(unsafe { ffi::Pa_GetDeviceInfo(index) as *mut _ })
                .ok_or(Error::NoSuchDevice)?;
        let name = unsafe { std::ffi::CStr::from_ptr(device_info.as_ref().name) }
            .to_str()
            .or(Err(Error::Unknown))? // TODO: Better error message.
            .to_string();
        Ok(Device {
            name,
            info: device_info,
            index,
            _parent_host: host_handle,
        })
    }

    pub fn open_outstream<Frame>(&self, options: &StreamOptions<Frame>) -> Result<()> {
        let (params, sample_rate) = self.options_to_stream_params(options)?;
        let mut pa_stream = std::ptr::null_mut();
        unsafe {
            ffi::Pa_OpenStream(
                &mut pa_stream,
                std::ptr::null(),
                &params,
                sample_rate.into(),
                options
                    .frames_per_buffer
                    .unwrap_or(ffi::paFramesPerBufferUnspecified as i32) as c_ulong,
                ffi::PaStreamFlags::PaNoFlag, // No flags
                Some(dummy_callback),
                std::ptr::null_mut(),
            )
        }
        .as_result()?;
        Ok(())
    }

    pub fn is_stream_spec_supported<F>(
        &self,
        options: &StreamOptions<F>,
        is_output: bool,
        _guard: &LockGuard,
    ) -> Result<()> {
        let (params, sample_rate) = self.options_to_stream_params(options)?;
        if is_output {
            unsafe { ffi::Pa_IsFormatSupported(std::ptr::null(), &params, sample_rate.into()) }
        } else {
            unsafe { ffi::Pa_IsFormatSupported(&params, std::ptr::null(), sample_rate.into()) }
        }
        .as_result()?;
        Ok(())
    }

    fn options_to_stream_params<F>(
        &self,
        options: &StreamOptions<F>,
    ) -> Result<(ffi::PaStreamParameters, i32)> {
        let sample_rate = match options.sample_rate {
            Some(SampleRate::Exact(rate)) => rate,
            None | Some(SampleRate::NearestTo(_)) => {
                unsafe { self.info.as_ref() }.defaultSampleRate as i32
            }
            _ => panic!("Non-exhaustive sample rate."),
        };
        let latency = if let Some(frames_per_buffer) = options.frames_per_buffer {
            frames_per_buffer_to_latency(frames_per_buffer, sample_rate)
        } else {
            unsafe { self.info.as_ref().defaultHighOutputLatency }
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
    f64::from(sample_rate) / f64::from(frames_per_buffer)
}
