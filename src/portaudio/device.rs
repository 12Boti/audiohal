use crate::error::Result;
use crate::portaudio::host::HostHandle;
use crate::portaudio::{global_lock, LockGuard};
use crate::stream_options::StreamOptions;

pub mod internal;

pub type DeviceHandle = std::sync::Arc<internal::Device>;
pub struct Device(DeviceHandle);

impl Device {
    /// The device's system name (e.g. "Built-in Output").
    pub fn name(&self) -> &str {
        &self.0.name
    }

    /// Creates an output stream.
    ///
    /// Output streams stream digital audio (in the form of frames) to a system's output device.
    /// The callback in  [`StreamOptions`] is called multiple times per second (depending on how you
    /// setup frames_per_buffer) in order to satisfy the requested sample-rate.
    pub fn open_outstream<Frame: sample::Frame>(
        &mut self,
        options: StreamOptions<Frame>,
    ) -> Result<()> {
        unimplemented!();
    }
}

impl Device {}
