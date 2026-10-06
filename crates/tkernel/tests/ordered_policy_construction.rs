//! Exact native policy callbacks and graph transfer, including throwing construction.
#[path = "ordered_policy_construction/fixture.rs"]
mod fixture;
use fixture::*;
use rustycad_tkernel::ncollection::{
    BaseAllocator, BasicMapIteratorSource, OrderedDataMap, OrderedMap,
};
use std::panic::{AssertUnwindSafe, catch_unwind};
macro_rules! policy_suite {
    ($module:ident,$prefix:literal,$map:ty,$move_map:ty,$fill:expr,$move_fill:expr) => {
        mod $module {
            use super::*;
            type Map = $map;
            #[test]
            fn borrowed_constructors_match_native_validation_clone_and_throw_order() {
                for (suffix, count, signed, reject) in [
                    ("borrow_size", 13, false, false),
                    ("borrow_signed", 0, true, false),
                    ("borrow_negative", -7, true, false),
                    ("borrow_throw", 13, false, true),
                    ("borrow_throw_signed", 13, true, true),
                    ("borrow_negative_throw", -7, true, true),
                ] {
                    let policy = Policy::new(7, reject);
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        if signed {
                            Map::with_borrowed_hasher_int(&policy, count, BaseAllocator::common())
                        } else {
                            Ok(Map::with_borrowed_hasher(
                                &policy,
                                count as usize,
                                BaseAllocator::common(),
                            ))
                        }
                    }));
                    let outcome = constructor_outcome(result);
                    let label = format!("{}_{}", $prefix, suffix);
                    native_row(
                        &label,
                        format!("{label} {outcome} {} {}", policy.state, policy.trace.text()),
                    );
                }
            }
            #[test]
            fn moved_constructors_match_native_validation_and_moved_from_throw_state() {
                for (suffix, count, signed, reject) in [
                    ("move_size", 13, false, false),
                    ("move_signed", 0, true, false),
                    ("move_negative", -7, true, false),
                    ("move_throw", 13, false, true),
                    ("move_throw_signed", 13, true, true),
                    ("move_negative_throw", -7, true, true),
                ] {
                    let mut policy = Policy::new(7, reject);
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        if signed {
                            Map::with_moved_hasher_int(
                                &mut policy,
                                count,
                                BaseAllocator::common(),
                                Policy::take,
                            )
                        } else {
                            Ok(Map::with_moved_hasher(
                                &mut policy,
                                count as usize,
                                BaseAllocator::common(),
                                Policy::take,
                            ))
                        }
                    }));
                    let outcome = constructor_outcome(result);
                    let label = format!("{}_{}", $prefix, suffix);
                    native_row(
                        &label,
                        format!("{label} {outcome} {} {}", policy.state, policy.trace.text()),
                    );
                }
            }
            fn constructor_outcome(
                result: std::thread::Result<Result<Map, rustycad_tkernel::standard::Failure>>,
            ) -> String {
                match result {
                    Ok(Ok(map)) => format!("ok {} {}", map.hasher().state, map.bucket_count()),
                    Ok(Err(error)) => {
                        assert!(error.to_string().contains("-7"));
                        "range".to_owned()
                    }
                    Err(_) => "throw".to_owned(),
                }
            }
            fn transfer(populated: bool, reject: bool) {
                let mut source = Map::with_hasher(Policy::new(7, false), 101);
                if populated {
                    ($fill)(&mut source);
                }
                let (first, buckets) = (source.basic_first_node(), source.buckets());
                source.hasher().trace.reject.set(reject);
                let result = catch_unwind(AssertUnwindSafe(|| {
                    Map::move_from_with(&mut source, Policy::take)
                }));
                let outcome = match result {
                    Ok(target) => format!(
                        "ok {} {} {} {} {}",
                        target.hasher().state,
                        target.len(),
                        u8::from(target.basic_first_node() == first),
                        u8::from(target.buckets() == buckets),
                        u8::from(target.allocator() == source.allocator())
                    ),
                    Err(_) => format!(
                        "throw {} {}",
                        u8::from(source.basic_first_node() == first),
                        u8::from(source.buckets() == buckets)
                    ),
                };
                let label = format!(
                    "{}_transfer_{}_{}",
                    $prefix,
                    u8::from(populated),
                    u8::from(reject)
                );
                native_row(
                    &label,
                    format!(
                        "{label} {outcome} {} {} {} {} {}",
                        source.hasher().state,
                        source.len(),
                        source.bucket_count(),
                        u8::from(source.buckets().is_null()),
                        source.hasher().trace.text()
                    ),
                );
            }
            #[test]
            fn empty_graph_move_preserves_bucket_selection() {
                transfer(false, false);
            }
            #[test]
            fn full_graph_move_preserves_node_bucket_and_allocator_identity() {
                transfer(true, false);
            }
            #[test]
            fn throwing_move_keeps_source_graph_and_mutated_policy() {
                transfer(true, true);
            }
            #[test]
            fn move_only_policy_requires_neither_clone_nor_default() {
                type MoveMap = $move_map;
                let mut policy = MoveOnly(Policy::new(19, false));
                let mut source = MoveMap::with_moved_hasher(
                    &mut policy,
                    101,
                    BaseAllocator::common(),
                    MoveOnly::take,
                );
                ($move_fill)(&mut source);
                let first = source.basic_first_node();
                source.hasher().0.trace.events.borrow_mut().clear();
                let target = MoveMap::move_from_with(&mut source, MoveOnly::take);
                let label = concat!($prefix, "_move_only");
                native_row(
                    label,
                    format!(
                        "{label} {} {} {} {} {} {}",
                        target.hasher().0.state,
                        source.hasher().0.state,
                        target.len(),
                        source.len(),
                        u8::from(target.basic_first_node() == first),
                        target.hasher().0.trace.text()
                    ),
                );
            }
            #[test]
            fn emptied_donor_reallocates_nodes_with_its_moved_from_policy() {
                let arena = BaseAllocator::incremental(1024);
                let mut source = Map::with_allocator(Policy::new(7, false), 101, arena.clone());
                ($fill)(&mut source);
                let target = Map::move_from_with(&mut source, Policy::take);
                ($fill)(&mut source);
                assert_eq!(
                    (source.len(), target.len(), source.hasher().state),
                    (2, 2, -99)
                );
                assert_ne!(source.basic_first_node(), target.basic_first_node());
                assert!(arena.reset(false).is_err());
                drop(target);
                drop(source);
                assert!(arena.reset(false).is_ok());
            }
        }
    };
}
policy_suite!(set,"set",OrderedMap<i32,Policy>,OrderedMap<i32,MoveOnly>,
    |map:&mut OrderedMap<i32,Policy>| { map.add(4); map.add(1); },
    |map:&mut OrderedMap<i32,MoveOnly>| { map.add(4); map.add(1); });
policy_suite!(data,"data",OrderedDataMap<i32,i32,Policy>,OrderedDataMap<i32,i32,MoveOnly>,
    |map:&mut OrderedDataMap<i32,i32,Policy>| { map.bind(4,40); map.bind(1,10); },
    |map:&mut OrderedDataMap<i32,i32,MoveOnly>| { map.bind(4,40); map.bind(1,10); });

#[path = "ordered_policy_construction/payloads.rs"]
mod payloads;
