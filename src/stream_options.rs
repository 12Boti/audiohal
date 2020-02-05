#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    F32,
    I32,
    I24,
    I16,
    I8,
    U8,
}

#[non_exhaustive]
pub enum SampleRate {
    Exact(i32),
    NearestTo(i32),
}

/// Configures the creation of input/output streams.
///
/// This struct sets properties of a stream such as its format, number of channels, sample rate, and
/// user-provided callback. The [`Default`] trait is implemented for `StreamOptions` with common
/// formats and number-of-channel combinations. E.g.:
///
/// ```
/// # use audiohal::*;
/// # fn my_stream_callback(_: &mut [f32]) {}
/// // Creates a StreamOptions with a stereo f32 frame.
/// let options = StreamOptions::<[f32; 2]> {
///     callback: Box::new(my_stream_callback),
///     ..Default::default()
/// };
/// assert_eq!(options.format, Format::F32);
/// assert_eq!(options.n_channels, 2);
/// ```
pub struct StreamOptions<Frame> {
    pub format: Format,
    pub n_channels: i32,

    pub frames_per_buffer: Option<i32>,
    pub sample_rate: Option<SampleRate>,

    pub callback: Box<dyn FnMut(&mut [Frame])>,
}

// Default dummy callback that does nothing.
fn dummy_callback<T>(_: &mut [T]) {}

impl<Frame, Sample> Default for StreamOptions<Frame>
where
    Frame: 'static + sample::Frame<Sample = Sample> + HasDefaultNChannels,
    Sample: sample::Sample + HasDefaultFormat,
{
    fn default() -> StreamOptions<Frame> {
        StreamOptions {
            format: Sample::FORMAT,
            n_channels: Frame::N_CHANNELS,

            frames_per_buffer: None,
            sample_rate: None,

            callback: Box::new(dummy_callback),
        }
    }
}

/// This trait is implemented for primitive types that have a direct [`Format`] equivalent.
pub trait HasDefaultFormat {
    const FORMAT: Format;
}

impl HasDefaultFormat for f32 {
    const FORMAT: Format = Format::F32;
}
impl HasDefaultFormat for i16 {
    const FORMAT: Format = Format::I16;
}

/// This trait is implemented for array primitive types (e.g. array [`sample::Frame`](frames)).
pub trait HasDefaultNChannels {
    const N_CHANNELS: i32;
}

impl<T> HasDefaultNChannels for [T; 1] {
    const N_CHANNELS: i32 = 1;
}
impl<T> HasDefaultNChannels for [T; 2] {
    const N_CHANNELS: i32 = 2;
}
