use libportaudio_sys as ffi;
use std::convert::{TryFrom, TryInto as _};

use crate::backend::Backend;
use crate::error::{Error, Result};
use crate::portaudio::device::Device;
use crate::portaudio::error::PaErrorAsResult;
use crate::portaudio::{global_lock, LockGuard, RawPtr};

pub type HostHandle = std::sync::Arc<HostImpl>;
pub struct Host(HostHandle);

impl TryFrom<Backend> for ffi::PaHostApiTypeId {
    type Error = crate::error::Error;

    fn try_from(backend: Backend) -> Result<Self> {
        use ffi::PaHostApiTypeId::*;
        use Backend::*;
        match backend {
            Jack => Ok(paJACK),
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
    host_info: RawPtr<ffi::PaHostApiInfo>,
}

impl Host {
    /// Creates a host with the default system backend.
    pub fn with_default_backend() -> Result<Host> {
        let _guard = global_lock();
        unsafe { ffi::Pa_Initialize() }.as_result()?;
        let mut host = HostImpl::new();
        // TODO: Expose default backend.
        let host_index = unsafe { ffi::Pa_GetDefaultHostApi() };
        if host_index < 0 {
            return Err(ffi::PaError::from(host_index).as_result().unwrap_err());
        }
        host.init_with_pa_host_index(host_index, _guard)?;
        Ok(Host(HostHandle::new(host)))
    }

    /// Creates a host with a specific backend.
    ///
    /// Will return [`Error::BackendUnavailable`] if the backend support was not
    /// compiled.
    ///
    /// # Examples
    /// ```
    /// # use audiohal::*;
    /// assert!(Host::with_backend(Backend::Dummy).is_ok(), "The dummy backend should always be available.");
    /// ```
    pub fn with_backend(backend: Backend) -> Result<Host> {
        let _guard = global_lock();
        // Initialize Pa.
        unsafe { ffi::Pa_Initialize() }.as_result()?;
        let mut host = HostImpl::new();
        let pa_backend = backend.try_into()?;
        host.init_with_pa_host_type(pa_backend, _guard)?;
        Ok(Host(HostHandle::new(host)))
    }

    /// Returns the host API's descriptive name (e.g. "CoreAudio").
    pub fn name(&self) -> &str {
        &self.0.name
    }

    /// Creates and returns the default output device for this host.
    ///
    /// This is the recommended device to use for audio playback.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut host = audiohal::Host::with_default_backend()?;
    /// match host.default_output_device() {
    ///     Ok(device) => println!("Default output device name is {}.", device.name()),
    ///     Err(_) => println!("No devices available."),
    /// };
    /// # audiohal::Result::Ok(())
    /// ```
    ///
    pub fn default_output_device(&mut self) -> Result<Device> {
        let guard = global_lock();
        let device_index = self.0.default_output_device_index(&guard)?;
        crate::portaudio::device::internal::from_device_index(
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
            host_info: RawPtr::dangling(),
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
        self.host_info = RawPtr::new(unsafe { ffi::Pa_GetHostApiInfo(host_index) })
            .ok_or(Error::BackendUnavailable)?;
        self.name = unsafe { std::ffi::CStr::from_ptr(self.host_info.as_ref().unwrap().name) }
            .to_str()
            .or(Err(Error::Unknown("Could not convert host name to UTF-8.")))?
            .to_string();
        Ok(())
    }

    fn default_output_device_index(&self, _guard: &LockGuard) -> Result<i32> {
        let host_device_index = unsafe { self.host_info.as_ref().unwrap() }.defaultOutputDevice;
        if host_device_index == ffi::paNoDevice {
            return Err(Error::NoSuchDevice);
        }
        assert!(host_device_index >= 0);
        let device_index =
            unsafe { ffi::Pa_HostApiDeviceIndexToDeviceIndex(self.host_index, host_device_index) };
        if device_index < 0 {
            return Err(ffi::PaError::from(device_index).as_result().unwrap_err());
        }
        Ok(device_index)
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
    use crate::portaudio::test_prelude::*;

    #[test]
    fn host_is_send() {
        assert_send::<Host>();
    }

    #[test]
    fn creates_default_backend() -> Result<()> {
        let host = Host::with_default_backend()?;
        println!("Host is called {}", host.name());
        Ok(())
    }

    #[test]
    fn handles_invalid_backend() {
        // Pick a backend that we know doesn't exist.
        #[cfg(any(unix, target_os = "macos"))]
        let backend = Backend::Wasapi;
        #[cfg(windows)]
        let backend = Backend::CoreAudio;
        assert_that!(
            &Host::with_backend(backend),
            maybe_err(eq(Error::BackendUnavailable))
        );
    }

    #[test]
    fn internal_handles_invalid_host_index() {
        let _guard = global_lock();
        unsafe { ffi::Pa_Initialize() }.as_result().unwrap();
        let mut host = HostImpl::new();
        assert_eq!(
            host.init_with_pa_host_index(100_000, _guard),
            Err(Error::BackendUnavailable)
        );
    }
}
