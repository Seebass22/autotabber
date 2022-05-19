use hound;

use std::env;
use std::io;

const BUFSIZE : usize = 32;

/// Compute the RMS of either integers or float samples.
fn compute_rms<S, R>(reader: &mut hound::WavReader<R>) -> f64
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
    println!("{:?}", buf);
}

fn main() {
    let mut reader = hound::WavReader::open("C.wav").unwrap();
    let pitch = match reader.spec().sample_format {
        hound::SampleFormat::Float => compute_rms::<f32, _>(&mut reader),
        hound::SampleFormat::Int => compute_rms::<i32, _>(&mut reader),
    };
    println!("pitch: {}", pitch);
}
