//! API-completion behaviour of NCollection_List, NCollection_Sequence and NCollection_HSequence:
//! the native exception conditions, cursor positioning and handle sharing.
use rustycad_tkernel::ncollection::{
    BaseAllocator, HSequence, List, ListCursor, Sequence, SequenceCursor,
};
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
fn items(list: &List<i32>) -> Vec<i32> {
    list.iter().copied().collect()
}
fn values(sequence: &Sequence<i32>) -> Vec<i32> {
    sequence.iter().copied().collect()
}
#[test]
fn list_end_positions_raise_no_such_object() {
    let mut list: List<i32> = List::default();
    assert_eq!(
        list.remove_first().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    let mut end = ListCursor::default();
    assert!(list.remove(&mut end).is_err());
    assert!(list.insert_before(1, &end).is_err());
    assert!(list.insert_after(1, &end).is_err());
    assert!(list.emplace_before(&end, || 1).is_err());
    assert!(list.emplace_after(&end, || 1).is_err());
    assert!(list.is_empty());
}
#[test]
fn list_cursor_edits_follow_native_positions() {
    let mut list: List<i32> = [10, 30].into_iter().collect();
    let mut cursor = ListCursor::default();
    list.append_at(40, &mut cursor);
    assert_eq!(cursor.value(&list), Some(&40));
    cursor.init(&list);
    assert_eq!(cursor.value(&list), Some(&10));
    *list.last_mut().unwrap() += 1;
    assert_eq!(items(&list), [10, 30, 41]);
    assert!(list.contains(&30) && !list.contains(&40));
    for value in &mut list {
        *value *= 2;
    }
    assert_eq!(items(&list), [20, 60, 82]);
    assert_eq!(list.length(), 3);
}
#[test]
fn list_insert_list_after_splices_or_appends_at_end() {
    let mut list: List<i32> = [1, 4].into_iter().collect();
    let mut source: List<i32> = [2, 3].into_iter().collect();
    let cursor = list.cursor();
    list.insert_list_after(&mut source, &cursor).unwrap();
    assert_eq!(items(&list), [1, 2, 3, 4]);
    assert!(source.is_empty());
    let other = BaseAllocator::incremental(128);
    let mut foreign = List::from_iter_in([5, 6], other.clone());
    list.insert_list_after(&mut foreign, &ListCursor::default())
        .unwrap();
    assert_eq!(items(&list), [1, 2, 3, 4, 5, 6]);
    assert!(foreign.is_empty());
    assert_eq!(foreign.allocator(), &other);
}
#[test]
fn list_insert_list_before_end_cursor_depends_on_allocator_like_occt() {
    let mut list: List<i32> = [1].into_iter().collect();
    let end = ListCursor::default();
    // Another allocator's empty list is "copied" without checking the position.
    let mut foreign = List::with_allocator(BaseAllocator::incremental(64));
    assert!(list.insert_list_before(&mut foreign, &end).is_ok());
    let mut same: List<i32> = List::default();
    assert!(list.insert_list_before(&mut same, &end).is_err());
    assert_eq!(items(&list), [1]);
}
#[test]
fn list_move_and_clear_keep_native_allocator_rules() {
    let arena = BaseAllocator::incremental(128);
    let mut source = List::from_iter_in([1, 2], arena.clone());
    let moved = List::move_from(&mut source);
    assert_eq!(items(&moved), [1, 2]);
    assert!(source.is_empty());
    assert_eq!(source.allocator(), &arena);
    let other = BaseAllocator::incremental(64);
    source.clear_with_allocator(other.clone());
    assert_eq!(source.allocator(), &other);
}
#[test]
fn sequence_indices_raise_out_of_range() {
    let mut sequence: Sequence<i32> = [1, 2, 3].into_iter().collect();
    for failure in [
        sequence.value(0).unwrap_err(),
        sequence.value(4).unwrap_err(),
        sequence.at(3).unwrap_err(),
        sequence.remove_range(2, 4).unwrap_err(),
        sequence.remove_range(3, 2).unwrap_err(),
        sequence.exchange(0, 1).unwrap_err(),
        sequence.insert_before(0, 9).map(|_| ()).unwrap_err(),
        sequence.insert_after(4, 9).map(|_| ()).unwrap_err(),
        sequence.emplace_after(4, || 9).map(|_| ()).unwrap_err(),
    ] {
        assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    }
    assert_eq!(values(&sequence), [1, 2, 3]);
    assert_eq!(*sequence.at(0).unwrap(), 1);
    *sequence.change_at(2).unwrap() = 30;
    *sequence.first_mut().unwrap() = 10;
    *sequence.last_mut().unwrap() += 1;
    assert_eq!(values(&sequence), [10, 2, 31]);
    for value in &mut sequence {
        *value += 1;
    }
    assert_eq!((&sequence).into_iter().sum::<i32>(), 46);
}
#[test]
fn sequence_split_clears_destination_before_checking_index() {
    let arena = BaseAllocator::incremental(128);
    let mut sequence = Sequence::with_allocator(arena.clone());
    sequence.append(1);
    let mut tail: Sequence<i32> = [7, 8].into_iter().collect();
    assert!(sequence.split(5, &mut tail).is_err());
    assert!(tail.is_empty());
    assert_eq!(tail.allocator(), &arena);
}
#[test]
fn sequence_splices_check_ranges_like_occt() {
    let mut sequence: Sequence<i32> = [1, 2].into_iter().collect();
    let mut empty = Sequence::default();
    // An empty source returns before InsertAfter checks the index.
    assert!(sequence.insert_sequence_after(9, &mut empty).is_ok());
    assert!(sequence.insert_sequence_before(0, &mut empty).is_err());
    let mut more: Sequence<i32> = [5].into_iter().collect();
    assert!(sequence.insert_sequence_after(3, &mut more).is_err());
    sequence.insert_sequence_before(2, &mut more).unwrap();
    assert_eq!(values(&sequence), [1, 5, 2]);
}
#[test]
fn sequence_cursor_walks_both_directions_from_either_end() {
    let sequence: Sequence<i32> = [1, 2, 3].into_iter().collect();
    let mut cursor = SequenceCursor::new(&sequence, false);
    assert!(!cursor.more());
    cursor.previous(&sequence);
    assert_eq!(cursor.value(&sequence), Some(&3));
    cursor.previous(&sequence);
    assert_eq!(cursor.value(&sequence), Some(&2));
    let mut other = sequence.cursor();
    other.next(&sequence);
    // Equality ignores the remembered previous node, as OCCT's IsEqual does.
    assert_eq!(cursor, other);
    cursor.init(&sequence, true);
    cursor.previous(&sequence);
    assert!(!cursor.more());
}
#[test]
fn sequence_cursor_edits_advance_or_prepend() {
    let mut sequence: Sequence<i32> = [1, 2, 3].into_iter().collect();
    let mut cursor = sequence.cursor();
    cursor.next(&sequence);
    sequence.remove_cursor(&mut cursor).unwrap();
    assert_eq!(cursor.value(&sequence), Some(&3));
    let mut end = SequenceCursor::default();
    sequence.remove_cursor(&mut end).unwrap();
    sequence.insert_after_cursor(&end, 0).unwrap();
    sequence.insert_after_cursor(&cursor, 4).unwrap();
    sequence.emplace_after_cursor(&end, || -1).unwrap();
    assert_eq!(values(&sequence), [-1, 0, 1, 3, 4]);
}
#[test]
fn hsequence_shares_one_sequence_through_handles() {
    let arena = BaseAllocator::incremental(128);
    let mut initial = Sequence::with_allocator(arena.clone());
    initial.append(1);
    let first = Handle::new(HSequence::from_sequence(&initial));
    let shared = first.clone();
    shared.get().unwrap().append(2);
    let mut more: Sequence<i32> = [3].into_iter().collect();
    first.get().unwrap().append_sequence(&mut more);
    assert!(more.is_empty());
    first.get().unwrap().sequence(|items| {
        assert_eq!(values(items), [1, 2, 3]);
        assert_eq!(items.allocator(), &arena);
    });
    first
        .get()
        .unwrap()
        .change_sequence(|items| items.reverse());
    assert_eq!(shared.get().unwrap().sequence(values), [3, 2, 1]);
}
#[test]
fn hsequence_append_handle_moves_items_and_rejects_null() {
    let target = HSequence::<i32>::default();
    let source = Handle::new(HSequence::<i32>::default());
    source.get().unwrap().append(7);
    target.append_handle(&source).unwrap();
    assert_eq!(target.sequence(values), [7]);
    assert!(source.get().unwrap().sequence(Sequence::is_empty));
    let null: Handle<HSequence<i32>> = Handle::default();
    assert_eq!(
        target.append_handle(&null).unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    let own = Handle::new(HSequence::<i32>::default());
    own.get().unwrap().append(1);
    own.get().unwrap().append_handle(&own).unwrap();
    assert_eq!(own.get().unwrap().sequence(values), [1]);
}
#[test]
fn hsequence_rtti_is_per_item_type() {
    let integers = HSequence::<i32>::static_type();
    let reals = HSequence::<f64>::static_type();
    assert_eq!(integers.name(), "NCollection_HSequence");
    assert_ne!(integers.rust_type(), reals.rust_type());
    assert!(std::ptr::eq(integers, HSequence::<i32>::static_type()));
    let handle = Handle::new(HSequence::<i32>::default());
    assert!(handle.is_kind_name("Standard_Transient"));
}
