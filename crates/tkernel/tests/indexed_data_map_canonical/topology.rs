use rustycad_tkernel::ncollection::{HashPolicy, IndexedDataMap, IndexedDataMapNode};
use std::fmt::Write;
#[derive(Clone)]
pub struct IntPolicy(pub bool);
impl HashPolicy<i32> for IntPolicy {
    fn hash(&self, key: &i32) -> usize {
        if self.0 { 0 } else { *key as usize }
    }
    fn equal(&self, a: &i32, b: &i32) -> bool {
        a == b
    }
}
pub type Probe = IndexedDataMap<i32, i32, IntPolicy>;
fn nodes(map: &Probe) -> std::collections::BTreeMap<usize, &IndexedDataMapNode<i32, i32>> {
    (1..=map.len())
        .map(|i| {
            let node = map.native_node(i).unwrap();
            (std::ptr::from_ref(node).addr(), node)
        })
        .collect()
}
pub fn topology_row(label: &str, map: &Probe) -> String {
    let state = map.observer().snapshot();
    let mut row = row_header(label, &state);
    for i in 1..=map.len() {
        let node = map.native_node(i).unwrap();
        assert_eq!(state.indices[i - 1], std::ptr::from_ref(node).addr());
        append_index(&mut row, node);
    }
    row.push_str(" B");
    append_buckets(&mut row, map, &state.buckets);
    assert!(
        state
            .indices
            .iter()
            .skip(map.len())
            .all(|address| *address == 0)
    );
    row
}
fn append_buckets(row: &mut String, map: &Probe, heads: &[usize]) {
    let nodes = nodes(map);
    for (bucket, &head) in heads.iter().enumerate().filter(|(_, head)| **head != 0) {
        write!(row, "{bucket}=").unwrap();
        let mut address = head;
        while address != 0 {
            let node = nodes[&address];
            write!(row, "{},", node.key()).unwrap();
            address = node.link().next().addr();
        }
        row.push(';');
    }
}
fn check(label: String, map: &Probe) {
    super::native_row(&label, &topology_row(&label, map));
}
fn variants(source: &Probe, prefix: &str) {
    let mut resized = source.clone();
    resized.resize(200);
    check(format!("{prefix}grow"), &resized);
    let mut cleared = source.clone();
    cleared.clear(false);
    check(format!("{prefix}keep"), &cleared);
    cleared.clear(true);
    check(format!("{prefix}release"), &cleared);
    let mut added = source.clone();
    added.add(99, 990);
    check(format!("{prefix}add"), &added);
    added.add(99, 990);
    check(format!("{prefix}hit"), &added);
    for i in 1..=source.len() {
        indexed_variants(source, prefix, i);
    }
}
fn indexed_variants(source: &Probe, prefix: &str, i: usize) {
    let mut removed = source.clone();
    removed.remove_from_index(i).unwrap();
    check(format!("{prefix}remove{i}"), &removed);
    let mut replaced = source.clone();
    replaced.substitute(i, 77, 770).unwrap();
    check(format!("{prefix}sub{i}"), &replaced);
    for j in 1..=source.len() {
        let mut swapped = source.clone();
        swapped.swap(i, j).unwrap();
        check(format!("{prefix}swap{i}-{j}"), &swapped);
    }
}
fn matrix(collision: bool) {
    for modulus in [0, 101] {
        for n in 0..=4 {
            let mut map = Probe::with_hasher(IntPolicy(collision), modulus);
            for key in 1..=n {
                map.add(key, key * 10);
            }
            let prefix = format!("{}-{modulus}-{n}-", u8::from(collision));
            check(format!("{prefix}initial"), &map);
            variants(&map, &prefix);
        }
    }
}
#[test]
fn all_native_distinct_bucket_transitions() {
    matrix(false);
}
#[test]
fn all_native_collision_transitions() {
    matrix(true);
}

fn row_header(label: &str, state: &rustycad_tkernel::ncollection::IndexedMapObservation) -> String {
    format!(
        "{label} {} {} {} {} I",
        state.size,
        state.bucket_count,
        u8::from(state.bucket_table != 0),
        u8::from(state.index_table != 0)
    )
}

fn append_index(row: &mut String, node: &IndexedDataMapNode<i32, i32>) {
    write!(
        row,
        "{}:{}:{},",
        node.key(),
        node.value(),
        node.index_value()
    )
    .unwrap();
}
