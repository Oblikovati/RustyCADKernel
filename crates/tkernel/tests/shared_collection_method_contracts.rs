//! Shared collection copy/move paths preserve storage and transient counters.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{
    Array1, BaseAllocator, HArray1, HArray2, HSequence, Sequence, Shared,
};
use rustycad_tkernel::standard::Transient;

#[test]
fn harray1_copy_and_assignment_keep_bounds_and_independent_reference_counts() {
    let source = HArray1::with_value(-2, 0, 7_i32).unwrap();
    source.transient_base().increment_ref_counter();
    let copy = source.clone();
    assert_eq!(copy.transient_base().ref_count(), 0);
    assert_eq!((copy.array1().lower(), copy.array1().upper()), (-2, 0));
    assert_ne!(copy.array1().data_ptr(), source.array1().data_ptr());
    let mut target = HArray1::with_value(8, 10, 1).unwrap();
    target.transient_base().increment_ref_counter();
    let pointer = target.array1().data_ptr();
    target.clone_from(&source);
    assert_eq!(target.array1().data_ptr(), pointer);
    assert_eq!(target.array1().as_slice(), [7, 7, 7]);
    assert_eq!(target.transient_base().ref_count(), 1);
    assert_eq!(source.transient_base().ref_count(), 1);
    target.transient_base().decrement_ref_counter();
    source.transient_base().decrement_ref_counter();
}

#[test]
fn harray1_move_preserves_buffer_and_resets_source_array() {
    let mut source = HArray1::with_value(-2, 0, 7_i32).unwrap();
    let pointer = source.array1().data_ptr();
    let mut moved = HArray1::move_from(&mut source);
    assert_eq!(moved.array1().data_ptr(), pointer);
    assert_eq!((source.array1().lower(), source.array1().upper()), (1, 0));
    let mut target = HArray1::default();
    let destination = &raw const target;
    assert_eq!(target.move_assign(&mut moved) as *const _, destination);
    assert_eq!(target.array1().data_ptr(), pointer);
    assert!(moved.array1().is_empty());
}

#[test]
fn harray2_copy_assignment_and_move_preserve_rectangle_metadata() {
    let source = HArray2::with_value(-1, 0, 4, 5, 7_i32).unwrap();
    let copy = source.clone();
    assert_eq!(
        (copy.array2().lower_row(), copy.array2().lower_col()),
        (-1, 4)
    );
    assert_ne!(copy.array2().data_ptr(), source.array2().data_ptr());
    let mut target = HArray2::with_value(1, 1, 1, 4, 0).unwrap();
    target.transient_base().increment_ref_counter();
    target.clone_from(&source);
    assert_eq!(target.transient_base().ref_count(), 1);
    assert_eq!(
        (target.array2().row_length(), target.array2().col_length()),
        (2, 2)
    );
    let pointer = target.array2().data_ptr();
    let mut moved = HArray2::move_from(&mut target);
    assert_eq!(moved.array2().data_ptr(), pointer);
    assert!(target.array2().is_empty());
    let destination = &raw const target;
    assert_eq!(target.move_assign(&mut moved) as *const _, destination);
    assert_eq!(target.array2().data_ptr(), pointer);
    target.transient_base().decrement_ref_counter();
}

#[test]
fn harray1_mutable_external_buffer_keeps_native_writable_alias() {
    let raw = Box::into_raw(vec![10_i32, 20, 30].into_boxed_slice());
    // The wrapper's static loan ends before the original allocation is reclaimed below.
    let array = HArray1::from_buffer_mut(unsafe { &mut *raw }, -2, 0, true).unwrap();
    assert_eq!(array.array1().data_ptr(), raw.cast::<i32>());
    assert!(!array.array1().is_deletable());
    array.change_array1().set_value(-1, 99).unwrap();
    assert!(!array.array1().is_deletable());
    let copy = array.clone();
    assert!(copy.array1().is_deletable());
    assert_ne!(copy.array1().data_ptr(), raw.cast::<i32>());
    drop(array);
    let buffer = unsafe { Box::from_raw(raw) };
    assert_eq!(&*buffer, [10, 99, 30]);
}

#[test]
fn harray1_mutable_buffer_checks_length_and_ignored_buffer_flag() {
    let short = Box::into_raw(vec![1_i32].into_boxed_slice());
    let failed = HArray1::from_buffer_mut(unsafe { &mut *short }, 1, 2, true);
    assert!(failed.is_err());
    unsafe {
        drop(Box::from_raw(short));
    }
    let ignored = Box::into_raw(vec![8_i32].into_boxed_slice());
    let fresh = HArray1::from_buffer_mut(unsafe { &mut *ignored }, 1, 3, false).unwrap();
    assert_eq!(fresh.array1().as_slice(), [0, 0, 0]);
    assert!(fresh.array1().is_deletable());
    unsafe {
        drop(Box::from_raw(ignored));
    }
}

