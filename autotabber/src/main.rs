use autotabber::*;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// print a note every buffer, instead of on note change
    #[clap(long, action, default_value_t = false)]
    full: bool,

    /// number of occurences required to print note
    #[clap(short, long, value_parser, default_value_t = 4)]
    count: u8,

    #[clap(short, long, value_parser, default_value_t = 512)]
    buffer_size: usize,

    /// minimum volume to detect notes
    #[clap(short, long, value_parser, default_value_t = 0.6)]
    min_volume: f64,
}

fn main() {
    let args = Args::parse();
    run(args.buffer_size, args.count, args.full, args.min_volume);
}
