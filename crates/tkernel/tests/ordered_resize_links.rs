//! Native ReSize relinks every chain in old-bucket order and hashes each key once.
//! The `_0` rows of tools/oracle/tkernel_ordered_resize_progress.expected are the native
//! successful rehash; the rows with a throwing hasher are D1 class B3 and are not reproduced.
#![allow(unsafe_code)]
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
fn active(arena: &BaseAllocator) -> i32 {
    arena
        .accumulating_chain()
        .iter()
        .map(|block| block.alloc_count)
        .sum()
}
fn native_line(label: &str) -> &'static str {
    include_str!("../../../tools/oracle/tkernel_ordered_resize_progress.expected")
        .lines()
        .find(|line| line.starts_with(&format!("{label} ")))
        .unwrap_or_else(|| panic!("oracle row {label}; expected a native resize row"))
}
/// Bucket graph text `bucket:key,key,;` as the native probe prints it (`-` when empty).
unsafe fn graph(
    buckets: *const *mut MapLinkNode,
    count: usize,
    key: unsafe fn(*mut MapLinkNode) -> i32,
) -> String {
    let mut text = String::new();
    for bucket in 0..=count {
        let mut node = unsafe { *buckets.add(bucket) };
        if node.is_null() {
            continue;
        }
        text.push_str(&format!("{bucket}:"));
        while !node.is_null() {
            text.push_str(&format!("{},", unsafe { key(node) }));
            node = unsafe { (*node).next() };
        }
        text.push(';');
    }
    if text.is_empty() { "-".into() } else { text }
}
macro_rules! resize_suite {
    ($module:ident, $family:literal, $map:ty, $node:ty, $fill:ident) => {
        mod $module {
            use super::*;
            type Map<'a> = $map;
            type Node<'a> = $node;
            fn build<'a>(trace: &'a ResizeTrace, arena: &BaseAllocator) -> Map<'a> {
                let mut map = Map::with_allocator(ResizePolicy(trace), 101, arena.clone());
                for id in [1, 2, 3] {
                    $fill(&mut map, ResizeKey { id, trace });
                }
                map
            }
            unsafe fn key(node: *mut MapLinkNode) -> i32 {
                unsafe { (*node.cast::<Node<'_>>()).key().id }
            }
            fn map_nodes<'a>(map: &Map<'a>) -> Vec<*mut Node<'a>> {
                let mut nodes = Vec::new();
                let mut node = map.basic_first_node();
                while !node.is_null() {
                    nodes.push(node);
                    node = unsafe { (*node).order_next() };
                }
                nodes
            }
            /// Each node's chain successor `id:next,` in key order (0 for the chain end).
            fn links(nodes: &[*mut Node<'_>]) -> String {
                let mut ordered = nodes.to_vec();
                ordered.sort_by_key(|node| unsafe { (**node).key().id });
                ordered
                    .iter()
                    .map(|&node| unsafe {
                        let next = (*node).link().next();
                        let next = if next.is_null() { 0 } else { key(next) };
                        format!("{}:{next},", (*node).key().id)
                    })
                    .collect()
            }
            fn counts(map: &Map<'_>) -> String {
                let bucket_bytes = (map.bucket_count() + 1) * size_of::<*mut MapLinkNode>();
                let active = active(map.allocator()) as usize;
                let fields = [
                    0,
                    map.len(),
                    map.bucket_count(),
                    0,
                    1,
                    0,
                    bucket_bytes,
                    active,
                    1,
                    3,
                ];
                fields.map(|count| count.to_string()).join(" ")
            }
            fn resize_case(pattern: &str, hashes: [usize; 3]) {
                let arena = BaseAllocator::accumulating(4096).unwrap();
                let trace = ResizeTrace::new(hashes);
                let mut map = build(&trace, &arena);
                let old = map.buckets();
                let nodes = map_nodes(&map);
                trace.start();
                map.resize(1000);
                assert_ne!(map.buckets(), old);
                let label = format!("{}_{}_0", $family, pattern);
                let snapshot = unsafe { graph(map.buckets(), map.bucket_count(), key) };
                let row = format!(
                    "{label} {} {} {snapshot} {snapshot} {}",
                    counts(&map),
                    links(&nodes),
                    trace.events.borrow()
                );
                assert_eq!(row, native_line(&label));
                trace.events.borrow_mut().clear();
                map.clear(false);
                let clear = format!(
                    "{label}_clear defined {} 0 {}",
                    active(&arena),
                    trace.events.borrow()
                );
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
                let old = map.buckets();
                trace.start();
                map.resize(0);
                assert_eq!(map.buckets(), old);
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
                assert!(!map.buckets().is_null());
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
    OrderedMapNode<ResizeKey<'a>>,
    add_set
);
resize_suite!(
    data,
    "data",
    OrderedDataMap<ResizeKey<'a>, i32, ResizePolicy<'a>>,
    OrderedDataMapNode<ResizeKey<'a>, i32>,
    bind_data
);
