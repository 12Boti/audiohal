#![warn(clippy::all)]
// #![warn(missing_docs)]
#![doc(deny(intra_link_resolution))]

#[macro_use]
extern crate more_asserts;

pub mod backend;
pub mod error;
pub mod stream;
mod stream_options;

mod portaudio;

// Exporting public types.
pub use error::Result;
pub use portaudio::Host;
pub use stream_options::{Format, SampleRate, StreamOptions};
