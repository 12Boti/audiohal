use std::sync::Arc;

use crate::error::Result;
use crate::portaudio::host::HostHandle;
use crate::portaudio::LockGuard;
use crate::stream_options::StreamOptions;
use crate::Stream;

use crate::portaudio::internal::device as internal;

pub type DeviceHandle = Arc<internal::Device>;
pub struct Device(DeviceHandle);

impl Device {
    /// The device's system name (e.g. "Built-in Output").
    pub fn name(&self) -> &str {
        &self.0.name
    }

    /// Creates an output stream.
    ///
    /// `Frame` is the stream's frame type, and is inferred from the stream callback.
    ///
    /// Output streams stream digital audio (in the form of frames) to a system's output device.
    /// The callback in  [`StreamOptions`] is called multiple times per second (depending on how you
    /// setup frames_per_buffer) in order to satisfy the requested sample-rate. See
    /// [`Stream`] for more details.
    ///
    /// # Examples
    ///
    /// ```
    /// # use audiohal::*;
    /// fn callback(buffer: &mut [[f32; 2]]) {
    ///     # buffer;
    /// }
    /// let mut device = Host::with_default_backend()?.default_output_device()?;
    /// let stream = device.open_outstream(
    ///     StreamOptions {
    ///         callback: Box::new(callback),
    ///         // The rest of the parameters will be set to device defaults.
    ///         ..Default::default()
    ///     });
    /// assert!(stream.is_ok());
    /// # Result::Ok(())
    /// ```
    pub fn open_outstream<Frame>(
        &mut self,
        options: StreamOptions<Frame>,
    ) -> Result<Stream<Frame>> {
        self.0.open_outstream(options, Arc::clone(&self.0))
    }
}

pub fn from_device_index(
    index: i32,
    host_handle: HostHandle,
    _guard: &LockGuard,
) -> Result<Device> {
    Ok(Device(DeviceHandle::new(
        internal::Device::from_device_index(index, host_handle, _guard)?,
    )))
}

#[cfg(test)]
mod tests {
    use crate::portaudio::test_prelude::*;

    #[test]
    fn device_is_send() {
        assert_send::<Device>();
    }

    #[test]
    fn device_holds_host_ref() -> Result<()> {
        begin!();
        let _device = {
            let mut host = Host::with_default_backend()?;
            host.default_output_device()?
        };
        // Host is out-of-scope, but pa must still be initialized.
        assert!(is_initialized());
        Ok(())
    }
}
