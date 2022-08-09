use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use find_peaks::PeakFinder;
use realfft::RealFftPlanner;
use std::io;
use std::sync::mpsc::Sender;

pub fn measure_volume(sender: Sender<String>) {
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .expect("failed to find input device");

    let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();

    let mut buf = Vec::<f64>::with_capacity(512);
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
            if buf.len() == 512 {
                let volume = calculate_volume(&buf);
                sender.send(format!("{}\n", volume)).unwrap();
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

fn calculate_volume(buf: &[f64]) -> f64 {
    100.0 * buf.iter().map(|x| x.abs()).sum::<f64>() / buf.len() as f64
}

pub fn run_wav(
    input: String,
    bufsize: usize,
    min_count: u8,
    min_volume: f64,
    key: String,
    sender: Sender<String>,
) {
    let mut reader = hound::WavReader::open(&input).unwrap();
    match reader.spec().sample_format {
        hound::SampleFormat::Float => {
            _run_wav::<f32, _>(&mut reader, bufsize, min_count, min_volume, &key, sender)
        }
        hound::SampleFormat::Int => {
            _run_wav::<i32, _>(&mut reader, bufsize, min_count, min_volume, &key, sender)
        }
    };
}

fn _run_wav<S, R>(
    reader: &mut hound::WavReader<R>,
    bufsize: usize,
    min_count: u8,
    min_volume: f64,
    key: &str,
    sender: Sender<String>,
) where
    f64: From<S>,
    S: hound::Sample,
    R: io::Read,
{
    let mut previous_note = "";
    let mut count = 0;
    let mut notes_printed = 0;

    let mut buf = Vec::<f64>::with_capacity(bufsize);
    for sample in reader.samples::<S>().flatten() {
        buf.push(f64::from(sample));
        if buf.len() == bufsize {
            let note = get_buffer_note(&buf, min_volume, 44100, key);
            handle_note(
                note,
                &mut previous_note,
                &mut count,
                min_count,
                &mut notes_printed,
                &sender,
            );
            buf.clear();
        }
    }
}

pub fn run(
    bufsize: usize,
    min_count: u8,
    full: bool,
    min_volume: f64,
    key: String,
    sender: Sender<String>,
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
                    let c = get_buffer_note(&buf, min_volume, config.sample_rate.0, &key);
                    sender.send(format!("{}\n", c)).unwrap();
                } else {
                    let note = get_buffer_note(&buf, min_volume, config.sample_rate.0, &key);
                    handle_note(
                        note,
                        &mut previous_note,
                        &mut count,
                        min_count,
                        &mut notes_printed,
                        &sender,
                    );
                }
                buf.clear();
            }
        }
    };

    let input_stream = input_device
        .build_input_stream(&config, input_data_fn, err_fn)
        .unwrap();

    input_stream.play().unwrap();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

fn handle_note(
    note: &'static str,
    previous_note: &mut &'static str,
    count: &mut u32,
    min_count: u8,
    notes_printed: &mut u32,
    sender: &Sender<String>,
) {
    if note == *previous_note {
        *count += 1;
    } else {
        *count = 1
    }
    if *count == min_count as u32 && !note.is_empty() {
        sender.send(format!("{} ", note)).unwrap();
        *notes_printed += 1;
        if *notes_printed == 20 {
            *notes_printed = 0;
            sender.send("\n".to_string()).unwrap();
        }
    }
    *previous_note = note;
}

fn get_buffer_note(buf: &[f64], min_volume: f64, sample_rate: u32, key: &str) -> &'static str {
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
    let freq = distance_to_frequency(dist, sample_rate);
    let midi = freq_to_midi(freq);
    midi_to_tab(midi, key)
}

fn freq_to_midi(freq: f64) -> u8 {
    (12.0 * (freq / 440.0).log2() + 69.0).round() as u8
}

fn is_loud_enough(buf: &[f64], min_volume: f64) -> bool {
    let volume = calculate_volume(buf);
    volume > min_volume
}

fn distance_to_frequency(dist: usize, sample_rate: u32) -> f64 {
    sample_rate as f64 / dist as f64
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

fn midi_to_tab(midi: u8, key: &str) -> &'static str {
    let notes_in_order = [
        "1", "-1'", "-1", "1o", "2", "-2''", "-2'", "-2", "-3'''", "-3''", "-3'", "-3", "4", "-4'",
        "-4", "4o", "5", "-5", "5o", "6", "-6'", "-6", "6o", "-7", "7", "-7o", "-8", "8'", "8",
        "-9", "9'", "9", "-9o", "-10", "10''", "10'", "10",
    ];
    let offset = match key {
        "C" => 0,
        "G" => -5,
        "D" => 2,
        "A" => -3,
        "E" => 4,
        "B" => -1,
        "F#" => 6,
        "Db" => 1,
        "Ab" => -4,
        "Eb" => 3,
        "Bb" => -2,
        "F" => 5,
        "LF" => -7,
        "LC" => -12,
        "LD" => -10,
        "HG" => 7,
        _ => {
            panic!()
        }
    };
    let index: isize = midi as isize - 60 - offset;
    if index < 0 || index > notes_in_order.len() as isize - 1 {
        return "";
    }
    notes_in_order[index as usize]
}
