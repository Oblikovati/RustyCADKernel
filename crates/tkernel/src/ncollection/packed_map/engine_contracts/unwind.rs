//! Rust-only open-trait unwind safety; OCCT's integral template arguments cannot panic.
use super::{PackedInteger, PackedMap, PackedWord, pointers};
use std::cell::Cell;
thread_local! { static FAIL_WORD_READ: Cell<bool> = const { Cell::new(false) }; }
#[derive(Clone, Copy, Default)]
struct UnwindWord(u32);
impl PackedWord for UnwindWord {
    fn from_bits(value: u64) -> Self {
        Self(value as u32)
    }
    fn bits(self) -> u64 {
        FAIL_WORD_READ
            .with(|fail| assert!(!fail.get(), "injected word read: expected no active fault"));
        u64::from(self.0)
    }
}
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct UnwindInteger(u32);
impl PackedInteger for UnwindInteger {
    type Word = UnwindWord;
    const IS_64_BIT: bool = false;
    const MIN: Self = Self(0);
    const MAX: Self = Self(u32::MAX);
    fn bits(self) -> u64 {
        u64::from(self.0)
    }
    fn from_bits(value: u64) -> Self {
        Self(value as u32)
    }
}
#[test]
fn engine_hash_panic_preserves_original_node_ownership() {
    let mut map = PackedMap::<UnwindInteger>::default();
    for key in [0, 1, 32, 3232, u32::MAX] {
        map.add(UnwindInteger(key));
    }
    let original = pointers(&map);
    let values: Vec<_> = map.iter().collect();
    FAIL_WORD_READ.with(|fail| fail.set(true));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| map.resize(1000)));
    FAIL_WORD_READ.with(|fail| fail.set(false));
    assert!(result.is_err());
    assert_eq!(pointers(&map), original);
    assert_eq!(map.iter().collect::<Vec<_>>(), values);
    assert_eq!(map.bucket_count(), 101);
}
#[test]
fn engine_drop_never_reads_user_words() {
    let mut map = PackedMap::<UnwindInteger>::default();
    map.add(UnwindInteger(1));
    map.add(UnwindInteger(3232));
    FAIL_WORD_READ.with(|fail| fail.set(true));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(map)));
    FAIL_WORD_READ.with(|fail| fail.set(false));
    assert!(result.is_ok());
}
#[test]
fn engine_all_collision_removal_positions_release_once() {
    for order in [[0, 3232, 6464], [3232, 6464, 0], [6464, 0, 3232]] {
        let mut map = PackedMap::<u32>::default();
        for key in [0, 3232, 6464] {
            map.add(key);
        }
        for key in order {
            assert!(map.remove(key));
            assert!(!map.remove(key));
        }
        assert!(map.is_empty());
        map.clear();
    }
}

#[test]
fn engine_deleted_cursor_node_resets_search_mask() {
    let mut map = PackedMap::<u32>::default();
    map.add(31);
    map.add(32);
    let (mut cursor, first) = map.cursor_first();
    assert_eq!(first, Some(31));
    assert!(map.remove(31));
    assert_eq!(map.cursor_next(&mut cursor), Some(32));
    assert_eq!(map.cursor_next(&mut cursor), None);
}
