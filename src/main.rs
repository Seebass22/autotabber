use find_peaks::PeakFinder;
use std::io;

const BUFSIZE: usize = 1024;

fn run<S, R>(reader: &mut hound::WavReader<R>)
where
    f64: From<S>,
    S: hound::Sample,
    R: io::Read,
{
    let mut buf = Vec::<f64>::with_capacity(BUFSIZE);
    for sample in reader.samples::<S>().flatten() {
        buf.push(f64::from(sample));
        if buf.len() == BUFSIZE {
            handle_buffer(&buf);
            buf.clear();
        }
    }
}

fn main() {
    let mut reader = hound::WavReader::open("C.wav").unwrap();
    match reader.spec().sample_format {
        hound::SampleFormat::Float => run::<f32, _>(&mut reader),
        hound::SampleFormat::Int => run::<i32, _>(&mut reader),
    };
}

fn handle_buffer(buf: &[f64]) {
    let autoc = autocorrelation(buf);
    let ps = PeakFinder::new(&autoc).find_peaks();
    let main = ps[0].middle_position() as isize;
    let second = ps[1].middle_position() as isize;
    let dist = (main - second).abs() as usize;
    let freq = distance_to_frequency(dist);
    println!("{}", freq);
}

fn distance_to_frequency(dist: usize) -> f64 {
    44100.0 / dist as f64
}

fn autocorrelation(signal: &[f64]) -> [f64; 3 * BUFSIZE] {
    let mut original = [0f64; 3 * BUFSIZE];
    let mut lagged = [0f64; 3 * BUFSIZE];
    let mut res = [0f64; 3 * BUFSIZE];

    // create array with original signal in middle
    for i in 0..BUFSIZE {
        original[BUFSIZE + i] = signal[i];
    }

    for i in 0..(BUFSIZE * 2) {
        lagged.fill(0f64);
        // move lagged signal
        for j in 0..BUFSIZE {
            lagged[i + j] = signal[j];
        }

        // sum
        let mut sum = 0.0;
        for j in 0..(BUFSIZE * 3) {
            sum += lagged[j] * original[j];
        }
        res[i] = sum;
    }
    res
}
