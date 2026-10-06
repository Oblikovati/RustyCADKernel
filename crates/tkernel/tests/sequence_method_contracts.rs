//! Native NCollection_Sequence method contracts beyond the original GTest cases.
use rustycad_tkernel::ncollection::{BaseAllocator, Sequence, SequenceCursor};
use rustycad_tkernel::standard::Failure;
use std::cell::Cell;

fn sequence(values: &[i32]) -> Sequence<i32> {
    values.iter().copied().collect()
}

fn rejects_negative<T>(result: Result<T, Failure>) {
    let failure = result.err().expect("negative index must be rejected");
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert!(failure.message().contains("-1"));
    assert!(failure.message().contains("nonnegative"));
}

#[test]
fn signed_access_and_edit_overloads_reject_before_modification() {
    let mut values = sequence(&[10, 20, 30]);
    rejects_negative(values.value_signed(-1));
    rejects_negative(values.change_value_signed(-1));
    rejects_negative(values.set_value_signed(-1, 99));
    rejects_negative(values.exchange_signed(-1, 1));
    rejects_negative(values.exchange_signed(1, -1));
    rejects_negative(values.remove_signed(-1));
    rejects_negative(values.remove_range_signed(-1, 2));
    rejects_negative(values.remove_range_signed(1, -1));
    rejects_negative(values.insert_before_signed(-1, 99));
    rejects_negative(values.insert_after_signed(-1, 99));
    assert_eq!(values.iter().copied().collect::<Vec<_>>(), [10, 20, 30]);
}

#[test]
fn signed_overloads_preserve_positive_index_semantics() {
    let mut values = sequence(&[10, 20, 30]);
    assert_eq!(values.value_signed(1).unwrap(), &10);
    *values.change_value_signed(1).unwrap() = 11;
    values.set_value_signed(2, 22).unwrap();
    values.exchange_signed(1, 2).unwrap();
    values.insert_before_signed(1, 1).unwrap();
    values.insert_after_signed(0, 0).unwrap();
    values.remove_signed(1).unwrap();
    values.remove_range_signed(2, 3).unwrap();
    assert_eq!(values.iter().copied().collect::<Vec<_>>(), [1, 30]);
    assert!(values.value_signed(0).is_err());
    assert!(values.change_value_signed(3).is_err());
}

#[test]
fn signed_emplacement_checks_indices_before_constructing() {
    let constructed = Cell::new(0);
    let mut values = sequence(&[10]);
    let construct = || {
        constructed.set(constructed.get() + 1);
        20
    };
    rejects_negative(values.emplace_before_signed(-1, construct));
    rejects_negative(values.emplace_after_signed(-1, construct));
    assert!(values.emplace_before_signed(0, construct).is_err());
    assert!(values.emplace_after_signed(2, construct).is_err());
    assert_eq!(constructed.get(), 0);
    assert_eq!(*values.emplace_after_signed(0, construct).unwrap(), 20);
    assert_eq!(*values.emplace_before_signed(3, construct).unwrap(), 20);
    assert_eq!(constructed.get(), 2);
    assert_eq!(values.iter().copied().collect::<Vec<_>>(), [20, 10, 20]);
}

#[test]
fn signed_split_preserves_negative_destination_but_clears_zero_destination() {
    let first = BaseAllocator::incremental(128);
    let second = BaseAllocator::incremental(128);
    let mut values = Sequence::with_allocator(first.clone());
    values.append(10);
    let mut tail = Sequence::with_allocator(second.clone());
    tail.append(99);
    rejects_negative(values.split_signed(-1, &mut tail));
    assert_eq!(tail.first(), Some(&99));
    assert_eq!(tail.allocator(), &second);
    assert!(values.split_signed(0, &mut tail).is_err());
    assert!(tail.is_empty());
    assert_eq!(tail.allocator(), &first);
    values.split_signed(1, &mut tail).unwrap();
    assert!(values.is_empty());
    assert_eq!(tail.first(), Some(&10));
}

#[test]
fn signed_sequence_splice_checks_negatives_even_for_empty_source() {
    let mut values = sequence(&[10]);
    let mut empty = Sequence::default();
    rejects_negative(values.insert_sequence_after_signed(-1, &mut empty));
    rejects_negative(values.insert_sequence_before_signed(-1, &mut empty));
    assert!(values.insert_sequence_before_signed(0, &mut empty).is_err());
    values
        .insert_sequence_after_signed(i32::MAX, &mut empty)
        .unwrap();
    values
        .insert_sequence_before_signed(i32::MAX, &mut empty)
        .unwrap();
    let mut head = sequence(&[1, 2]);
    values.insert_sequence_before_signed(1, &mut head).unwrap();
    let mut tail = sequence(&[20, 30]);
    values.insert_sequence_after_signed(3, &mut tail).unwrap();
    assert!(head.is_empty() && tail.is_empty());
    assert_eq!(
        values.iter().copied().collect::<Vec<_>>(),
        [1, 2, 10, 20, 30]
    );
}

