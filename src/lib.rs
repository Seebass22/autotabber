use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::thread;
use std::sync::mpsc;
use find_peaks::PeakFinder;
use realfft::RealFftPlanner;


// good enough for a C harp
const BUFSIZE: usize = 512;

pub fn run() {
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .expect("failed to find input device");

    let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();


    let (tx, rx) = mpsc::channel();
    let mut buf = Vec::<f64>::with_capacity(BUFSIZE);

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
            if buf.len() == BUFSIZE {
                tx.send(buf.clone()).unwrap();
                buf.clear();
            }
        }
    };

    thread::spawn(move || {
        loop {
            let received = rx.recv().unwrap();
            let c = handle_buffer(&received);
            println!("{}", c);
        }
    });

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

    std::thread::sleep(std::time::Duration::from_secs(1000));
    drop(input_stream);
    println!("Done!");
}

fn handle_buffer(buf: &[f64]) -> &'static str {
    let autoc = autocorrelation(buf);
    let ps = PeakFinder::new(&autoc).find_peaks();
    if ps.len() < 2 {
        return "X";
    }
    let main = ps[0].middle_position() as isize;
    let second = ps[1].middle_position() as isize;
    let dist = (main - second).abs() as usize;
    let freq = distance_to_frequency(dist);
    find_note(freq)
}

fn distance_to_frequency(dist: usize) -> f64 {
    44100.0 / dist as f64
}

fn autocorrelation(signal: &[f64]) -> Vec<f64> {
    let length = BUFSIZE*2;

    // make a planner
    let mut real_planner = RealFftPlanner::<f64>::new();

    // create a FFT
    let r2c = real_planner.plan_fft_forward(length);

    let mut indata = signal.to_owned();
    // zero pad signal by factor of 2
    indata.extend_from_slice(&[0f64; BUFSIZE]);
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
    outdata.rotate_right(BUFSIZE);
    outdata
}


fn find_note(pitch: f64) -> &'static str {
    let mut mindist = 10000.0;
    let mut index = 0;
    let mut minindex = 0;
    for n in NOTES.iter() {
        let dist = (pitch - n.0).abs();
        if dist < mindist {
            mindist = dist;
            minindex = index;
        }
        index += 1;
    }
    NOTES[minindex].1
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

const NOTES: [(f64, &'static str); 108] = [
    (16.35, ""),
    (17.32, ""),
    (18.35, ""),
    (19.45, ""),
    (20.60, ""),
    (21.83, ""),
    (23.12, ""),
    (24.50, ""),
    (25.96, ""),
    (27.50, ""),
    (29.14, ""),
    (30.87, ""),
    (32.70, ""),
    (34.65, ""),
    (36.71, ""),
    (38.89, ""),
    (41.20, ""),
    (43.65, ""),
    (46.25, ""),
    (49.00, ""),
    (51.91, ""),
    (55.00, ""),
    (58.27, ""),
    (61.74, ""),
    (65.41, ""),
    (69.30, ""),
    (73.42, ""),
    (77.78, ""),
    (82.41, ""),
    (87.31, ""),
    (92.50, ""),
    (98.00, ""),
    (103.83, ""),
    (110.00, ""),
    (116.54, ""),
    (123.47, ""),
    (130.81, ""),
    (138.59, ""),
    (146.83, ""),
    (155.56, ""),
    (164.81, ""),
    (174.61, ""),
    (185.00, ""),
    (196.00, ""),
    (207.65, ""),
    (220.00, ""),
    (233.08, ""),
    (246.94, ""),
    (261.63, " 1"),
    (277.18, "-1'" ),
    (293.66, "-1" ),
    (311.13, " 1o"),
    (329.63, " 2"),
    (349.23, "-2''" ),
    (369.99, "-2'" ),
    (392.00, "-2" ),
    (415.30, "-3'''" ),
    (440.00, "-3''" ),
    (466.16, "-3'" ),
    (493.88, "-3" ),
    (523.25, " 4"),
    (554.37, "-4'" ),
    (587.33, "-4" ),
    (622.25, " 4o"),
    (659.25, " 5"),
    (698.46, "-5" ),
    (739.99, " 5o"),
    (783.99, " 6"),
    (830.61, "-6'" ),
    (880.00, "-6" ),
    (932.33, " 6o"),
    (987.77, "-7" ),
    (1046.50, " 7"),
    (1108.73, "-7o" ),
    (1174.66, "-8" ),
    (1244.51, " 8'"),
    (1318.51, "-8" ),
    (1396.91, "-9" ),
    (1479.98, " 9'"),
    (1567.98, " 9"),
    (1661.22, "-9o" ),
    (1760.00, "-10" ),
    (1864.66, " 10''"),
    (1975.53, " 10'"),
    (2093.00, " 10"),
    (2217.46, ""),
    (2349.32, ""),
    (2489.02, ""),
    (2637.02, ""),
    (2793.83, ""),
    (2959.96, ""),
    (3135.96, ""),
    (3322.44, ""),
    (3520.00, ""),
    (3729.31, ""),
    (3951.07, ""),
    (4186.01, ""),
    (4434.92, ""),
    (4698.63, ""),
    (4978.03, ""),
    (5274.04, ""),
    (5587.6, ""),
    (5919.9, ""),
    (6271.93, ""),
    (6644.88, ""),
    (7040.00, ""),
    (7458.62, ""),
    (7902.13, ""),
];