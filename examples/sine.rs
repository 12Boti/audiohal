use audiohal;

const SAMPLE_RATE: i32 = 48_000;

fn callback(mut time: f32, buffer: &mut [[f32; 1]]) {
    let secs_per_frame = 1.0 / (SAMPLE_RATE as f32);
    for buf in buffer.iter_mut() {
        buf[0] = (time * 2.0 * std::f32::consts::PI).sin();
        time += secs_per_frame;
        if time >= 1.0 {
            time = 0.0;
        }
    }
}

fn main() {
    let mut host = audiohal::Host::with_default_backend().unwrap();
    let mut device = host.default_output_device().unwrap();

    let mut time = 0.0;
    let mut stream = device
        .open_outstream(audiohal::StreamOptions {
            callback: Box::new(move |buffer| callback(time, buffer)),
            sample_rate: audiohal::SampleRate::Exact(SAMPLE_RATE),
            ..Default::default()
        })
        .unwrap();
    stream.start().unwrap();

    std::thread::park();
}
