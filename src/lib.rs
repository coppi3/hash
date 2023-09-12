use sha256::digest;
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::ops::Range;
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HHashMap(pub HashMap<usize, String>);
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
    pub fn hash_gen(&mut self, number: usize, frequency: usize, workers: usize) {
        let hashes = Arc::new(Mutex::new(self.clone()));

        let nc = num_cpus::get_physical();
        println!("Max workers available: {}", nc);
        let workers = std::cmp::min(nc, workers);

        let step = (usize::MAX - 1) / workers;
        println!("w: {}, step: {}", workers, step);

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

    /// inner function for hash_gen

    fn _hash_gen(
        tx: Sender<()>,
        frequency: usize,
        number: usize,
        range: Range<usize>,
        hashes: Arc<Mutex<HHashMap>>,
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
                        hashes.insert(n, digest);
                    }
                };
            }
        }
    }
}
