//! Native policy callbacks distinguish borrowed construction, transfer and assignment.
#[path = "flat_move_contract/fixtures.rs"]
mod fixtures;
use fixtures::{MoveElement, MovePolicy, MoveTrace, move_dictionary, move_policy, move_set};
use rustycad_tkernel::ncollection::{FlatDataMap, FlatMap};

macro_rules! assert_move_state {
    ($map:expr, $expected:expr) => {
        assert_eq!(($map.len(), $map.capacity(), $map.hasher().salt), $expected);
    };
}

macro_rules! borrowed_policy_constructor {
    ($test:ident, $map:ty) => {
        #[test]
        fn $test() {
            let trace: MoveTrace = MoveTrace::default();
            let policy: MovePolicy = move_policy(&trace, 7);
            for (hint, capacity) in [(0, 0), (1, 8), (13, 16), (14, 32), (40, 64)] {
                trace.borrow_mut().clear();
                let map = <$map>::with_hasher_ref(&policy, hint);
                assert_eq!(&*trace.borrow(), "C");
                assert_eq!((map.len(), map.capacity()), (0, capacity));
                assert_eq!((map.hasher().salt, policy.salt), (7, 7));
            }
        }
    };
}
borrowed_policy_constructor!(flat_set_copies_borrowed_policy, FlatMap<MoveElement, MovePolicy>);
borrowed_policy_constructor!(flat_dictionary_copies_borrowed_policy, FlatDataMap<MoveElement, MoveElement, MovePolicy>);

macro_rules! moved_policy_constructor {
    ($test:ident, $map:ty) => {
        #[test]
        fn $test() {
            let trace: MoveTrace = MoveTrace::default();
            for (hint, capacity) in [(0, 0), (1, 8), (13, 16), (14, 32), (40, 64)] {
                let mut policy: MovePolicy = move_policy(&trace, 7);
                trace.borrow_mut().clear();
                let map = <$map>::with_hasher_move_from(&mut policy, hint, MovePolicy::take_policy);
                assert_eq!(&*trace.borrow(), "M");
                assert_eq!((map.len(), map.capacity()), (0, capacity));
                assert_eq!((map.hasher().salt, policy.salt), (7, -1));
            }
        }
    };
}
moved_policy_constructor!(flat_set_moves_injected_policy_without_default, FlatMap<MoveElement, MovePolicy>);
moved_policy_constructor!(flat_dictionary_moves_injected_policy_without_default, FlatDataMap<MoveElement, MoveElement, MovePolicy>);

macro_rules! custom_move_construction {
    ($test:ident, $build:ident, $map:ty, $empty:literal) => {
        #[test]
        fn $test() {
            let trace: MoveTrace = MoveTrace::default();
            let mut source = $build(&trace, 4, 7);
            if $empty {
                source.clear(false);
            }
            let capacity: usize = source.capacity();
            trace.borrow_mut().clear();
            let moved = <$map>::move_from_with(&mut source, MovePolicy::take_policy);
            assert_eq!(&*trace.borrow(), "M");
            assert_move_state!(source, (0, 0, -1));
            assert_move_state!(moved, (usize::from(!$empty), capacity, 7));
        }
    };
}
custom_move_construction!(flat_set_moves_custom_policy, move_set, FlatMap<MoveElement, MovePolicy>, false);
custom_move_construction!(flat_dictionary_moves_custom_policy, move_dictionary, FlatDataMap<MoveElement, MoveElement, MovePolicy>, false);
custom_move_construction!(flat_set_moves_empty_reserved_storage, move_set, FlatMap<MoveElement, MovePolicy>, true);
custom_move_construction!(flat_dictionary_moves_empty_reserved_storage, move_dictionary, FlatDataMap<MoveElement, MoveElement, MovePolicy>, true);

macro_rules! custom_move_assignment {
    ($test:ident, $build:ident, $events:literal, $empty:literal) => {
        #[test]
        fn $test() {
            let trace: MoveTrace = MoveTrace::default();
            let mut source = $build(&trace, 4, 7);
            let mut target = $build(&trace, 9, 2);
            if $empty {
                source.clear(false);
            }
            let identity = std::ptr::from_ref(&target);
            trace.borrow_mut().clear();
            assert!(std::ptr::eq(
                target.move_assign_with(&mut source, MovePolicy::assign_policy),
                identity
            ));
            assert_eq!(&*trace.borrow(), $events);
            assert_move_state!(source, (0, 0, -1));
            assert_move_state!(target, (usize::from(!$empty), 64, 7));
        }
    };
}
custom_move_assignment!(
    flat_set_assigns_policy_after_destroying_keys,
    move_set,
    "kA",
    false
);
custom_move_assignment!(
    flat_dictionary_assigns_policy_after_destroying_pairs,
    move_dictionary,
    "kvA",
    false
);
custom_move_assignment!(
    flat_set_assigns_empty_reserved_storage,
    move_set,
    "kA",
    true
);
custom_move_assignment!(
    flat_dictionary_assigns_empty_reserved_storage,
    move_dictionary,
    "kvA",
    true
);

macro_rules! self_move_assignment {
    ($test:ident, $build:ident) => {
        #[test]
        fn $test() {
            let trace: MoveTrace = MoveTrace::default();
            let mut map = $build(&trace, 4, 7);
            let identity = std::ptr::from_ref(&map);
            trace.borrow_mut().clear();
            assert!(std::ptr::eq(map.move_assign_self(), identity));
            assert!(trace.borrow().is_empty());
            assert_move_state!(map, (1, 64, 7));
        }
    };
}
self_move_assignment!(flat_set_preserves_self_move, move_set);
self_move_assignment!(flat_dictionary_preserves_self_move, move_dictionary);

#[test]
fn flat_set_default_move_returns_destination_and_keeps_key_address() {
    let mut source = FlatMap::<i32>::new(40);
    source.add(4);
    let key_address = std::ptr::from_ref(source.iter().next().unwrap());
    let mut target = FlatMap::<i32>::new(1);
    target.add(9);
    let identity = std::ptr::from_ref(&target);
    assert!(std::ptr::eq(
        target.move_assign_returning(&mut source),
        identity
    ));
    assert_eq!((source.len(), source.capacity()), (0, 0));
    assert_eq!((target.len(), target.capacity()), (1, 64));
    assert!(std::ptr::eq(target.iter().next().unwrap(), key_address));
}

#[test]
fn flat_dictionary_default_move_returns_destination_and_keeps_item_address() {
    let mut source = FlatDataMap::<i32, i32>::new(40);
    source.bind(4, 40);
    let item_address = std::ptr::from_ref(source.iter().next().unwrap());
    let mut target = FlatDataMap::<i32, i32>::new(1);
    target.bind(9, 90);
    let identity = std::ptr::from_ref(&target);
    assert!(std::ptr::eq(
        target.move_assign_returning(&mut source),
        identity
    ));
    assert_eq!((source.len(), source.capacity()), (0, 0));
    assert_eq!((target.len(), target.capacity()), (1, 64));
    assert!(std::ptr::eq(target.iter().next().unwrap(), item_address));
}
