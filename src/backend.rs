#[non_exhaustive]
#[derive(Debug)]
pub enum Backend {
    None,
    Jack,
    PulseAudio,
    Alsa,
    CoreAudio,
    Wasapi,
    LinuxFallback,
    Dummy,
}
