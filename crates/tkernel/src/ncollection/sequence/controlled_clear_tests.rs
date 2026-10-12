//! Actual-node traversal preserves old metadata, saved-next and lost callback ownership.
use super::*;
use crate::standard::{Handle, StandardTransient, catch_native_failure};

fn retire_integer_node(sequence: &mut Sequence<i32>, progress: &mut SequenceClearProgress) -> i32 {
    let value = *sequence.visit_clear_node(progress).unwrap().unwrap();
    let node = sequence.detach_clear_node(progress).unwrap();
    assert_eq!(*node.value(), value);
    drop(node);
    value
}
#[test]
fn ordered_retirement_keeps_native_metadata_and_guards_freed_prefixes() {
    let mut sequence: Sequence<i32> = [1, 2, 3].into_iter().collect();
    let observed = sequence.observer();
    let mut progress = sequence.begin_clear_progress();
    assert_eq!(sequence.first(), Some(&1));
    assert_eq!(retire_integer_node(&mut sequence, &mut progress), 1);
    assert_eq!(observed.snapshot().size, 3);
    assert!(sequence.first().is_none());
    assert!(sequence.iter().next().is_none());
    assert_eq!(sequence.value(3).unwrap(), &3);
    assert_eq!(retire_integer_node(&mut sequence, &mut progress), 2);
    assert_eq!(observed.snapshot().size, 3);
    assert_eq!(retire_integer_node(&mut sequence, &mut progress), 3);
    assert_eq!(observed.snapshot().size, 3);
    sequence.finish_clear_progress(progress).unwrap();
    assert!(sequence.is_empty());
}
#[test]
fn saved_next_visits_first_callback_append_and_loses_last_callback_append() {
    let mut sequence: Sequence<i32> = [1, 2].into_iter().collect();
    let mut progress = sequence.begin_clear_progress();
    assert_eq!(
        *sequence.visit_clear_node(&mut progress).unwrap().unwrap(),
        1
    );
    sequence.append(3);
    drop(sequence.detach_clear_node(&mut progress).unwrap());
    assert_eq!(retire_integer_node(&mut sequence, &mut progress), 2);
    assert_eq!(retire_integer_node(&mut sequence, &mut progress), 3);
    sequence.finish_clear_progress(progress).unwrap();
    assert!(sequence.is_empty());
    verify_lost_last_callback_handle();
}
fn verify_lost_last_callback_handle() {
    let anchor = Handle::new(StandardTransient::default());
    let mut sequence: Sequence<Handle<StandardTransient>> =
        [Handle::default()].into_iter().collect();
    let mut progress = sequence.begin_clear_progress();
    sequence.visit_clear_node(&mut progress).unwrap().unwrap();
    sequence.append(anchor.clone());
    drop(sequence.detach_clear_node(&mut progress).unwrap());
    assert!(sequence.visit_clear_node(&mut progress).unwrap().is_none());
    sequence.finish_clear_progress(progress).unwrap();
    assert!(sequence.is_empty());
    assert_eq!(anchor.ref_count(), 2);
}
#[test]
fn active_clear_rejects_misordered_or_foreign_tokens_without_dereferencing_freed_links() {
    let mut sequence: Sequence<i32> = [1, 2].into_iter().collect();
    let mut progress = sequence.begin_clear_progress();
    assert!(sequence.detach_clear_node(&mut progress).is_err());
    sequence.visit_clear_node(&mut progress).unwrap();
    assert!(sequence.visit_clear_node(&mut progress).is_err());
    let other: Sequence<i32> = [3].into_iter().collect();
    assert!(other.check_clear_progress(&progress).is_err());
    assert!(catch_native_failure(|| Ok(sequence.begin_clear_progress())).is_err());
    drop(sequence.detach_clear_node(&mut progress).unwrap());
    assert!(catch_native_failure(|| sequence.value(1)).is_err());
    sequence.clear(None);
    assert!(sequence.is_empty());
}
#[test]
fn remaining_live_nodes_keep_native_index_origin_and_exchange() {
    let mut sequence: Sequence<i32> = [1, 2, 3, 4].into_iter().collect();
    let mut progress = sequence.begin_clear_progress();
    retire_integer_node(&mut sequence, &mut progress);
    sequence.exchange(3, 4).unwrap();
    assert_eq!(sequence.value(3).unwrap(), &4);
    assert_eq!(sequence.value(4).unwrap(), &3);
    assert_eq!(retire_integer_node(&mut sequence, &mut progress), 2);
    assert_eq!(retire_integer_node(&mut sequence, &mut progress), 4);
    assert_eq!(retire_integer_node(&mut sequence, &mut progress), 3);
    sequence.finish_clear_progress(progress).unwrap();
    assert!(sequence.is_empty());
}
