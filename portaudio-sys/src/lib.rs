#![warn(clippy::all)]

use std::os::raw::c_int;

pub use bindings::*;
pub use flags::*;

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
mod bindings;

impl From<PaError> for Result<c_int, PaErrorCode> {
    fn from(error: PaError) -> Result<c_int, PaErrorCode> {
        if error.0 >= 0 {
            Ok(error.0)
        } else if let -10000..=-9972 = error.0 {
            Err(unsafe { std::mem::transmute(error.0) }) // TODO: Make safer.
        } else {
            panic!(
                "Unexpected error code {}. Likely undefined behavior.",
                error.0
            )
        }
    }
}

impl From<c_int> for PaError {
    fn from(val: c_int) -> PaError {
        PaError(val)
    }
}

#[allow(non_upper_case_globals)]
mod flags {
    use bitflags::bitflags;

    pub const paNoDevice: i32 = -1;

    bitflags! {
        #[repr(transparent)]
        pub struct PaStreamFlags: std::os::raw::c_ulong {
            const PaNoFlag          = 0b00;
            /// Disable default clipping of out of range samples.
            const PaClipOff         = 0b01;
            /// Disable default dithering.
            const PaDitherOff       = 0b10;
            /// Flag requests that where possible a full duplex stream will not discard
            /// overflowed input samples without calling the stream callback. This flag is
            /// only valid for full duplex callback streams and only when used in combination
            /// with the paFramesPerBufferUnspecified (0) framesPerBuffer parameter. Using
            /// this flag incorrectly results in a paInvalidFlag error being returned from
            /// Pa_OpenStream and Pa_OpenDefaultStream.
            const PaNeverDropInput  = 0b100;
            /// Call the stream callback to fill initial output buffers, rather than the
            /// default behavior of priming the buffers with zeros (silence). This flag has
            /// no effect for input-only and blocking read/write streams.
            const PaPrimeOutputBuffersUsingStreamCallback = 0b1000;
            /// A mask specifying the platform specific bits.
            const PaPlatformSpecificFlags = 0xFFFF_0000;
        }
    }

    bitflags! {
        #[repr(transparent)]
        pub struct PaStreamCallbackFlags: std::os::raw::c_ulong {
            const PaInputUnderflow  = 0x0000_0001;
            const PaInputOverflow   = 0x0000_0002;
            const PaOutputUnderflow = 0x0000_0004;
            const PaOutputOverflow  = 0x0000_0008;
            const PaPrimingOutput   = 0x0000_0010;
        }
    }

    bitflags! {
        /// A type used to specify one or more sample formats. Each value indicates
        /// a possible format for sound data passed to and from the stream callback,
        /// Pa_ReadStream and Pa_WriteStream.
        ///
        /// The standard formats paFloat32, paInt16, paInt32, paInt24, paInt8
        /// and aUInt8 are usually implemented by all implementations.
        ///
        /// The floating point representation (paFloat32) uses +1.0 and -1.0 as the
        /// maximum and minimum respectively.
        ///
        /// paUInt8 is an unsigned 8 bit format where 128 is considered "ground"
        ///
        /// The paNonInterleaved flag indicates that audio data is passed as an array
        /// of pointers to separate buffers, one buffer for each channel. Usually,
        /// when this flag is not used, audio data is passed as a single buffer with
        /// all channels interleaved.
        #[repr(transparent)]
        pub struct PaSampleFormat: std::os::raw::c_ulong {
            const paFloat32 = 0x0000_0001;
            const paInt32   = 0x0000_0002;
            const paInt24   = 0x0000_0004;
            const paInt16   = 0x0000_0008;
            const paInt8    = 0x0000_0010;
            const paUInt8   = 0x0000_0020;

            const paCustomFormat    = 0x0001_0000;
            const paNonInterleaved  = 0x8000_0000;
        }
    }
}
