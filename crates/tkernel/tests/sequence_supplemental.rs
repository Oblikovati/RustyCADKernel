//! Sequence cursor identity and allocator transfer boundaries.
use rustycad_tkernel::ncollection::{BaseAllocator, Sequence};
#[test]
fn zero_sized_sequence_nodes_have_distinct_cursor_positions() {
    let mut sequence = Sequence::default();
    sequence.append(());
    sequence.append(());
    let mut cursor = sequence.cursor();
    let first = cursor;
    cursor.next(&sequence);
    assert!(cursor.more());
    assert_ne!(cursor, first);
    cursor.next(&sequence);
    assert!(!cursor.more());
}
#[test]
fn sequence_split_and_cross_allocator_splice_preserve_ownership() {
    let first = BaseAllocator::incremental(128);
    let second = BaseAllocator::incremental(128);
    let mut sequence = Sequence::with_allocator(first.clone());
    sequence.append(1);
    sequence.append(2);
    let mut tail = Sequence::with_allocator(second.clone());
    tail.append(3);
    sequence.split(2, &mut tail);
    assert_eq!(tail.allocator(), &first);
    second.reset(true).unwrap();
    let mut destination = Sequence::with_allocator(second.clone());
    destination.append_sequence(&mut tail);
    assert!(tail.is_empty());
    assert_eq!(tail.allocator(), &first);
    assert_eq!(destination[1], 2);
    sequence.clear(None);
    first.reset(true).unwrap();
    destination[1] = 4;
    assert_eq!(destination[1], 4);
    assert_eq!(destination.iter().next_back(), Some(&4));
}
