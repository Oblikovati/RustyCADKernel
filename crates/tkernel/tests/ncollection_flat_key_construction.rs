//! Variadic set constructors retain OCCT capacity and final-slot contracts.
#[path = "flat_key_construction/fixtures.rs"]
mod fixtures;
use fixtures::{ConstructionPolicy, ConstructionTrace, constructed_key, failed_key_construction};
use rustycad_tkernel::ncollection::FlatMap;

macro_rules! construction_result {
    ($result:expr, boolean) => {
        i32::from($result)
    };
    ($result:expr, reference) => {
        $result.serial
    };
}
macro_rules! construction_hit_and_miss {
    ($test:ident, $method:ident, $kind:ident, $hit:literal, $stored:literal) => {
        #[test]
        fn $test() {
            let trace: ConstructionTrace = ConstructionTrace::default();
            let mut map = FlatMap::with_hasher(
                ConstructionPolicy {
                    trace: trace.clone(),
                },
                0,
            );
            let miss = map.$method(|| constructed_key(4, 1, &trace));
            assert_eq!(construction_result!(miss, $kind), 1);
            assert_eq!(&*trace.borrow(), "CH");
            trace.borrow_mut().clear();
            let hit = map.$method(|| constructed_key(4, 2, &trace));
            assert_eq!(construction_result!(hit, $kind), $hit);
            assert_eq!(&*trace.borrow(), "CHE");
            assert_eq!(
                (map.len(), map.capacity(), map.iter().next().unwrap().serial),
                (1, 8, $stored)
            );
        }
    };
}
construction_hit_and_miss!(
    flat_set_emplace_constructs_before_hash_on_hits_and_misses,
    emplace_with,
    boolean,
    0,
    2
);
construction_hit_and_miss!(
    flat_set_emplaced_constructs_before_hash_on_hits_and_misses,
    emplaced_with,
    reference,
    2,
    2
);
construction_hit_and_miss!(
    flat_set_try_emplace_constructs_before_hash_but_preserves_hits,
    try_emplace_with,
    boolean,
    0,
    1
);
construction_hit_and_miss!(
    flat_set_try_emplaced_constructs_before_hash_but_preserves_hits,
    try_emplaced_with,
    reference,
    1,
    1
);

macro_rules! failing_constructor_keeps_prepared_capacity {
    ($test:ident, $method:ident) => {
        #[test]
        fn $test() {
            let trace: ConstructionTrace = ConstructionTrace::default();
            let mut map = FlatMap::with_hasher(
                ConstructionPolicy {
                    trace: trace.clone(),
                },
                0,
            );
            let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                map.$method(|| failed_key_construction(&trace));
            }));
            assert!(failure.is_err());
            assert_eq!(&*trace.borrow(), "C");
            assert_eq!((map.len(), map.capacity()), (0, 8));
        }
    };
}
failing_constructor_keeps_prepared_capacity!(
    flat_set_emplace_failure_keeps_prepared_capacity,
    emplace_with
);
failing_constructor_keeps_prepared_capacity!(
    flat_set_emplaced_failure_keeps_prepared_capacity,
    emplaced_with
);
failing_constructor_keeps_prepared_capacity!(
    flat_set_try_emplace_failure_keeps_prepared_capacity,
    try_emplace_with
);
failing_constructor_keeps_prepared_capacity!(
    flat_set_try_emplaced_failure_keeps_prepared_capacity,
    try_emplaced_with
);

macro_rules! constructed_displacement_return {
    ($test:ident, $method:ident) => {
        #[test]
        fn $test() {
            let mut map = FlatMap::<i32>::default();
            map.add(0);
            map.add(1);
            assert_eq!(*map.$method(|| 8), 1);
            assert!(map.contains(&8));
            assert_eq!(map.len(), 3);
        }
    };
}
constructed_displacement_return!(
    flat_set_constructed_emplaced_returns_displaced_key,
    emplaced_with
);
constructed_displacement_return!(
    flat_set_constructed_try_emplaced_returns_displaced_key,
    try_emplaced_with
);
