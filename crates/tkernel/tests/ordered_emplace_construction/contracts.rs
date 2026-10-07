use super::{fixture::*, scenarios::*};
use rustycad_tkernel::ncollection::{BaseAllocator, OrderedMap};
fn fill_integer(map: &mut OrderedMap<i32, EmplacePolicy<'_>>, count: usize) {
    if count == 3 {
        for key in [4, 1, 7] {
            map.add(key);
        }
        return;
    }
    for key in 0..count {
        map.add(key as i32);
    }
}
fn integer_case(operation: Operation, test: EmplaceCase) {
    let arena = BaseAllocator::accumulating(8192).unwrap();
    let trace = EmplaceTrace::new(&arena);
    let mut map = OrderedMap::with_allocator(EmplacePolicy(&trace), 0, arena.clone());
    fill_integer(&mut map, test.count);
    // The first entry is compared by key: slot growth may move it (plan D1 class B5).
    let first = map.first_node().map(|node| node.key().display());
    trace.configure(&test);
    let result = invoke(&mut map, operation, || trace.construct(test.key));
    snapshot(
        &format!("{}_{}", operation.name(), test.label),
        &result,
        &map,
        &trace,
        map.first_node().map(|node| node.key().display()) == first,
    );
}
fn tracked_case(operation: Operation, test: EmplaceCase) {
    let arena = BaseAllocator::accumulating(8192).unwrap();
    let trace = EmplaceTrace::new(&arena);
    let mut map = OrderedMap::with_allocator(EmplacePolicy(&trace), 0, arena.clone());
    map.add(make_key(&trace, 7, 10));
    // The first entry is compared by key: slot growth may move it (plan D1 class B5).
    let first = map.first_node().map(|node| node.key().display());
    trace.configure(&test);
    let result = invoke(&mut map, operation, || make_key(&trace, test.key, 20));
    snapshot(
        &format!("{}_tracked_{}", operation.name(), test.label),
        &result,
        &map,
        &trace,
        map.first_node().map(|node| node.key().display()) == first,
    );
}
macro_rules! emplace_suite {
    ($module:ident,$operation:expr) => {
        mod $module {
            use super::*;
            #[test]
            fn deferred_conversion_preserves_resize_before_constructor_and_lookup() {
                for test in cases() {
                    integer_case($operation, test);
                }
            }
            #[test]
            fn retained_temporary_drops_before_return_or_propagated_lookup_failure() {
                for index in [5, 6, 7] {
                    tracked_case($operation, cases()[index]);
                }
                if $operation.is_try() {
                    tracked_case($operation, cases()[4]);
                }
            }
        }
    };
}
emplace_suite!(emplace, Operation::Emplace);
emplace_suite!(emplaced, Operation::Emplaced);
emplace_suite!(try_emplace, Operation::TryEmplace);
emplace_suite!(try_emplaced, Operation::TryEmplaced);

fn make_key<'a>(trace: &'a EmplaceTrace<'a>, id: i32, tag: i32) -> EmplaceKey<'a> {
    EmplaceKey {
        id: trace.construct(id),
        tag,
        trace,
    }
}