#[test]
fn hsequence_copy_and_assignment_preserve_destination_allocator() {
    let source_arena = BaseAllocator::incremental(4096);
    let target_arena = BaseAllocator::incremental(4096);
    let mut sequence = Sequence::with_allocator(source_arena.clone());
    sequence.append(7);
    let source = HSequence::from_owned(sequence);
    source.transient_base().increment_ref_counter();
    let copied = source.clone();
    assert_eq!(copied.transient_base().ref_count(), 0);
    assert_eq!(copied.sequence(|s| s.allocator().clone()), source_arena);
    let mut target = HSequence::from_owned(Sequence::with_allocator(target_arena.clone()));
    target.transient_base().increment_ref_counter();
    target.clone_from(&source);
    assert_eq!(target.sequence(|s| s.allocator().clone()), target_arena);
    assert_eq!(target.sequence(|s| s.first().copied()), Some(7));
    assert_eq!(target.transient_base().ref_count(), 1);
    target.transient_base().decrement_ref_counter();
    source.transient_base().decrement_ref_counter();
}

#[test]
fn hsequence_move_paths_keep_node_addresses() {
    let mut source = HSequence::<i32>::default();
    source.append_ref(&7);
    let pointer = source.sequence(|s| s.first().unwrap() as *const i32);
    let mut moved = HSequence::move_from(&mut source);
    assert!(source.sequence(Sequence::is_empty));
    assert_eq!(
        moved.sequence(|s| s.first().unwrap() as *const i32),
        pointer
    );
    let destination = &raw const source;
    assert_eq!(source.move_assign(&mut moved) as *const _, destination);
    assert!(moved.sequence(Sequence::is_empty));
    assert_eq!(
        source.sequence(|s| s.first().unwrap() as *const i32),
        pointer
    );
}

struct Probe {
    arena: BaseAllocator,
    fail: bool,
}
impl Clone for Probe {
    fn clone(&self) -> Self {
        assert!(
            self.arena.reset(false).is_err(),
            "expected allocated sequence node before item copy"
        );
        assert!(!self.fail, "injected sequence item copy failure");
        Self {
            arena: self.arena.clone(),
            fail: self.fail,
        }
    }
}

#[test]
fn hsequence_borrowed_append_allocates_before_copy_and_releases_failed_node() {
    let arena = BaseAllocator::incremental(4096);
    let sequence = HSequence::from_owned(Sequence::with_allocator(arena.clone()));
    let bad = Probe {
        arena: arena.clone(),
        fail: true,
    };
    let failed =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| sequence.append_ref(&bad)));
    assert!(failed.is_err());
    assert!(sequence.sequence(Sequence::is_empty));
    assert!(arena.reset(false).is_ok());
    sequence.append_ref(&Probe {
        arena: arena.clone(),
        fail: false,
    });
    assert_eq!(sequence.sequence(Sequence::len), 1);
}

#[test]
fn shared_constructor_forwards_mutable_arguments_and_clone_keeps_counter_separate() {
    let mut prefix = String::from("a");
    let suffix = "b";
    let source = Shared::from_constructor(|| {
        prefix.push_str(suffix);
        vec![prefix.clone()]
    });
    assert_eq!(prefix, "ab");
    source.transient_base().increment_ref_counter();
    let copied = source.clone();
    assert_eq!(copied.transient_base().ref_count(), 0);
    assert_eq!(copied.as_slice(), ["ab"]);
    let mut target = Shared::new(vec![String::from("old")]);
    target.transient_base().increment_ref_counter();
    target.clone_from(&source);
    assert_eq!(target.as_slice(), ["ab"]);
    assert_eq!(target.transient_base().ref_count(), 1);
    target.transient_base().decrement_ref_counter();
    source.transient_base().decrement_ref_counter();
}

#[test]
fn harray1_copy_from_borrowed_base_preserves_bounds_without_aliasing() {
    let mut buffer = [10, 20];
    let mut base = Array1::from_slice(&mut buffer);
    base.update_lower_bound(-3);
    let copied = HArray1::from_array(&base);
    assert_eq!((copied.array1().lower(), copied.array1().upper()), (-3, -2));
    base.set_value(-3, 99).unwrap();
    assert_eq!(copied.array1().as_slice(), [10, 20]);
}
