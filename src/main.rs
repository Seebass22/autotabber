use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use find_peaks::PeakFinder;

const BUFSIZE: usize = 1024;
// const BUFSIZE: usize = 2048;
// const BUFSIZE: usize = 32;

fn main() {
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .expect("failed to find input device");

    let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();

    let mut buf = Vec::<f32>::with_capacity(BUFSIZE);
    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        for &sample in data {
            buf.push(sample);
            if buf.len() == BUFSIZE {
                handle_buffer(&buf);
                buf.clear();
            }
        }
    };

    // Build streams.
    println!(
        "Attempting to build both streams with f32 samples and `{:?}`.",
        config
    );
    let input_stream = input_device
        .build_input_stream(&config, input_data_fn, err_fn)
        .unwrap();
    println!("Successfully built streams.");

    // Play the streams.
    input_stream.play().unwrap();

    // Run for 3 seconds before closing.
    println!("Playing for 3 seconds... ");
    std::thread::sleep(std::time::Duration::from_secs(3));
    drop(input_stream);
    println!("Done!");
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

fn handle_buffer(buf: &[f32]) {
    let autoc = autocorrelation(buf);

    // let ps = PeakFinder::new(&autoc).find_peaks();
    // let main = ps[0].middle_position();
    // let second = ps[1].middle_position();
    // let res = (main as i32 - second as i32).abs();

    // println!("{}", res);
    // for i in 0..10 {
    //     print!("{} ", ps[i].middle_position());
    // }
    // println!();

    // println!("{} - {} = {}", ps[1].position);
    // println!("{}", ps[0].right_diff);

    // println!("{:?}", buf);
}

fn autocorrelation<T: cpal::Sample>(signal: &[T]) -> [i32; 3*BUFSIZE] {
    let signal: Vec<i16> = signal.iter().map(|x| x.to_i16()).collect();

    let mut original = [0i16; 3 * BUFSIZE];
    let mut lagged = [0i16; 3 * BUFSIZE];
    let mut res_arr = [0i32; 3 * BUFSIZE];
    let mut res = [0i32; 3 * BUFSIZE];

    // create array with original signal in middle
    for i in 0..BUFSIZE {
        original[BUFSIZE + i] = signal[i];
    }

    for i in 0..(BUFSIZE * 2) {
        lagged.fill(0i16);
        // move lagged signal
        for j in 0..BUFSIZE {
            lagged[i + j] = signal[j];
        }

        // sum
        for j in 0..(BUFSIZE * 3) {
            res_arr[j] = lagged[j] as i32 * original[j] as i32;
        }
        res[i] = res_arr.iter().sum();
    }
    // let mut s = Vec::from(signal);
    // s.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // println!("{:?}", s);
    // println!("{:?}", res);
    // println!("END");
    println!("{:?}", signal);
    res
    // println!("{:?}", lagged);
}

// fn readfile() {
// }
