use super::fixture::*;
use rustycad_tkernel::ncollection::{
    BaseAllocator, OrderedDataMap, OrderedDataMapNode, OrderedMap, OrderedMapNode,
};
use std::panic::{AssertUnwindSafe, catch_unwind};
macro_rules! copy_suite {
    ($module:ident,$prefix:literal,$map:ty,$node:ty,$fill:expr,$payloads:expr) => {
        mod $module {
            use super::*;
            type Map<'a> = $map;
            type Node<'a> = $node;
            fn outcome(source: &Map<'_>) -> String {
                match catch_unwind(AssertUnwindSafe(|| source.clone())) {
                    Ok(map) => format!("ok {} {}", map.len(), map.bucket_count()),
                    Err(_) => "throw".to_owned(),
                }
            }
            fn copy_case(label: &str, mode: CopyFailure) {
                let arena = BaseAllocator::accumulating(4096).unwrap();
                let trace = CopyTrace::new(&arena);
                let mut source = Map::with_allocator(CopyPolicy(&trace), 101, arena.clone());
                if mode != CopyFailure::Empty {
                    ($fill)(&mut source, &trace);
                }
                trace.prepare(mode);
                let result = outcome(&source);
                let label = format!("{}_{label}", $prefix);
                trace.verify(&label, Some(&result));
                trace.prepare(CopyFailure::None);
                drop(source);
                trace.verify(&format!("{label}_source_drop"), None);
            }
            #[test]
            fn retained_node_can_be_reclaimed_after_recorded_native_failure() {
                let arena = BaseAllocator::accumulating(4096).unwrap();
                let trace = CopyTrace::new(&arena);
                let mut source = Map::with_allocator(CopyPolicy(&trace), 101, arena.clone());
                ($fill)(&mut source, &trace);
                trace.capture.set(true);
                trace.prepare(CopyFailure::Hash);
                assert_eq!(outcome(&source), "throw");
                trace.verify(concat!($prefix, "_reclaim_failure"), Some("throw"));
                trace.prepare(CopyFailure::None);
                drop(source);
                reclaim::<Node<'_>>($prefix, &trace);
            }
            #[test]
            fn failed_copy_releases_leases_and_reset_reuses_abandoned_nodes() {
                let arena = BaseAllocator::incremental(4096);
                let first = arena.allocate_raw(0).unwrap();
                let trace = CopyTrace::new(&arena);
                let mut source = Map::with_allocator(CopyPolicy(&trace), 101, arena.clone());
                ($fill)(&mut source, &trace);
                trace.prepare(CopyFailure::Hash);
                assert_eq!(outcome(&source), "throw");
                trace.verify(concat!($prefix, "_reset_failure"), Some("throw"));
                assert!(arena.reset(false).is_err());
                trace.prepare(CopyFailure::None);
                drop(source);
                verify_reset($prefix, &trace, first);
            }
            #[test]
            fn completed_copy_drop_releases_nodes_before_policy_and_base() {
                copy_case("copy_ok", CopyFailure::None);
            }
            #[test]
            fn empty_copy_retains_modulus_without_allocating_nodes() {
                copy_case("copy_empty", CopyFailure::Empty);
            }
            #[test]
            fn policy_copy_failure_releases_only_constructed_base() {
                copy_case("copy_policy", CopyFailure::Policy);
            }
            #[test]
            fn hash_failure_retains_completed_native_nodes() {
                copy_case("copy_hash", CopyFailure::Hash);
            }
            #[test]
            fn equality_failure_retains_completed_native_nodes() {
                copy_case("copy_equal", CopyFailure::Equal);
            }
            #[test]
            fn every_key_value_copy_failure_preserves_native_progress() {
                for rejected in 1..=$payloads {
                    copy_case(
                        &format!("copy_payload{rejected}"),
                        CopyFailure::Payload(rejected),
                    );
                }
            }
        }
    };
}
fn fill_set<'a>(map: &mut OrderedMap<CopyPayload<'a>, CopyPolicy<'a>>, trace: &'a CopyTrace<'a>) {
    map.add(CopyPayload { id: 4, trace });
    map.add(CopyPayload { id: 1, trace });
}
fn fill_map<'a>(
    map: &mut OrderedDataMap<CopyPayload<'a>, CopyPayload<'a>, CopyPolicy<'a>>,
    trace: &'a CopyTrace<'a>,
) {
    map.bind(CopyPayload { id: 4, trace }, CopyPayload { id: 40, trace });
    map.bind(CopyPayload { id: 1, trace }, CopyPayload { id: 10, trace });
}
copy_suite!(
    set,
    "set",
    OrderedMap<CopyPayload<'a>, CopyPolicy<'a>>,
    OrderedMapNode<CopyPayload<'a>>,
    fill_set,
    2
);
copy_suite!(
    data,
    "data",
    OrderedDataMap<CopyPayload<'a>, CopyPayload<'a>, CopyPolicy<'a>>,
    OrderedDataMapNode<CopyPayload<'a>, CopyPayload<'a>>,
    fill_map,
    4
);
