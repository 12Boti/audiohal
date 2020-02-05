use libportaudio_sys as ffi;
use std::convert::{TryFrom, TryInto as _};
use std::ptr::NonNull;

use crate::backend::Backend;
use crate::error::{Error, Result};
use crate::portaudio::device::Device;
use crate::portaudio::error::PaErrorAsResult;
use crate::portaudio::{global_lock, LockGuard};

pub type HostHandle = std::sync::Arc<HostImpl>;
pub struct Host(HostHandle);

impl TryFrom<Backend> for ffi::PaHostApiTypeId {
    type Error = crate::error::Error;

    fn try_from(backend: Backend) -> Result<Self> {
        use ffi::PaHostApiTypeId::*;
        use Backend::*;
        match backend {
            Jack => Ok(paJACK),
            PulseAudio => Err(Error::BackendUnavailable),
            Alsa => Ok(paALSA),
            CoreAudio => Ok(paCoreAudio),
            Wasapi => Ok(paWASAPI),
            LinuxFallback => Ok(paOSS),
            Dummy => Ok(paInDevelopment),
            _ => panic!("Backend pattern is not exhaustive."),
        }
    }
}

pub struct HostImpl {
    name: String,
    host_index: ffi::PaHostApiIndex,
    host_info: NonNull<ffi::PaHostApiInfo>,
}

impl HostImpl {
    fn default_output_device_index(&self, _guard: &LockGuard) -> Result<i32> {
        let host_device_index = unsafe { self.host_info.as_ref() }.defaultOutputDevice;
        if host_device_index == ffi::paNoDevice {
            return Err(Error::NoSuchDevice);
        }
        assert!(host_device_index >= 0);
        let device_index =
            unsafe { ffi::Pa_HostApiDeviceIndexToDeviceIndex(self.host_index, host_device_index) };
        if device_index < 0 {
            return Err(ffi::PaErrorCode::from(device_index).into());
        }
        Ok(device_index)
    }
}

impl Host {
    pub fn with_default_backend() -> Result<Host> {
        let guard = global_lock();
        unsafe { ffi::Pa_Initialize().as_result()? };
        let mut host = HostImpl::new();
        // TODO: Expose default backend.
        let host_index = unsafe { ffi::Pa_GetDefaultHostApi() };
        if host_index < 0 {
            return Err(Error::Unknown); // TODO: Better error message.
        }
        host.init_with_pa_host_index(host_index, guard)?;
        Ok(Host(HostHandle::new(host)))
    }

    pub fn with_backend(backend: Backend) -> Result<Host> {
        let guard = global_lock();
        // Initialize Pa.
        unsafe { ffi::Pa_Initialize().as_result()? };
        let mut host = HostImpl::new();
        let pa_backend = backend.try_into()?;
        host.init_with_pa_host_type(pa_backend, guard)?;
        Ok(Host(HostHandle::new(host)))
    }

    /// Returns the host API's descriptive name (e.g. "CoreAudio").
    ///
    /// # Examples
    ///
    /// ```
    /// println!("This system's default host is {}.", audiohal::Host::with_default_backend()?.name());
    /// # audiohal::Result::Ok(())
    /// ```
    pub fn name(&self) -> &str {
        &self.0.name
    }

    /// Creates and returns the default output device for this host.
    ///
    /// This is the recommended device to use for audio playback because it is usually the user's
    /// operating system's default playback device.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut host = audiohal::Host::with_default_backend()?;
    /// let mut output_device = host.default_output_device()?;
    /// println!("{}.", output_device.name());
    /// # audiohal::Result::Ok(())
    /// ```
    ///
    pub fn default_output_device(&mut self) -> Result<Device> {
        let guard = global_lock();
        let device_index = self.0.default_output_device_index(&guard)?;
        crate::portaudio::Device::from_device_index(
            device_index,
            HostHandle::clone(&self.0),
            &guard,
        )
    }
}

impl HostImpl {
    fn new() -> HostImpl {
        HostImpl {
            name: String::new(),
            host_index: -1,
            host_info: NonNull::dangling(),
        }
    }

    /// Expects Pa_Initialize() to have already been called.
    fn init_with_pa_host_type(
        &mut self,
        pa_backend: ffi::PaHostApiTypeId,
        _guard: LockGuard,
    ) -> Result<()> {
        let host_index = unsafe { ffi::Pa_HostApiTypeIdToHostApiIndex(pa_backend) };
        if host_index < 0 {
            return Err(Error::BackendUnavailable);
        }
        self.init_with_pa_host_index(host_index, _guard)
    }

    /// Expects Pa_Initialize() to have already been called.
    fn init_with_pa_host_index(&mut self, host_index: i32, _guard: LockGuard) -> Result<()> {
        debug_assert!(host_index >= 0);
        self.host_index = host_index;
        self.host_info = NonNull::new(unsafe { ffi::Pa_GetHostApiInfo(host_index) } as *mut _)
            .ok_or(Error::BackendUnavailable)?;
        self.name = unsafe { std::ffi::CStr::from_ptr(self.host_info.as_ref().name) }
            .to_str()
            .or(Err(Error::Unknown))? // TODO: Better error message.
            .to_string();
        Ok(())
    }
}

impl Drop for HostImpl {
    fn drop(&mut self) {
        let _guard = global_lock();
        unsafe { ffi::Pa_Terminate() }.as_result().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creates_default_backend() {
        let host = Host::with_default_backend().unwrap();
        println!("Host is called {}", host.name());
    }

    #[test]
    fn test_name() {
        let host = Host::with_default_backend().unwrap();
        assert_eq!(host.0.name, host.name());
    }
}
