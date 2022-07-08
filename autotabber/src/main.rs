use autotabber::*;
use clap::Parser;

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

    #[clap(short, long, value_parser, default_value_t = 512)]
    buffer_size: usize,

    /// minimum volume to detect notes
    #[clap(short, long, value_parser, default_value_t = 0.12)]
    min_volume: f64,

    /// harmonica key
    #[clap(short, long, value_parser, default_value_t = String::from("C"))]
    key: String,
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
    if args.measure_volume {
        measure_volume(None);
    }
    run(
        args.buffer_size,
        args.count,
        args.full,
        args.min_volume,
        args.key,
        None,
    );
}
