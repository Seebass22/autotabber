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
    let note = find_note(freq);
    println!("{}", note);
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

fn find_note(pitch: f64) -> &'static str {
    let mut mindist = 10000.0;
    let mut index = 0;
    let mut minindex = 0;
    for n in notes.iter() {
        let dist = (pitch - n.0).abs();
        if dist < mindist {
            mindist = dist;
            minindex = index;
        }
        index += 1;
    }
    notes[minindex].1
}

const notes: [(f64, &'static str); 108] = [
    (16.35, "C_0"),
    (17.32, "CS_0"),
    (18.35, "D_0"),
    (19.45, "DS_0"),
    (20.60, "E_0"),
    (21.83, "F_0"),
    (23.12, "FS_0"),
    (24.50, "G_0"),
    (25.96, "GS_0"),
    (27.50, "A_0"),
    (29.14, "AS_0"),
    (30.87, "B_0"),
    (32.70, "C_1"),
    (34.65, "CS_1"),
    (36.71, "D_1"),
    (38.89, "DS_1"),
    (41.20, "E_1"),
    (43.65, "F_1"),
    (46.25, "FS_1"),
    (49.00, "G_1"),
    (51.91, "GS_1"),
    (55.00, "A_1"),
    (58.27, "AS_1"),
    (61.74, "B_1"),
    (65.41, "C_2"),
    (69.30, "CS_2"),
    (73.42, "D_2"),
    (77.78, "DS_2"),
    (82.41, "E_2"),
    (87.31, "F_2"),
    (92.50, "FS_2"),
    (98.00, "G_2"),
    (103.83, "GS_2"),
    (110.00, "A_2"),
    (116.54, "AS_2"),
    (123.47, "B_2"),
    (130.81, "C_3"),
    (138.59, "CS_3"),
    (146.83, "D_3"),
    (155.56, "DS_3"),
    (164.81, "E_3"),
    (174.61, "F_3"),
    (185.00, "FS_3"),
    (196.00, "G_3"),
    (207.65, "GS_3"),
    (220.00, "A_3"),
    (233.08, "AS_3"),
    (246.94, "B_3"),
    (261.63, "1"),
    (277.18, "-1'4"),
    (293.66, "-1"),
    (311.13, "1o"),
    (329.63, "2"),
    (349.23, "-2''"),
    (369.99, "-2'"),
    (392.00, "-2"),
    (415.30, "-3'''"),
    (440.00, "-3''"),
    (466.16, "-3'"),
    (493.88, "-3"),
    (523.25, "4"),
    (554.37, "-4'"),
    (587.33, "-4"),
    (622.25, "4o"),
    (659.25, "5"),
    (698.46, "-5"),
    (739.99, "5o"),
    (783.99, "6"),
    (830.61, "-6'"),
    (880.00, "-6"),
    (932.33, "6o"),
    (987.77, "-7"),
    (1046.50, "7"),
    (1108.73, "-7o"),
    (1174.66, "-8"),
    (1244.51, "8'"),
    (1318.51, "-8"),
    (1396.91, "-9"),
    (1479.98, "9'"),
    (1567.98, "9"),
    (1661.22, "-9o"),
    (1760.00, "-10"),
    (1864.66, "10''"),
    (1975.53, "10'"),
    (2093.00, "10"),
    (2217.46, "CS_7"),
    (2349.32, "D_7"),
    (2489.02, "DS_7"),
    (2637.02, "E_7"),
    (2793.83, "F_7"),
    (2959.96, "FS_7"),
    (3135.96, "G_7"),
    (3322.44, "GS_7"),
    (3520.00, "A_7"),
    (3729.31, "AS_7"),
    (3951.07, "B_7"),
    (4186.01, "C_8"),
    (4434.92, "CS_8"),
    (4698.63, "D_8"),
    (4978.03, "DS_8"),
    (5274.04, "E_8"),
    (5587.6, "F_8"),
    (5919.9, "FS_8"),
    (6271.93, "G_8"),
    (6644.88, "GS_8"),
    (7040.00, "A_8"),
    (7458.62, "AS_8"),
    (7902.13, "B_8"),
];
