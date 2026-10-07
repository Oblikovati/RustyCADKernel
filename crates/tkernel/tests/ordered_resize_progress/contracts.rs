use super::fixture::*;
use rustycad_tkernel::ncollection::*;
use std::panic::{AssertUnwindSafe, catch_unwind};
fn fill_set<'a>(map: &mut OrderedMap<ResizeKey<'a>, ResizePolicy<'a>>, trace: &'a ResizeTrace) {
    for id in [1, 2, 3] {
        map.add(ResizeKey { id, trace });
    }
}
fn fill_data<'a>(
    map: &mut OrderedDataMap<ResizeKey<'a>, i32, ResizePolicy<'a>>,
    trace: &'a ResizeTrace,
) {
    for id in [1, 2, 3] {
        map.bind(ResizeKey { id, trace }, id * 10);
    }
}
macro_rules! progress_suite {
    ($module:ident,$family:literal,$map:ty,$token:ty,$node:ty,$fill:ident) => {
        mod $module {
            use super::*;
            type Map<'a> = $map;
            type Token<'a> = $token;
            type Node<'a> = $node;
            fn build<'a>(trace: &'a ResizeTrace, arena: &BaseAllocator) -> Map<'a> {
                let mut map = Map::with_allocator(ResizePolicy(trace), 101, arena.clone());
                $fill(&mut map, trace);
                map
            }
            unsafe fn key(node: *mut MapLinkNode) -> i32 {
                unsafe { (*node.cast::<Node<'_>>()).key().id }
            }
            fn interrupted<'a>(map: Map<'a>, trace: &ResizeTrace, rejected: usize) -> Token<'a> {
                trace.prepare(rejected);
                match map.resize_with_progress(1000) {
                    Err(token) => token,
                    Ok(_) => panic!(
                        "resize returned success at rejected hash {rejected}; expected interruption"
                    ),
                }
            }
            fn links(nodes: &[*mut Node<'_>]) -> String {
                let mut ordered = nodes.to_vec();
                ordered.sort_by_key(|node| unsafe { (**node).key().id });
                ordered
                    .iter()
                    .map(|&node| unsafe {
                        format!(
                            "{}:{},",
                            (*node).key().id,
                            if (*node).link().next().is_null() {
                                0
                            } else {
                                key((*node).link().next())
                            }
                        )
                    })
                    .collect()
            }
            fn snapshot_counts(token: &Token<'_>, current: &GraphSnapshot) -> String {
                let counts = [
                    1,
                    token.len(),
                    token.bucket_count(),
                    1,
                    1,
                    0,
                    (token.replacement_bucket_count() + 1) * size_of::<*mut MapLinkNode>(),
                    active(token.allocator()) as usize,
                    usize::from(current.unique),
                    current.reached.len(),
                ];
                counts.map(|count| count.to_string()).join(" ")
            }
            fn verify_snapshot(
                label: &str,
                token: &Token<'_>,
                trace: &ResizeTrace,
            ) -> GraphSnapshot {
                let current = unsafe { graph(token.buckets(), token.bucket_count(), key) };
                let pending = pending_graph(token);
                let counts = snapshot_counts(token, &current);
                let row = format!(
                    "{label} {counts} {} {} {} {}",
                    links(&token.node_pointers()),
                    current.text,
                    pending.text,
                    trace.events.borrow()
                );
                assert_eq!(row, native_line(label));
                assert!(!token.is_empty());
                current
            }
            fn pending_graph(token: &Token<'_>) -> GraphSnapshot {
                unsafe {
                    graph(
                        token.replacement_buckets(),
                        token.replacement_bucket_count(),
                        key,
                    )
                }
            }
            fn failure_case(pattern: &str, hashes: [usize; 3], rejected: usize) {
                let arena = BaseAllocator::accumulating(4096).unwrap();
                let trace = ResizeTrace::new(hashes);
                let map = build(&trace, &arena);
                let old = map.buckets();
                let token = interrupted(map, &trace, rejected);
                assert_eq!(token.buckets(), old);
                assert_eq!(token.allocator(), &arena);
                let label = format!("{}_{}_{}", $family, pattern, rejected);
                let current = verify_snapshot(&label, &token, &trace);
                verify_panic(token.panic_payload(), rejected);
                if current.unique {
                    defined_clear(&label, token, &trace, &arena, current);
                } else {
                    rejected_clear(token, &trace, old);
                }
            }
            fn verify_panic(panic: &(dyn std::any::Any + Send), rejected: usize) {
                assert_eq!(
                    panic.downcast_ref::<CallbackPanic>().unwrap().ordinal,
                    rejected
                );
            }
            fn clear_or_fail(token: Token<'_>) -> (Map<'_>, Box<dyn std::any::Any + Send>) {
                match token.native_clear(false) {
                    Ok(result) => result,
                    Err(_) => panic!("disjoint graph rejected; expected native clear"),
                }
            }
            fn clear_observation(label: &str, arena: &BaseAllocator, trace: &ResizeTrace) {
                let row = format!(
                    "{label}_clear defined {} 0 {}",
                    active(arena),
                    trace.events.borrow()
                );
                assert_eq!(row, native_line(&format!("{label}_clear")));
            }
            fn defined_clear(
                label: &str,
                token: Token<'_>,
                trace: &ResizeTrace,
                arena: &BaseAllocator,
                current: GraphSnapshot,
            ) {
                let nodes = token.node_pointers();
                let replacement = token.replacement_buckets();
                let count = token.replacement_bucket_count();
                trace.events.borrow_mut().clear();
                let (map, panic) = clear_or_fail(token);
                assert!(map.is_empty());
                assert!(panic.is::<CallbackPanic>());
                clear_observation(label, arena, trace);
                trace.active.set(false);
                repair_orphans(arena, nodes, current, replacement, count);
            }
            fn repair_orphans(
                arena: &BaseAllocator,
                nodes: Vec<*mut Node<'_>>,
                current: GraphSnapshot,
                replacement: *const *mut MapLinkNode,
                count: usize,
            ) {
                let orphans = nodes
                    .into_iter()
                    .filter(|node| !current.reached.contains(&node.addr()));
                unsafe {
                    reclaim_nodes(arena, orphans);
                    reclaim_bucket(replacement, count);
                }
            }
            fn reject_clear(token: Token<'_>) -> Token<'_> {
                match token.native_clear(true) {
                    Err(token) => token,
                    Ok(_) => panic!("duplicate graph accepted; expected untouched interruption"),
                }
            }
            fn verify_repaired_clear(map: &mut Map<'_>, trace: &ResizeTrace) {
                trace.events.borrow_mut().clear();
                map.clear(false);
                let label = concat!($family, "_merge_3_clear");
                let row = format!(
                    "{label} repaired {} 0 {}",
                    active(map.allocator()),
                    trace.events.borrow()
                );
                assert_eq!(row, native_line(label));
                trace.active.set(false);
            }
            fn rejected_clear(token: Token<'_>, trace: &ResizeTrace, old: *const *mut MapLinkNode) {
                let before = links(&token.node_pointers());
                let pending = token.replacement_buckets();
                let events = trace.events.borrow().clone();
                let token = reject_clear(reject_clear(token));
                assert_eq!(links(&token.node_pointers()), before);
                assert_eq!(token.replacement_buckets(), pending);
                assert_eq!(*trace.events.borrow(), events);
                let (mut map, panic) = token.recover();
                assert_eq!(map.buckets(), old);
                assert!(panic.is::<CallbackPanic>());
                assert_eq!(*trace.events.borrow(), events);
                verify_repaired_clear(&mut map, trace);
            }
            #[test]
            fn all_native_partial_links_and_defined_clear_prefixes_match() {
                for (pattern, hashes) in PATTERNS {
                    for rejected in 1..=3 {
                        failure_case(pattern, hashes, rejected);
                    }
                }
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
            fn successful_case(pattern: &str, hashes: [usize; 3]) {
                let arena = BaseAllocator::accumulating(4096).unwrap();
                let trace = ResizeTrace::new(hashes);
                let map = build(&trace, &arena);
                let old = map.buckets();
                let nodes = map_nodes(&map);
                trace.prepare(0);
                let mut map = match map.resize_with_progress(1000) {
                    Ok(map) => map,
                    Err(_) => panic!("accepted hash failed; expected success"),
                };
                assert_ne!(map.buckets(), old);
                let label = format!("{}_{}_0", $family, pattern);
                verify_success(&label, &map, &trace, &nodes);
                trace.events.borrow_mut().clear();
                map.clear(false);
                clear_observation(&label, &arena, &trace);
            }
            fn success_counts(map: &Map<'_>) -> String {
                let counts = [
                    0,
                    map.len(),
                    map.bucket_count(),
                    0,
                    1,
                    0,
                    (map.bucket_count() + 1) * size_of::<*mut MapLinkNode>(),
                    active(map.allocator()) as usize,
                    1,
                    3,
                ];
                counts.map(|count| count.to_string()).join(" ")
            }
            fn verify_success(
                label: &str,
                map: &Map<'_>,
                trace: &ResizeTrace,
                nodes: &[*mut Node<'_>],
            ) {
                let snapshot = unsafe { graph(map.buckets(), map.bucket_count(), key) };
                let counts = success_counts(map);
                let row = format!(
                    "{label} {counts} {} {} {} {}",
                    links(nodes),
                    snapshot.text,
                    snapshot.text,
                    trace.events.borrow()
                );
                assert_eq!(row, native_line(label));
            }
            #[test]
            fn successful_resize_matches_native_links_and_drop_order() {
                for (pattern, hashes) in PATTERNS {
                    successful_case(pattern, hashes);
                }
            }
            #[test]
            fn no_op_and_empty_resize_never_invoke_hasher() {
                let arena = BaseAllocator::fresh_heap();
                let trace = ResizeTrace::new([0, 0, 0]);
                let map = build(&trace, &arena);
                let old = map.buckets();
                trace.prepare(1);
                let map = match map.resize_with_progress(0) {
                    Ok(map) => map,
                    Err(_) => panic!("no-op resize failed; expected unchanged map"),
                };
                assert_eq!(map.buckets(), old);
                assert!(trace.events.borrow().is_empty());
                trace.active.set(false);
                drop(map);
            }
            #[test]
            fn empty_resize_allocates_without_invoking_hasher() {
                let trace = ResizeTrace::new([0, 0, 0]);
                let arena = BaseAllocator::fresh_heap();
                let map = Map::with_allocator(ResizePolicy(&trace), 13, arena);
                trace.prepare(1);
                let map = match map.resize_with_progress(0) {
                    Ok(map) => map,
                    Err(_) => panic!("empty resize failed; expected allocated map"),
                };
                assert!(!map.buckets().is_null());
                assert!(trace.events.borrow().is_empty());
                trace.active.set(false);
            }
            fn preflight_case(cycle: bool) {
                let arena = BaseAllocator::fresh_heap();
                let trace = ResizeTrace::new([0, 0, 0]);
                let map = build(&trace, &arena);
                let token = interrupted(map, &trace, 2);
                let buckets = token.buckets().cast_mut();
                let original = unsafe { *buckets.add(1) };
                let mut foreign = MapLinkNode::new(std::ptr::null_mut());
                unsafe {
                    inject_bad_link(buckets, original, &mut foreign, cycle);
                }
                let events = trace.events.borrow().clone();
                let token = reject_clear(reject_clear(token));
                assert_eq!(*trace.events.borrow(), events);
                unsafe {
                    *buckets.add(1) = original;
                }
                recover_preflight(token, &trace, &events);
            }
            fn recover_preflight(token: Token<'_>, trace: &ResizeTrace, events: &str) {
                let (map, panic) = token.recover();
                assert_eq!(*trace.events.borrow(), events);
                verify_panic(&*panic, 2);
                trace.active.set(false);
                verify_lookup(&map, trace, &map_nodes(&map));
            }
            unsafe fn inject_bad_link(
                buckets: *mut *mut MapLinkNode,
                original: *mut MapLinkNode,
                foreign: &mut MapLinkNode,
                cycle: bool,
            ) {
                unsafe {
                    if cycle {
                        *(*original).next_mut() = original;
                    } else {
                        *buckets.add(1) = std::ptr::from_mut(foreign);
                    }
                }
            }
            #[test]
            fn cycles_and_unknown_nodes_reject_without_partial_cleanup() {
                preflight_case(false);
                preflight_case(true);
            }
            fn native_clear_arena_case(arena: BaseAllocator) {
                let trace = ResizeTrace::new([0, 0, 0]);
                let map = build(&trace, &arena);
                let token = interrupted(map, &trace, 2);
                let nodes = token.node_pointers();
                let replacement = token.replacement_buckets();
                let count = token.replacement_bucket_count();
                let current = unsafe { graph(token.buckets(), token.bucket_count(), key) };
                trace.events.borrow_mut().clear();
                let (map, panic) = match token.native_clear(true) {
                    Ok(result) => result,
                    Err(_) => panic!("disjoint graph rejected; expected released empty map"),
                };
                assert!(map.is_empty());
                assert!(map.buckets().is_null());
                verify_panic(&*panic, 2);
                assert_eq!(*trace.events.borrow(), "D3,");
                trace.active.set(false);
                repair_orphans(&arena, nodes, current, replacement, count);
            }
            #[test]
            fn native_clear_abandons_exact_orphans_for_every_allocator() {
                for arena in arenas() {
                    native_clear_arena_case(arena);
                }
            }
            fn recover_case(arena: BaseAllocator, rejected: usize) {
                let trace = ResizeTrace::new([0, 0, 0]);
                let map = build(&trace, &arena);
                let old = map.buckets();
                let token = interrupted(map, &trace, rejected);
                let old_nodes = token.node_pointers();
                let events = trace.events.borrow().clone();
                let (mut map, panic) = token.recover();
                assert_eq!(map.buckets(), old);
                assert_eq!(*trace.events.borrow(), events);
                assert_eq!(map.len(), 3);
                verify_panic(&*panic, rejected);
                trace.active.set(false);
                verify_lookup(&map, &trace, &old_nodes);
                map.clear(true);
            }
            fn verify_lookup(map: &Map<'_>, trace: &ResizeTrace, old_nodes: &[*mut Node<'_>]) {
                for id in [1, 2, 3] {
                    let mut node = std::ptr::null_mut();
                    assert!(map.lookup_node(&ResizeKey { id, trace }, &mut node));
                    assert!(old_nodes.iter().any(|old| old.addr() == node.addr()));
                }
                assert_eq!(
                    unsafe { graph(map.buckets(), map.bucket_count(), key) }
                        .reached
                        .len(),
                    3
                );
            }
            #[test]
            fn recover_preserves_original_panic_and_nodes_for_every_allocator() {
                for rejected in 1..=3 {
                    for arena in arenas() {
                        recover_case(arena, rejected);
                    }
                }
            }
            fn containment_case(arena: BaseAllocator, policy_panics: bool) {
                let trace = ResizeTrace::new([0, 1009, 2018]);
                let map = build(&trace, &arena);
                let token = interrupted(map, &trace, 3);
                let nodes = token.node_pointers();
                let replacement = token.replacement_buckets();
                let count = token.replacement_bucket_count();
                check_contained_drop(token, &trace, policy_panics);
                assert_eq!(arena.ref_count(), 1);
                trace.active.set(false);
                unsafe {
                    reclaim_nodes(&arena, nodes.into_iter());
                    reclaim_bucket(replacement, count);
                }
            }
            fn check_contained_drop(token: Token<'_>, trace: &ResizeTrace, policy_panics: bool) {
                trace.events.borrow_mut().clear();
                trace.policy_panic.set(policy_panics);
                assert_eq!(
                    catch_unwind(AssertUnwindSafe(|| drop(token))).is_err(),
                    policy_panics
                );
                assert!(trace.events.borrow().is_empty());
                assert_eq!(trace.policy_drops.get(), 1);
            }
            #[test]
            fn token_drop_abandons_payloads_before_even_panicking_policy_teardown() {
                for panics in [false, true] {
                    for arena in arenas() {
                        containment_case(arena, panics);
                    }
                }
            }
            #[test]
            fn incremental_reset_reclaims_abandoned_nodes_without_payload_drops() {
                let arena = BaseAllocator::incremental(4096);
                let trace = ResizeTrace::new([0, 0, 0]);
                let map = build(&trace, &arena);
                let token = interrupted(map, &trace, 2);
                let replacement = token.replacement_buckets();
                let count = token.replacement_bucket_count();
                assert!(arena.reset(false).is_err());
                trace.events.borrow_mut().clear();
                drop(token);
                arena.reset(false).unwrap();
                assert!(trace.events.borrow().is_empty());
                unsafe {
                    reclaim_bucket(replacement, count);
                }
            }
        }
    };
}
progress_suite!(
    set,
    "set",
    OrderedMap<ResizeKey<'a>, ResizePolicy<'a>>,
    OrderedMapResizeInterruption<ResizeKey<'a>, ResizePolicy<'a>>,
    OrderedMapNode<ResizeKey<'a>>,
    fill_set
);
progress_suite!(
    data,
    "data",
    OrderedDataMap<ResizeKey<'a>, i32, ResizePolicy<'a>>,
    OrderedDataMapResizeInterruption<ResizeKey<'a>, i32, ResizePolicy<'a>>,
    OrderedDataMapNode<ResizeKey<'a>, i32>,
    fill_data
);
