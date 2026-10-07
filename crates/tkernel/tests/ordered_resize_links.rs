//! Native ReSize relinks every chain in old-bucket order and hashes each key once.
//! The `_0` rows of tools/oracle/tkernel_ordered_resize_progress.expected are the native
//! successful rehash; the rows with a throwing hasher are D1 class B3 and are not reproduced.
//! The rows' live-node count in the map arena is protected allocator state (plan D1 class B6):
//! slot storage allocates no per-entry node, so that one field is not compared.
#[allow(dead_code)]
#[path = "support/native_node_counts.rs"]
mod native_node_counts;
use native_node_counts::mask_column;
use rustycad_tkernel::ncollection::*;
use std::cell::{Cell, RefCell};

/// Records hash and destructor events once a phase starts.
struct ResizeTrace {
    hashes: [usize; 3],
    active: Cell<bool>,
    events: RefCell<String>,
}
impl ResizeTrace {
    fn new(hashes: [usize; 3]) -> Self {
        Self {
            hashes,
            active: Cell::new(false),
            events: RefCell::new(String::new()),
        }
    }
    fn note(&self, event: char, id: i32) {
        if self.active.get() {
            self.events.borrow_mut().push_str(&format!("{event}{id},"));
        }
    }
    fn start(&self) {
        self.events.borrow_mut().clear();
        self.active.set(true);
    }
}
struct ResizeKey<'a> {
    id: i32,
    trace: &'a ResizeTrace,
}
impl Drop for ResizeKey<'_> {
    fn drop(&mut self) {
        self.trace.note('D', self.id);
    }
}
struct ResizePolicy<'a>(&'a ResizeTrace);
impl HashPolicy<ResizeKey<'_>> for ResizePolicy<'_> {
    fn hash(&self, key: &ResizeKey<'_>) -> usize {
        self.0.note('H', key.id);
        self.0.hashes[key.id as usize - 1]
    }
    fn equal(&self, left: &ResizeKey<'_>, right: &ResizeKey<'_>) -> bool {
        left.id == right.id
    }
}
const PATTERNS: [(&str, [usize; 3]); 4] = [
    ("same", [0, 0, 0]),
    ("split", [0, 101, 202]),
    ("merge", [0, 1009, 2018]),
    ("mixed", [0, 101, 1009]),
];
/// Index of the B6 live-node count among the space-separated fields of a `_0` row.
const LIVE_NODES_FIELD: usize = 8;
fn native_line(label: &str) -> &'static str {
    include_str!("../../../tools/oracle/tkernel_ordered_resize_progress.expected")
        .lines()
        .find(|line| line.starts_with(&format!("{label} ")))
        .unwrap_or_else(|| panic!("oracle row {label}; expected a native resize row"))
}
/// Every chain of the table head first, as `(bucket, key ids)`.
fn chains<V>(table: &HashBucketTable<'_, ResizeKey<'_>, V>) -> Vec<(usize, Vec<i32>)> {
    (0..table.len())
        .map(|bucket| {
            (
                bucket,
                table
                    .chain(bucket)
                    .unwrap()
                    .map(|(key, _)| key.id)
                    .collect(),
            )
        })
        .filter(|(_, keys): &(usize, Vec<i32>)| !keys.is_empty())
        .collect()
}
/// Bucket graph text `bucket:key,key,;` as the native probe prints it (`-` when empty).
fn graph<V>(table: &HashBucketTable<'_, ResizeKey<'_>, V>) -> String {
    let text: String = chains(table)
        .iter()
        .map(|(bucket, keys)| {
            let keys: String = keys.iter().map(|id| format!("{id},")).collect();
            format!("{bucket}:{keys};")
        })
        .collect();
    if text.is_empty() { "-".into() } else { text }
}
/// Each key's chain successor `id:next,` in key order (0 for the chain end).
fn links<V>(table: &HashBucketTable<'_, ResizeKey<'_>, V>) -> String {
    let mut successors = Vec::new();
    for (_, keys) in chains(table) {
        let nexts = keys.iter().skip(1).copied().chain([0]);
        successors.extend(keys.iter().copied().zip(nexts));
    }
    successors.sort();
    successors
        .iter()
        .map(|(id, next)| format!("{id}:{next},"))
        .collect()
}
fn counts(len: usize, bucket_count: usize) -> String {
    let bucket_bytes = (bucket_count + 1) * size_of::<*const ()>();
    let fields = [0, len, bucket_count, 0, 1, 0, bucket_bytes, 0, 1, 3];
    fields.map(|count| count.to_string()).join(" ")
}
macro_rules! resize_suite {
    ($module:ident, $family:literal, $map:ty, $fill:ident) => {
        mod $module {
            use super::*;
            type Map<'a> = $map;
            fn build<'a>(trace: &'a ResizeTrace, arena: &BaseAllocator) -> Map<'a> {
                let mut map = Map::with_allocator(ResizePolicy(trace), 101, arena.clone());
                for id in [1, 2, 3] {
                    $fill(&mut map, ResizeKey { id, trace });
                }
                map
            }
            fn resize_case(pattern: &str, hashes: [usize; 3]) {
                let arena = BaseAllocator::accumulating(4096).unwrap();
                let trace = ResizeTrace::new(hashes);
                let mut map = build(&trace, &arena);
                let old = map.buckets().map(|table| table.identity());
                trace.start();
                map.resize(1000);
                let table = map.buckets().unwrap();
                assert_ne!(Some(table.identity()), old);
                let label = format!("{}_{}_0", $family, pattern);
                let snapshot = graph(&table);
                let row = format!(
                    "{label} {} {} {snapshot} {snapshot} {}",
                    counts(map.len(), map.bucket_count()),
                    links(&table),
                    trace.events.borrow()
                );
                assert_eq!(
                    mask_column(&row, LIVE_NODES_FIELD),
                    mask_column(native_line(&label), LIVE_NODES_FIELD)
                );
                trace.events.borrow_mut().clear();
                map.clear(false);
                let clear = format!("{label}_clear defined 0 0 {}", trace.events.borrow());
                assert_eq!(clear, native_line(&format!("{label}_clear")));
                trace.active.set(false);
            }
            #[test]
            fn successful_resize_matches_native_links_and_drop_order() {
                for (pattern, hashes) in PATTERNS {
                    resize_case(pattern, hashes);
                }
            }
            #[test]
            fn no_op_resize_never_invokes_the_hasher() {
                let trace = ResizeTrace::new([0, 0, 0]);
                let mut map = build(&trace, &BaseAllocator::fresh_heap());
                let old = map.buckets().map(|table| table.identity());
                trace.start();
                map.resize(0);
                assert_eq!(map.buckets().map(|table| table.identity()), old);
                assert!(trace.events.borrow().is_empty());
                trace.active.set(false);
            }
            #[test]
            fn empty_resize_allocates_without_invoking_the_hasher() {
                let trace = ResizeTrace::new([0, 0, 0]);
                let arena = BaseAllocator::fresh_heap();
                let mut map = Map::with_allocator(ResizePolicy(&trace), 13, arena);
                trace.start();
                map.resize(0);
                assert!(map.buckets().is_some());
                assert!(trace.events.borrow().is_empty());
                trace.active.set(false);
            }
        }
    };
}
fn add_set<'a>(map: &mut OrderedMap<ResizeKey<'a>, ResizePolicy<'a>>, key: ResizeKey<'a>) {
    map.add(key);
}
fn bind_data<'a>(
    map: &mut OrderedDataMap<ResizeKey<'a>, i32, ResizePolicy<'a>>,
    key: ResizeKey<'a>,
) {
    let value = key.id * 10;
    map.bind(key, value);
}
resize_suite!(
    set,
    "set",
    OrderedMap<ResizeKey<'a>, ResizePolicy<'a>>,
    add_set
);
resize_suite!(
    data,
    "data",
    OrderedDataMap<ResizeKey<'a>, i32, ResizePolicy<'a>>,
    bind_data
);