#[test]
fn assignment_returns_destination_and_preserves_allocator_policy() {
    let source_allocator = BaseAllocator::incremental(128);
    let target_allocator = BaseAllocator::incremental(128);
    let mut source = Sequence::with_allocator(source_allocator.clone());
    source.append(10);
    let mut target = Sequence::with_allocator(target_allocator.clone());
    let target_address = std::ptr::addr_of!(target);
    assert_eq!(target.assign(&source) as *const _, target_address);
    assert_eq!(target.allocator(), &target_allocator);
    let source_item = source.first().unwrap() as *const i32;
    assert_ne!(target.first().unwrap() as *const i32, source_item);
    assert_eq!(target.move_assign(&mut source) as *const _, target_address);
    assert_eq!(target.allocator(), &source_allocator);
    assert_eq!(source.allocator(), &source_allocator);
    assert!(source.is_empty());
    assert_eq!(target.first().unwrap() as *const i32, source_item);
}

#[test]
fn explicit_self_operations_preserve_native_early_returns() {
    let mut values = sequence(&[10, 20]);
    let cursor = values.cursor();
    let owner = std::ptr::addr_of!(values);
    assert_eq!(values.assign_self() as *const _, owner);
    values.append_self();
    values.prepend_self();
    values.insert_self_after(usize::MAX);
    values.insert_self_before(usize::MAX).unwrap();
    assert!(values.insert_self_before(0).is_err());
    assert!(cursor.is_equal(&values.cursor()));
    assert_eq!(values.iter().copied().collect::<Vec<_>>(), [10, 20]);
    let error = values.split_self(1).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert!(values.is_empty());
}

#[test]
fn cursor_returning_steps_and_copy_preserve_previous_link() {
    let values = sequence(&[10, 20, 30]);
    let mut cursor = values.cursor();
    let first = cursor.post_advance(&values);
    assert_eq!(first.value(&values), Some(&10));
    assert_eq!(cursor.value(&values), Some(&20));
    let owner = std::ptr::addr_of!(cursor);
    assert_eq!(cursor.advance(&values) as *const _, owner);
    assert_eq!(cursor.post_retreat(&values).value(&values), Some(&30));
    assert_eq!(cursor.value(&values), Some(&20));
    assert_eq!(cursor.retreat(&values) as *const _, owner);
    assert_eq!(cursor.value(&values), Some(&10));
    cursor.initialize(&values, false);
    let mut copy = SequenceCursor::default();
    cursor.copy_to(&mut copy);
    assert!(copy.is_equal(&cursor));
    assert_eq!(copy.retreat(&values).value(&values), Some(&30));
}

#[test]
fn cursor_assignment_and_end_operations_preserve_native_identity() {
    let values = sequence(&[10, 20]);
    let other = sequence(&[99]);
    let mut cursor = SequenceCursor::new(&values, false);
    let mut target = SequenceCursor::default();
    let owner = std::ptr::addr_of!(target);
    assert_eq!(target.assign(&cursor) as *const _, owner);
    assert_eq!(target.assign_self() as *const _, owner);
    assert_eq!(target.retreat(&values).value(&values), Some(&20));
    cursor.advance(&values);
    assert!(cursor.is_equal(&SequenceCursor::new(&other, false)));
    cursor.retreat(&values);
    assert_eq!(cursor.value(&values), Some(&20));
    cursor.initialize(&values, true);
    cursor.retreat(&values).retreat(&values);
    assert!(!cursor.more());
}

#[test]
fn shared_iterator_clone_and_cross_sequence_end_identity_match_native() {
    struct NonClone(i32);
    let mut first = Sequence::default();
    first.append(NonClone(10));
    let mut second = Sequence::default();
    second.append(NonClone(20));
    let mut position = first.iter();
    let copy = position.clone();
    assert!(position == copy);
    assert_eq!(position.next().unwrap().0, 10);
    assert!(position != copy);
    let mut other = second.iter();
    other.next();
    assert!(position == other);
    assert!(Sequence::<NonClone>::default().iter() == position);
}
