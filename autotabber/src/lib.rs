use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use find_peaks::PeakFinder;
use realfft::RealFftPlanner;
use std::io;
use std::io::Write;
use std::sync::mpsc::Sender;

pub fn run(
    bufsize: usize,
    min_count: u8,
    full: bool,
    min_volume: f64,
    sender: Option<Sender<String>>,
) {
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .expect("failed to find input device");

    let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();

    let mut buf = Vec::<f64>::with_capacity(bufsize);

    let mut previous_note = "";
    let mut count = 0;
    let mut notes_printed = 0;

    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let mut skip = true;
        for &sample in data {
            skip = !skip;
            // only use first channel of input
            // skip every 2nd sample
            if skip {
                continue;
            }

            buf.push(f64::from(sample));
            if buf.len() == bufsize {
                if full {
                    let c = get_buffer_note(&buf, min_volume);
                    send_or_print(c, &sender);
                    send_or_print(" \n", &sender);
                } else {
                    let c = get_buffer_note(&buf, min_volume);
                    if c == previous_note {
                        count += 1;
                    } else {
                        count = 1
                    }
                    if count == min_count && !c.is_empty() {
                        send_or_print(c, &sender);
                        send_or_print(" ", &sender);
                        io::stdout().flush().unwrap();
                        notes_printed += 1;
                        if notes_printed == 20 {
                            notes_printed = 0;
                            send_or_print("\n", &sender);
                        }
                    }
                    previous_note = c;
                }
                buf.clear();
            }
        }
    };

    println!(
        "Attempting to build input stream with f32 samples and `{:?}`.",
        config
    );
    let input_stream = input_device
        .build_input_stream(&config, input_data_fn, err_fn)
        .unwrap();
    println!("Successfully built streams.");

    input_stream.play().unwrap();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

fn send_or_print(data: &str, sender: &Option<Sender<String>>) {
    if let Some(sender) = sender {
        // panic (exit) thread if there is no receiver
        sender.send(data.to_string()).unwrap();
    } else {
        print!("{}", data);
    }
}

fn get_buffer_note(buf: &[f64], min_volume: f64) -> &'static str {
    if !is_loud_enough(buf, min_volume) {
        return "";
    }
    let autoc = autocorrelation(buf);
    let ps = PeakFinder::new(&autoc).find_peaks();
    if ps.len() < 2 {
        return "";
    }
    let main = ps[0].middle_position() as isize;
    let second = ps[1].middle_position() as isize;
    let dist = (main - second).abs() as usize;
    let freq = distance_to_frequency(dist);
    let midi = freq_to_midi(freq);
    midi_to_tab(midi)
}

fn freq_to_midi(freq: f64) -> u8 {
    (12.0 * (freq / 440.0).log2() + 69.0).round() as u8
}

fn is_loud_enough(buf: &[f64], min_volume: f64) -> bool {
    let volume: f64 = buf.iter().map(|x| x.abs()).sum();
    volume > min_volume
}

fn distance_to_frequency(dist: usize) -> f64 {
    44100.0 / dist as f64
}

fn autocorrelation(signal: &[f64]) -> Vec<f64> {
    let bufsize = signal.len();
    let length = bufsize * 2;

    // make a planner
    let mut real_planner = RealFftPlanner::<f64>::new();

    // create a FFT
    let r2c = real_planner.plan_fft_forward(length);

    let mut indata = signal.to_owned();
    // zero pad signal by factor of 2
    indata.resize(bufsize * 2, 0f64);
    let mut spectrum = r2c.make_output_vec();

    // Forward transform the input data
    r2c.process(&mut indata, &mut spectrum).unwrap();
    for c in spectrum.iter_mut() {
        *c *= c.conj();
    }

    // create an iFFT and an output vector
    let c2r = real_planner.plan_fft_inverse(length);
    let mut outdata = c2r.make_output_vec();
    assert_eq!(outdata.len(), length);

    c2r.process(&mut spectrum, &mut outdata).unwrap();
    // rotate right so that the peaks match up
    outdata.rotate_right(bufsize);
    outdata
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

fn midi_to_tab(midi: u8) -> &'static str {
    let notes_in_order = [
        "1", "-1'", "-1", "1o", "2", "-2''", "-2'", "-2", "-3'''", "-3''", "-3'", "-3", "4", "-4'",
        "-4", "4o", "5", "-5", "5o", "6", "-6'", "-6", "6o", "-7", "7", "-7o", "-8", "8'", "8",
        "-9", "9'", "9", "-9o", "-10", "10''", "10'", "10",
    ];
    let index: isize = midi as isize - 60;
    if index < 0 || index > notes_in_order.len() as isize - 1 {
        return "";
    }
    notes_in_order[index as usize]
}
