use bip39::Language;
use clap::Parser;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[derive(Parser, Debug)]
#[command(
    about = "Generate secure random 3-word combos from the BIP-39 wordlist"
)]
struct Args {
    /// Number of 3-word combinations to generate
    #[arg(short, long, default_value_t = 20)]
    count: usize,
}

fn main() {
    let args = Args::parse();
    let wordlist = Language::English.word_list();
    let mut rng = StdRng::from_rng(&mut rand::rng());

    let words = (0..args.count)
        .map(|_| {
            (0..3)
                .map(|_| wordlist[rng.random_range(0..wordlist.len())])
                .collect::<Vec<_>>()
                .join("-")
        })
        .collect::<Vec<_>>();

    println!("{}", words.join("\n"));
}
