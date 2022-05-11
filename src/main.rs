use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::RingBuffer;

fn main() {
    println!("Hello, world!");
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("failed to find input device");
    let output_device = host.default_output_device().expect("failed to find output device");

    let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();

    // tweak this
    let latency = 150.0;

    // Create a delay in case the input and output devices aren't synced.
    let latency_frames = (latency / 1_000.0) * config.sample_rate.0 as f32;
    let latency_samples = latency_frames as usize * config.channels as usize;

    // The buffer to share samples
    let ring = RingBuffer::new(latency_samples * 2);
    let (mut producer, mut consumer) = ring.split();

    // Fill the samples with 0.0 equal to the length of the delay.
    for _ in 0..latency_samples {
        // The ring buffer has twice as much space as necessary to add latency here,
        // so this should never fail
        producer.push(0.0).unwrap();
    }

    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let mut output_fell_behind = false;
        for &sample in data {
            if producer.push(sample).is_err() {
                output_fell_behind = true;
            }
        }
        if output_fell_behind {
            eprintln!("output stream fell behind: try increasing latency");
        }
    };

    let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        let mut input_fell_behind = false;
        for sample in data {
            *sample = match consumer.pop() {
                Some(s) => s,
                None => {
                    input_fell_behind = true;
                    0.0
                }
            };
        }
        if input_fell_behind {
            eprintln!("input stream fell behind: try increasing latency");
        }
    };

    // Build streams.
    println!(
        "Attempting to build both streams with f32 samples and `{:?}`.",
        config
    );
    let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn).unwrap();
    let output_stream = output_device.build_output_stream(&config, output_data_fn, err_fn).unwrap();
    println!("Successfully built streams.");

    // Play the streams.
    println!(
        "Starting the input and output streams with `{}` milliseconds of latency.",
        latency
    );
    input_stream.play().unwrap();
    output_stream.play().unwrap();

    // Run for 3 seconds before closing.
    println!("Playing for 3 seconds... ");
    std::thread::sleep(std::time::Duration::from_secs(3));
    drop(input_stream);
    drop(output_stream);
    println!("Done!");
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
fn autocorrelation2(signal: &[f32]) {
    let mut original = [0f32; 3 * BUFSIZE];
    let mut lagged = [0f32; 3 * BUFSIZE];
    let mut res_arr = [0f32; 3 * BUFSIZE];
    let mut res = [0f32; 3 * BUFSIZE];

    // create array with original signal in middle
    for i in 0..BUFSIZE {
        original[BUFSIZE + i] = signal[i];
    }

    for i in 0..(BUFSIZE * 2) {
        lagged.fill(0f32);
        // move lagged signal
        for j in 0..BUFSIZE {
            lagged[i + j] = signal[j];
        }

        // sum
        for j in 0..(BUFSIZE*3) {
            res_arr[j] = lagged[j] * original[j];
        }
        res[i] = res_arr.iter().sum();
    }
    // println!("{:?}", res);
    // println!("{:?}", lagged);
    println!("{:?}", signal);
}

