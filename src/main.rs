use clap::Parser;
use sha256::{digest, try_digest};
use std::ops::Range;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

const NUMBER_STEP: usize = 500;

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
#[derive(Clone, Debug)]
struct Hash {
    n: usize,
    digest: String,
}
#[derive(Debug)]
enum GenErr {
    NotFound,
}

fn main() {
    let cli = Cli::parse();

    println!(
        "n: {}, f: {}, w: {}",
        cli.number, cli.frequency, cli.workers
    );

    let found_hashes = hash_gen(cli.number, cli.frequency, cli.workers);
    for hash in &found_hashes {
        println!("{}, {}", hash.n, hash.digest);
        // println!("{:?}", hash);
    }
}

fn hash_gen(number: usize, frequency: usize, workers: usize) -> Vec<Hash> {
    let hashes = Arc::new(Mutex::new(Vec::with_capacity(frequency)));

    let nc = num_cpus::get_physical();
    println!("Max workers available: {}", nc);
    let workers = std::cmp::min(num_cpus::get(), workers);

    let step = (usize::MAX - 1) / workers;
    println!("w: {}, step: {}", workers, step);

    let (tx, rx) = channel();
    let mut worker_handles = Vec::with_capacity(workers);
    let mut p = 1;
    for _ in 0..=workers {
        let (hashes_c, tx) = (Arc::clone(&hashes), tx.clone());
        let range = p..(p + step);
        // dbg!(&range);
        p += step;
        let join_handle = thread::spawn(move || {
            _generate_hash(tx, frequency, number, range.clone(), hashes_c);
        });
        worker_handles.push(join_handle);
    }
    _ = rx.recv();

    let res = hashes.lock().unwrap();
    res.to_vec()
}
// fn generate_hash(number: usize, frequency: usize) -> Vec<Hash> {
//     let mut found_hashes = vec![];
//     let mut p = 1;
//     let t = 0..=5;
//     for _ in 0..frequency {
//         let hash = _generate_hash(number, p);
//         p = hash.n + 1;
//         found_hashes.push(hash);
//     }
//     found_hashes
// }

fn _generate_hash(
    tx: Sender<()>,
    frequency: usize,
    number: usize,
    range: Range<usize>,
    hashes: Arc<Mutex<Vec<Hash>>>,
) {
    loop {
        for n in range.clone().into_iter() {
            let digest = digest(n.to_string());
            let t = &digest[digest.len() - number..];
            if t == "0".repeat(number) {
                let mut hashes = hashes.lock().unwrap();
                if hashes.len() == frequency {
                    _ = tx.send(());
                } else {
                    hashes.push(Hash { n, digest });
                }
            };
        }
    }
}
