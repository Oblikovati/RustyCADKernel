//! DataMap and Map safe slot storage against an executable model of the native bucket chains.
//!
//! The model is `NCollection_BaseMap` as written: `NbBuckets + 1` heads, head insertion,
//! `ReSize` that walks buckets `0..=NbBuckets` and prepends each node into the new table, and
//! `UnBind` that unlinks one node. Slot removal moves the last slot into the hole; these
//! scripts make that slot sit in other buckets and mid-chain, so a wrong relink shows up as a
//! different traversal, chain or lookup.
use rustycad_tkernel::ncollection::{DataMap, HashPolicy, Map, next_map_prime};
use std::{cell::RefCell, rc::Rc};

/// Identity hash: keys that differ by the modulus share a bucket.
#[derive(Clone, Copy, Default)]
struct ModuloHash;
impl HashPolicy<i64> for ModuloHash {
    fn hash(&self, key: &i64) -> usize {
        key.unsigned_abs() as usize
    }
    fn equal(&self, first: &i64, second: &i64) -> bool {
        first == second
    }
}

/// Native chains; `None` is the null `Buckets()` table.
struct NativeModel {
    buckets: Option<Vec<Vec<i64>>>,
    modulus: usize,
}

impl NativeModel {
    fn new(modulus: usize) -> Self {
        Self {
            buckets: None,
            modulus,
        }
    }
    fn len(&self) -> usize {
        self.buckets.iter().flatten().map(Vec::len).sum()
    }
    fn bucket(&self, key: i64) -> usize {
        key.unsigned_abs() as usize % self.modulus + 1
    }
    fn traversal(&self) -> Vec<i64> {
        self.buckets.iter().flatten().flatten().copied().collect()
    }
    fn resize(&mut self, extent: usize) {
        let requested = next_map_prime(extent);
        let allocated = self.buckets.is_some();
        if requested <= self.modulus && allocated {
            return;
        }
        let modulus = if requested <= self.modulus {
            self.modulus
        } else {
            requested
        };
        let mut fresh = vec![Vec::new(); modulus + 1];
        for key in self.traversal() {
            fresh[key.unsigned_abs() as usize % modulus + 1].insert(0, key);
        }
        (self.buckets, self.modulus) = (Some(fresh), modulus);
    }
    fn bind(&mut self, key: i64) -> bool {
        let size = self.len();
        if size == 0 || size > self.modulus {
            self.resize(size);
        }
        let bucket = self.bucket(key);
        let chain = &mut self.buckets.as_mut().unwrap()[bucket];
        if chain.contains(&key) {
            return false;
        }
        chain.insert(0, key);
        true
    }
    fn unbind(&mut self, key: i64) -> bool {
        let bucket = self.bucket(key);
        let Some(chain) = self.buckets.as_mut().map(|table| &mut table[bucket]) else {
            return false;
        };
        let Some(index) = chain.iter().position(|&stored| stored == key) else {
            return false;
        };
        chain.remove(index);
        true
    }
    fn clear(&mut self, release: bool) {
        match release {
            true => self.buckets = None,
            false => self.buckets.iter_mut().flatten().for_each(Vec::clear),
        }
    }
}

/// Deterministic script source (64-bit LCG), so failures replay exactly.
struct Script(u64);
impl Script {
    fn next(&mut self, bound: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) % bound
    }
    fn key(&mut self) -> i64 {
        // Multiples of the first two primes collide, so chains grow and slots move mid-chain.
        let base = [101, 1009][self.next(2) as usize];
        (base * self.next(9)) as i64 - 300 + self.next(4) as i64
    }
}

fn data_chains(map: &DataMap<i64, i64, ModuloHash>) -> Option<Vec<Vec<i64>>> {
    let table = map.buckets()?;
    let chain = |bucket| table.chain(bucket).unwrap().map(|(key, _)| *key).collect();
    Some((0..table.len()).map(chain).collect())
}

fn key_chains(map: &Map<i64, ModuloHash>) -> Option<Vec<Vec<i64>>> {
    let table = map.buckets()?;
    let chain = |bucket| table.chain(bucket).unwrap().map(|(key, _)| *key).collect();
    Some((0..table.len()).map(chain).collect())
}

fn assert_data_map_matches(map: &DataMap<i64, i64, ModuloHash>, model: &NativeModel, step: usize) {
    let keys: Vec<i64> = map.items().map(|(key, _)| *key).collect();
    assert_eq!(keys, model.traversal(), "traversal after step {step}");
    assert_eq!(data_chains(map), model.buckets, "chains after step {step}");
    assert_eq!(
        (map.len(), map.bucket_count()),
        (model.len(), model.modulus)
    );
    assert!(map.items().all(|(key, value)| *value == key * 3));
}

fn apply_data_step(
    map: &mut DataMap<i64, i64, ModuloHash>,
    model: &mut NativeModel,
    script: &mut Script,
) {
    let key = script.key();
    match script.next(20) {
        0 => {
            let extent = [0, 150, 1500][script.next(3) as usize];
            map.resize(extent);
            model.resize(extent);
        }
        1 => {
            let release = script.next(2) == 1;
            map.clear(release);
            model.clear(release);
        }
        2..=8 => assert_eq!(map.unbind(&key), model.unbind(key), "unbind {key}"),
        _ => assert_eq!(map.bind(key, key * 3), model.bind(key), "bind {key}"),
    }
}

#[test]
fn data_map_slots_follow_the_native_chains_through_random_scripts() {
    // Miri interprets every step; one short script still covers moves, rehash and clears.
    let (seeds, steps) = if cfg!(miri) { (1, 120) } else { (12, 400) };
    for seed in 1..=seeds {
        let mut script = Script(seed);
        let mut map = DataMap::with_hasher(ModuloHash, 1);
        let mut model = NativeModel::new(1);
        for step in 0..steps {
            apply_data_step(&mut map, &mut model, &mut script);
            assert_data_map_matches(&map, &model, step);
        }
    }
}

#[test]
fn key_only_map_slots_follow_the_native_chains_through_removals() {
    let mut script = Script(99);
    let mut map = Map::with_hasher(ModuloHash, 1);
    let mut model = NativeModel::new(1);
    for step in 0..if cfg!(miri) { 150 } else { 600 } {
        let key = script.key();
        if script.next(3) == 0 {
            assert_eq!(map.remove(&key), model.unbind(key), "remove {key}");
        } else {
            assert_eq!(map.add(key), model.bind(key), "add {key}");
        }
        assert_eq!(map.iter().copied().collect::<Vec<_>>(), model.traversal());
        assert_eq!(key_chains(&map), model.buckets, "chains after step {step}");
    }
}

/// Records destructor order by key.
struct Dropped(i64, Rc<RefCell<Vec<i64>>>);
impl Drop for Dropped {
    fn drop(&mut self) {
        self.1.borrow_mut().push(self.0);
    }
}

#[test]
fn clear_and_drop_destroy_entries_in_bucket_then_chain_order() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut map = DataMap::with_hasher(ModuloHash, 1);
    for key in [5, 106, 3, 207, 104] {
        map.bind(key, Dropped(key, Rc::clone(&log)));
    }
    map.unbind(&5);
    let native: Vec<i64> = map.items().map(|(key, _)| *key).collect();
    log.borrow_mut().clear();
    map.clear(false);
    assert_eq!(*log.borrow(), native);
    for key in [7, 108] {
        map.bind(key, Dropped(key, Rc::clone(&log)));
    }
    log.borrow_mut().clear();
    drop(map);
    assert_eq!(*log.borrow(), [108, 7]);
}
