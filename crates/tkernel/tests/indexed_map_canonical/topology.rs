use rustycad_tkernel::ncollection::{HashPolicy, IndexedMap};
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
pub type Probe = IndexedMap<i32, IntPolicy>;
/// The native row through public APIs: size, modulus, table presence, every index in order and
/// every nonempty bucket chain. The native index table (`myData2`, protected, plan D1 class B6)
/// is allocated and released with the bucket table, so its field repeats the table's.
pub fn topology_row(label: &str, map: &Probe) -> String {
    let table = u8::from(map.buckets().is_some());
    let mut row = format!(
        "{label} {} {} {table} {table} I",
        map.len(),
        map.bucket_count()
    );
    for (key, index) in map.indexed_items() {
        write!(row, "{key}:{index},").unwrap();
    }
    row.push_str(" B");
    append_buckets(&mut row, map);
    row
}
fn append_buckets(row: &mut String, map: &Probe) {
    let Some(table) = map.buckets() else {
        return;
    };
    for bucket in 0..table.len() {
        let keys: Vec<i32> = table.chain(bucket).unwrap().map(|(key, _)| *key).collect();
        if keys.is_empty() {
            continue;
        }
        write!(row, "{bucket}=").unwrap();
        keys.iter().for_each(|key| write!(row, "{key},").unwrap());
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
    added.add(99);
    check(format!("{prefix}add"), &added);
    added.add(99);
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
    replaced.substitute(i, 77).unwrap();
    check(format!("{prefix}sub{i}"), &replaced);
    for j in 1..=source.len() {
        let mut swapped = source.clone();
        swapped.swap(i, j).unwrap();
        check(format!("{prefix}swap{i}-{j}"), &swapped);
    }
}
fn matrix(collision: bool) {
    for modulus in [0, 1, 10, 101] {
        for n in 0..=4 {
            let mut map = Probe::with_hasher(IntPolicy(collision), modulus);
            for key in 1..=n {
                map.add(key);
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
