#![warn(clippy::all)]

pub use bindings::*;
pub use flags::*;

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
mod bindings;

// TODO: Implement this safely.
impl From<i32> for PaErrorCode {
    fn from(code: i32) -> PaErrorCode {
        match code {
            0 | -10000..=-9972 => unsafe { std::mem::transmute(code) },
            _ => panic!(
                "Unexpected error code: {}. Most likely unrecoverable error.",
                code
            ),
        }
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
            const PaClipOff         = 0b01;
            const PaDitherOff       = 0b10;
            const PaNeverDropInput  = 0b100;
            const PaPrimeOutputBuffersUsingStreamCallback = 0b1000;

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
