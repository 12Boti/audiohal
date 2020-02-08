use lazy_static::lazy_static;
use libportaudio_sys as ffi;
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};

mod device;
mod error;
mod host;
mod stream;
mod stream_options;

// Public API exports.
pub use device::Device;
pub use host::Host;
pub use stream::Stream;

lazy_static! {
    static ref GLOBAL_LOCK: ReentrantMutex<()> = ReentrantMutex::new(());
}

type LockGuard = ReentrantMutexGuard<'static, ()>;

fn global_lock() -> LockGuard {
    GLOBAL_LOCK.lock()
}

impl std::convert::TryFrom<crate::Format> for ffi::PaSampleFormat {
    type Error = crate::error::Error;
    fn try_from(format: crate::Format) -> crate::error::Result<ffi::PaSampleFormat> {
        use crate::error::Error;
        use crate::Format::*;
        use ffi::PaSampleFormat;
        Ok(match format {
            F32 => PaSampleFormat::paFloat32,
            I32 => PaSampleFormat::paInt32,
            I24 => PaSampleFormat::paInt24,
            I16 => PaSampleFormat::paInt16,
            I8 => PaSampleFormat::paInt8,
            U8 => PaSampleFormat::paUInt8,
            _ => return Err(Error::IncompatibleFormat(format)),
        })
    }
}

struct RawPtr<T>(*const T);

impl<T> RawPtr<T> {
    fn dangling() -> RawPtr<T> {
        RawPtr(std::ptr::null())
    }

    fn new(val: *const T) -> Option<RawPtr<T>> {
        if val.is_null() {
            None
        } else {
            Some(RawPtr(val))
        }
    }
}

impl<T> RawPtr<T> {
    unsafe fn as_ref<'a>(&self) -> Option<&'a T> {
        self.0.as_ref()
    }

    fn as_ptr(&self) -> *const T {
        self.0
    }
}

unsafe impl<T> Send for RawPtr<T> {}
unsafe impl<T> Sync for RawPtr<T> {}

#[cfg(test)]
mod test_prelude {
    pub use super::*;
    pub use crate::*;
    pub use galvanic_assert::matchers::variant::*;
    pub use galvanic_assert::matchers::*;

    pub fn is_initialized() -> bool {
        let _guard = global_lock();
        unsafe { ffi::Pa_GetDeviceCount() >= 0 }
    }

    pub fn assert_send<T: Send>() {}
}
