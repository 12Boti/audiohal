#![warn(clippy::all)]
#![warn(clippy::pedantic)]
// #![warn(missing_docs)]
#![doc(deny(intra_link_resolution))]
// TODO: Remove once non_exhaustive is in stable.
#![allow(unreachable_patterns)]

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
pub use error::{Error, Result};
pub use stream_options::{Callback, Format, SampleRate, StreamOptions};

// Exporting backend types.
pub use portaudio::Device;
pub use portaudio::Host;
pub use portaudio::Stream;
