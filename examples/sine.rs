///! This example plays a sine wave (which you'll hear as just a simple tone) with the frequency
///! [`WAVE_FREQUENCY`].
use audiohal;

/// The frequency of our sine wave. In this case, 300Hz.
const WAVE_FREQUENCY: f64 = 300.0;
/// Define our sample rate, or how many samples there are per second.
const SAMPLE_RATE: i32 = 48_000;

// The wave's angular frequency (2*pi*f).
const _ANGULAR_FREQUENCY: f64 = 2.0 * std::f64::consts::PI * WAVE_FREQUENCY;

/// This is the callback that gets called every time the audio library needs samples.
fn outstream_callback(time: &mut f64, buffer: &mut [[f32; 1]]) {
    // How long each sample "lasts" - or how much to advance our "time" by each step. We could also
    // just count how many samples we've written, and divide by the sample rate.
    let secs_per_frame = 1.0 / (SAMPLE_RATE as f64);
    for buf in buffer.iter_mut() {
        buf[0] = (*time * _ANGULAR_FREQUENCY).sin() as f32;
        *time += secs_per_frame;
        // Reset the time back to 0.
        if *time >= 1.0 {
            *time = 0.0;
        }
    }
}

fn main() -> Result<(), audiohal::Error> {
    // We connect to the default host, and with the default output device. This is usually the
    // user's current audio playback device.
    let mut host = audiohal::Host::with_default_backend()?;
    let mut device = host.default_output_device()?;
    println!(
        "Connected to host {} with device {}.",
        host.name(),
        device.name()
    );
    let mut time = 0.0;
    // Let's open an output stream that's fed by our outstream_callback.
    let mut stream = device.open_outstream(audiohal::StreamOptions {
        callback: Box::new(move |buffer| outstream_callback(&mut time, buffer)),
        sample_rate: audiohal::SampleRate::Exact(SAMPLE_RATE),
        ..Default::default()
    })?;
    // Start the stream (the stream will not play until we do this).
    stream.start()?;
    // Play the sound for a few seconds then exit.
    std::thread::sleep(std::time::Duration::from_secs(5));
    Ok(())
}
