use autotabber::*;
use clap::Parser;
use std::io;
use std::io::Write;
use std::sync::mpsc;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// print a note every buffer, instead of on note change
    #[clap(long, action, default_value_t = false)]
    full: bool,

    /// measure volume (to set --min-volume)
    #[clap(long, action, default_value_t = false)]
    measure_volume: bool,

    /// number of occurences required to print note
    #[clap(short, long, value_parser, default_value_t = 4)]
    count: u8,

    /// size of the window (in samples) used for pitch detection
    #[clap(short, long, value_parser, default_value_t = 512)]
    window_size: usize,

    /// minimum volume to detect notes
    #[clap(short, long, value_parser, default_value_t = 0.12)]
    min_volume: f64,

    /// harmonica key
    #[clap(short, long, value_parser, default_value_t = String::from("C"))]
    key: String,

    /// read from WAV file instead of microphone
    #[clap(short)]
    input: Option<String>,
}

fn main() {
    let args = Args::parse();
    let keys = [
        "C", "G", "D", "A", "E", "B", "F#", "Db", "Ab", "Eb", "Bb", "F", "LF", "LC", "LD", "HG",
    ];
    if !keys.iter().any(|k| k == &args.key) {
        eprintln!("invalid key. available keys: {:?}", keys);
        std::process::exit(-1);
    }

    let (sender, receiver) = mpsc::channel();

    if args.measure_volume {
        std::thread::spawn(move || {
            measure_volume(sender);
        });
    } else if let Some(input) = args.input {
        std::thread::spawn(move || {
            run_wav(
                input,
                args.window_size,
                args.count,
                args.min_volume,
                args.key,
                sender,
            );
        });
    } else {
        std::thread::spawn(move || {
            run(
                args.window_size,
                args.count,
                args.full,
                args.min_volume,
                args.key,
                sender,
            );
        });
    }

    loop {
        while let Ok(data) = receiver.recv() {
            print!("{}", data);
            io::stdout().flush().unwrap();
        }
    }
}
