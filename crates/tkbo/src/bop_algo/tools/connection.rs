//! `BOPAlgo_Tools::FillMap` and `BOPAlgo_Tools::MakeBlocks`: connection maps and their chains.
use rustycad_tkernel::ncollection::{HashPolicy, IndexedDataMap, Map};
use rustycad_tkernel::standard::Failure;
use std::fmt::Debug;

/// `NCollection_IndexedDataMap<K, NCollection_List<V>>`: each key's linked values in insertion order.
/// `let mut links = ConnectionMap::with_hasher(DefaultHashPolicy);`.
pub type ConnectionMap<K, V, H> = IndexedDataMap<K, Vec<V>, H>;

/// Appends `value` to the list of `key`, adding the key first when absent
/// (`BOPAlgo_Tools::FillMap(thePB, theF, ...)`); `fill_map_one(&mut links, block, face)`.
pub fn fill_map_one<K: Debug + Clone, V, H: HashPolicy<K>>(
    links: &mut ConnectionMap<K, V, H>,
    key: K,
    value: V,
) {
    if let Some(list) = links.change_seek(&key) {
        list.push(value);
        return;
    }
    links.add(key, vec![value]);
}

/// Links two elements in both directions, first `first -> second` then `second -> first`
/// (`BOPAlgo_Tools::FillMap(n1, n2, ...)`); `fill_map(&mut links, 1, 2)`.
pub fn fill_map<K: Debug + Clone, H: HashPolicy<K>>(
    links: &mut ConnectionMap<K, K, H>,
    first: K,
    second: K,
) {
    fill_map_one(links, first.clone(), second.clone());
    fill_map_one(links, second, first);
}

/// Splits a connection map into chains of connected elements: each chain starts at the first
/// unvisited key and grows breadth-first in list order (`BOPAlgo_Tools::MakeBlocks`).
/// `let chains = make_blocks(&links, DefaultHashPolicy)?;`.
pub fn make_blocks<K: Debug + Clone, H: HashPolicy<K>>(
    links: &ConnectionMap<K, K, H>,
    fence_hasher: H,
) -> Result<Vec<Vec<K>>, Failure> {
    let mut fence: Map<K, H> = Map::with_hasher(fence_hasher, 1);
    let mut chains: Vec<Vec<K>> = Vec::new();
    for index in 1..=links.extent() {
        let start = links.find_key(index)?;
        if !fence.add(start.clone()) {
            continue;
        }
        chains.push(grow_chain(links, start, &mut fence)?);
    }
    Ok(chains)
}

fn grow_chain<K: Debug + Clone, H: HashPolicy<K>>(
    links: &ConnectionMap<K, K, H>,
    start: &K,
    fence: &mut Map<K, H>,
) -> Result<Vec<K>, Failure> {
    let mut chain = vec![start.clone()];
    let mut position = 0;
    while position < chain.len() {
        for linked in links.find_from_key(&chain[position])? {
            if fence.add(linked.clone()) {
                chain.push(linked.clone());
            }
        }
        position += 1;
    }
    Ok(chain)
}
