use clap::Parser;
use hash::HHashMap;

#[derive(Parser)]
#[command(name = "Vanity HashGen")]
#[command(author, version, about, long_about = None)]
struct Cli {
    // Number of zeros in the end of the hash
    #[arg(short = 'N', long)]
    number: usize,
    // Number of hashes to be generated
    #[arg(short = 'F', long)]
    frequency: usize,
    // Number of workers
    #[arg(short = 'W', long)]
    workers: usize,
}

fn main() {
    let cli = Cli::parse();

    println!(
        "n: {}, f: {}, w: {}",
        cli.number, cli.frequency, cli.workers
    );

    let mut found_hashes = HHashMap::with_capacity(cli.frequency);
    found_hashes.hash_gen(cli.number, cli.frequency, cli.workers);
    for (n, hash) in &found_hashes {
        println!("{}, {}", n, hash);
    }
}
