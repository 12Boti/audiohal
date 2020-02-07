#[non_exhaustive]
#[derive(Debug)]
pub enum Backend {
    None,
    Jack,
    Alsa,
    CoreAudio,
    Wasapi,
    LinuxFallback,
    Dummy,
}
