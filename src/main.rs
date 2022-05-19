use hound;

use std::env;
use std::io;

/// Compute the RMS of either integers or float samples.
fn compute_rms<S, R>(reader: &mut hound::WavReader<R>) -> f64
where
    f64: From<S>,
    S: hound::Sample,
    R: io::Read,
{
    let sqr_sum = reader.samples::<S>().fold(0.0, |sqr_sum, s| {
        let sample = f64::from(s.unwrap());
        sqr_sum + sample * sample
    });
    (sqr_sum / reader.len() as f64).sqrt()
}

fn main() {
    // Compute the RMS for all files given on the command line.
    for fname in env::args().skip(1) {
        let mut reader = hound::WavReader::open(&fname).unwrap();
        let rms = match reader.spec().sample_format {
            hound::SampleFormat::Float => compute_rms::<f32, _>(&mut reader),
            hound::SampleFormat::Int => compute_rms::<i32, _>(&mut reader),
        };
        println!("{}: {:0.1} ({} samples)", fname, rms, reader.len());
    }
}
