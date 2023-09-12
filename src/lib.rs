use sha256::digest;
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::ops::Range;
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

/// Wrapper for `HashMap<usize, String>` (`Hashmap<number, Digest>`)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HHashMap(pub HashMap<usize, String>);

// simply for better code readability in the future
impl Deref for HHashMap {
    type Target = HashMap<usize, String>;
    fn deref(&self) -> &HashMap<usize, String> {
        &self.0
    }
}

impl DerefMut for HHashMap {
    fn deref_mut(&mut self) -> &mut HashMap<usize, String> {
        &mut self.0
    }
}

impl<'a> IntoIterator for &'a HHashMap {
    type Item = (&'a usize, &'a String);
    type IntoIter = Iter<'a, usize, String>;

    fn into_iter(self) -> Iter<'a, usize, String> {
        self.iter()
    }
}

impl HHashMap {
    /// Constructor (needed to reimplement because of using a wrapped struct)
    pub fn new() -> Self {
        HHashMap(HashMap::new())
    }

    /// wrapped with_capacity from HashMap
    pub fn with_capacity(capacity: usize) -> Self {
        HHashMap(HashMap::with_capacity(capacity))
    }

    /// Main function to do all the parallell computation of `F` SHA-256 hashes with `N` zeros in the end of the digest.
    /// (mutates an instance of HHashMap it's being called upon)
    ///
    /// Creates `W` workers and splits bruteforcing `SHA-256` between them.
    /// If there are 10 cores available but you are trying to run it with `-W 20` from the
    /// shell, it will use 10 workers.
    /// The real number of workers is:
    /// ```
    /// min(cli.workers, cpu_physical_cores)
    /// ```
    /// This was done according to benchmark analysis in order to get better performance. (check
    /// git README or benchmark.md)
    ///
    /// # Arguments
    ///
    /// * `number` - number of zeros in the end of the digest
    /// * `frequency` - number of digests needed to be found
    /// * `workers` - number of workers to do computation
    ///
    /// # Examples
    /// ```
    /// let mut found_hashes = HHashMap::default();
    /// found_hashes.hash_gen(cli.number, cli.frequency, cli.workers);
    /// ```
    ///
    pub fn hash_gen(&mut self, number: usize, frequency: usize, workers: usize) {
        let hashes = Arc::new(Mutex::new(self.clone()));

        let nc = num_cpus::get_physical();
        println!("Max workers available: {}", nc);
        let workers = std::cmp::min(nc, workers);

        let step = (usize::MAX - 1) / workers;
        println!("Real Workers: {}, Step: {}", workers, step);

        let (tx, rx) = channel();
        let mut p = 1;
        for _ in 0..=workers {
            let (hashes_c, tx) = (Arc::clone(&hashes), tx.clone());
            let range = p..(p + step);
            p += step;
            thread::spawn(move || {
                Self::_hash_gen(tx, frequency, number, range.clone(), hashes_c);
            });
        }
        _ = rx.recv();

        let res = hashes.lock().unwrap();
        let res = res.to_owned();

        *self = res;
    }

    /// Private inner function for `hash_gen` that computes SHA-256 hash and checks if its digest has enough zeros in the end
    ///
    fn _hash_gen(
        tx: Sender<()>,
        frequency: usize,
        number: usize,
        range: Range<usize>,
        hashes: Arc<Mutex<HHashMap>>,
    ) {
        for n in range.clone().into_iter() {
            let digest = digest(n.to_string());
            let t = &digest[digest.len() - number..];
            if t == "0".repeat(number) {
                let mut hashes = hashes.lock().unwrap();
                if hashes.len() == frequency {
                    // exit worker if there are enough hashes already
                    _ = tx.send(());
                    return;
                } else {
                    hashes.insert(n, digest);
                    // for better readability
                    continue;
                }
            };
        }
    }
}
