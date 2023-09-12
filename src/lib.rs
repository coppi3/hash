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

    /// Wrapped with_capacity from `HashMap`
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
    ///
    /// min(cli.workers, cpu_physical_cores)
    ///
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
    /// use hash::HHashMap;
    /// let mut found_hashes = HHashMap::default();
    /// found_hashes.hash_gen(3, 1, 12);
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hashing_correct_one() {
        // 4163
        let true_hash = "95d4362bd3cd4315d0bbe38dfa5d7fb8f0aed5f1a31d98d510907279194e3000";
        let mut gened_hashes = HHashMap::with_capacity(2);
        gened_hashes.hash_gen(3, 1, 1);
        let gened_hash: Vec<&String> = gened_hashes.values().collect();
        assert_eq!(true_hash, gened_hash[0], "{}:{}", true_hash, gened_hash[0]);
    }

    #[test]
    fn hashing_f_correct() {
        let frequency = 1337;
        let mut gened_hashes = HHashMap::with_capacity(frequency);
        gened_hashes.hash_gen(1, frequency, 10);
        let gened_hash: Vec<&String> = gened_hashes.values().collect();
        assert_eq!(
            frequency,
            gened_hash.len(),
            "{}:{}",
            frequency,
            gened_hash.len()
        );
    }

    #[test]
    fn hashing_n3() {
        let mut true_map: HashMap<usize, &str> = HashMap::new();
        true_map.insert(
            4163,
            "95d4362bd3cd4315d0bbe38dfa5d7fb8f0aed5f1a31d98d510907279194e3000",
        );
        true_map.insert(
            11848,
            "cb58074fd7620cd0ff471922fd9df8812f29f302904b15e389fc14570a66f000",
        );
        true_map.insert(
            12843,
            "bb90ff93a3ee9e93c123ebfcd2ca1894e8994fef147ad81f7989eccf83f64000",
        );
        true_map.insert(
            13467,
            "42254207576dd1cfb7d0e4ceb1afded40b5a46c501e738159d8ac10b36039000",
        );
        true_map.insert(
            20215,
            "1f463eb31d6fa7f3a7b37a80f9808814fc05bf10f01a3f653bf369d7603c8000",
        );
        true_map.insert(
            28892,
            "dab12874ecae90c0f05d7d87ed09921b051a586c7321850f6bb5e110bc6e2000",
        );
        let mut gened_hashes = HHashMap::with_capacity(6);
        gened_hashes.hash_gen(3, 6, 1);
        let gened_hash = gened_hashes.into_iter();

        let mut g = Vec::new();
        let mut t = Vec::new();

        _ = gened_hash.map(|(k, v)| g.push((*k, (*v).clone())));
        _ = true_map
            .into_iter()
            .map(|(k, v)| t.push((k, v.to_string())));
        assert_eq!(
            g, t,
            "something went terribly wrong\nt:\n{:?}\ng:\n{:?}",
            t, g
        );
    }
}
