#![warn(clippy::all)]
// #![warn(missing_docs)]
#![doc(deny(intra_link_resolution))]

#[macro_use]
extern crate more_asserts;
#[cfg(test)]
#[macro_use]
extern crate galvanic_assert;

mod backend;
mod error;
mod stream_options;

mod portaudio;

// Exporting public types.
pub use backend::Backend;
pub use error::Result;
pub use stream_options::{Format, SampleRate, StreamOptions};

// Exporting backend types.
pub use portaudio::Device;
pub use portaudio::Host;
pub use portaudio::Stream;
