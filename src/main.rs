use find_peaks::PeakFinder;
use hound;

use std::io;
use std::env;
use realfft::RealFftPlanner;

const BUFSIZE: usize = 1024;

fn run<S, R>(reader: &mut hound::WavReader<R>) -> f64
where
    f64: From<S>,
    S: hound::Sample,
    R: io::Read,
{
    let mut buf = Vec::<f64>::with_capacity(BUFSIZE);
    for sample in reader.samples::<S>() {
        if sample.is_ok() {
            buf.push(f64::from(sample.unwrap()));
            if buf.len() == BUFSIZE {
                handle_buffer(&buf);
                buf.clear();
            }
        }
    }
    0.0
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

fn main() {
    for fname in env::args().skip(1) {
        let mut reader = hound::WavReader::open(&fname).unwrap();
        let _pitch = match reader.spec().sample_format {
            hound::SampleFormat::Float => run::<f32, _>(&mut reader),
            hound::SampleFormat::Int => run::<i32, _>(&mut reader),
        };
    }
}
