//! Proxy-owner comparison policies preserve native Size dispatch without invoking accessors.
#[path = "base_map_indexed_proxy_comparisons/support.rs"]
mod support;
use support::{ComparisonOwner, Cursor, Fixture, trace};

const ATTACHED: [(&str, usize, usize, bool); 5] = [
    ("live_same", 1, 1, true),
    ("live_other", 1, 1, false),
    ("end_live", 3, 1, false),
    ("both_end", 3, 4, false),
    ("same_end", 3, 3, true),
];
const DETACHED: [(&str, Option<usize>, Option<usize>); 4] = [
    ("null_live", None, Some(1)),
    ("live_null", Some(1), None),
    ("both_null", None, None),
    ("null_end", None, Some(3)),
];
fn mutable_first_rows() -> String {
    let fixture = Fixture::new();
    let mut rows = String::new();
    for (name, left, right, same) in ATTACHED {
        rows.push_str(&fixture.pair_row::<true, false>(name, (Some(left), Some(right)), same));
    }
    rows
}
fn readonly_first_rows() -> String {
    let mut fixture = Fixture::new();
    let mut rows = fixture.pair_row::<false, true>("zero_live", (Some(0), Some(0)), false);
    rows.push_str(&fixture.pair_row::<false, true>("readonly_first", (Some(3), Some(1)), false));
    fixture.first.extent = 0;
    fixture.second.extent = 0;
    rows.push_str(&fixture.pair_row::<false, true>("size_changed", (Some(1), Some(1)), false));
    rows
}
fn detached_rows() -> String {
    let fixture = Fixture::new();
    let mut rows = String::new();
    for (name, left, right) in DETACHED {
        rows.push_str(&fixture.pair_row::<true, false>(name, (left, right), false));
    }
    rows
}
#[test]
fn native_proxy_comparisons_match_all_size_dispatch_rows() {
    let rows = mutable_first_rows() + &readonly_first_rows() + &detached_rows();
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkernel_base_map_indexed_proxy_comparisons.expected")
    );
}
#[test]
fn mismatched_left_owner_rejects_before_either_size_callback() {
    let f = Fixture::new();
    let left = Cursor::<true>::new(&f.first, 1);
    let right = Cursor::<false>::new(&f.second, 1);
    let (ls, rs) = (
        ComparisonOwner::size::<true>,
        ComparisonOwner::size::<false>,
    );
    assert!(
        left.is_equal_with(&f.second, &right, &f.second, ls, rs)
            .is_err()
    );
    assert!(
        left.is_not_equal_with(&f.second, &right, &f.second, ls, rs)
            .is_err()
    );
    assert_eq!(trace(&f.calls), "-");
}
#[test]
fn skipped_right_owner_and_callback_follow_native_short_circuit() {
    let f = Fixture::new();
    let left = Cursor::<true>::new(&f.first, 1);
    let right = Cursor::<false>::new(&f.second, 1);
    let ls = ComparisonOwner::size::<true>;
    let forbidden = |_: &ComparisonOwner| panic!("unexpected foreign Size");
    let equal = left.is_equal_with(&f.first, &right, &f.first, ls, forbidden);
    assert!(!equal.unwrap());
    assert_eq!(trace(&f.calls), "A");
}
#[test]
fn exhausted_left_validates_right_owner_before_callback() {
    let f = Fixture::new();
    let left = Cursor::<true>::new(&f.first, 3);
    let right = Cursor::<false>::new(&f.second, 1);
    let ls = ComparisonOwner::size::<true>;
    let forbidden = |_: &ComparisonOwner| panic!("unexpected foreign Size");
    let unequal = left.is_not_equal_with(&f.first, &right, &f.first, ls, forbidden);
    assert!(unequal.is_err());
    assert_eq!(trace(&f.calls), "A");
}
#[test]
fn each_size_callback_is_consumed_at_most_once() {
    let f = Fixture::new();
    let left = Cursor::<true>::new(&f.first, 3);
    let right = Cursor::<false>::new(&f.second, 3);
    let (left_capture, right_capture) = (String::from("left"), String::from("right"));
    let ls = |_: &ComparisonOwner| {
        drop(left_capture);
        2
    };
    let rs = |_: &ComparisonOwner| {
        drop(right_capture);
        2
    };
    assert!(
        left.is_equal_with(&f.first, &right, &f.second, ls, rs)
            .unwrap()
    );
}

#[test]
fn owned_getters_exist_but_are_independent_of_size_comparison() {
    let f = Fixture::new();
    let cursor = Cursor::<false>::new(&f.first, 1);
    let value = cursor
        .value_with(&f.first, |m, _| Ok(m.owned_value()))
        .unwrap();
    assert_eq!(value.0, 42);
    assert_eq!(cursor.key_with(&f.first, |m, _| Ok(m.key())).unwrap(), 7);
    assert_eq!(trace(&f.calls), "vk");
    let same = cursor.is_equal_with(
        &f.first,
        &cursor,
        &f.first,
        |m| m.size::<false>(),
        |m| m.size::<false>(),
    );
    assert!(same.unwrap());
    assert_eq!(trace(&f.calls), "a");
}
