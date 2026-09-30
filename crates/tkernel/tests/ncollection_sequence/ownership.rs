#[test]
fn NCollection_SequenceTest__CopyAndAssignment() {
    let mut first = seq(&[10, 20, 30]);
    let second = first.clone();
    equal(&second, &[10, 20, 30]);
    let mut third = Sequence::default();
    third.assign(&first);
    equal(&third, &[10, 20, 30]);
    first.set_value(2, 25).unwrap();
    assert_eq!(first[2], 25);
    assert_eq!(second[2], 20);
    assert_eq!(third[2], 20);
}
#[test]
fn NCollection_SequenceTest__CombiningSequences() {
    let first = seq(&[10, 20]);
    let mut second = seq(&[30, 40]);
    let mut third = first.clone();
    third.append_sequence(&mut second);
    equal(&third, &[10, 20, 30, 40]);
    assert!(second.is_empty());
    second.append(50);
    second.append(60);
    let mut fourth = seq(&[70]);
    fourth.prepend_sequence(&mut second);
    equal(&fourth, &[50, 60, 70]);
    assert!(second.is_empty());
    second.append(80);
    second.append(90);
    let mut fifth = seq(&[100, 110]);
    fifth.insert_sequence_after(1, &mut second).unwrap();
    equal(&fifth, &[100, 80, 90, 110]);
    assert!(second.is_empty());
}
#[test]
fn NCollection_SequenceTest__AllocatorTest() {
    let mut sequence = Sequence::with_allocator(BaseAllocator::incremental(12 * 1024));
    for value in [10, 20, 30] {
        sequence.append(value);
    }
    equal(&sequence, &[10, 20, 30]);
    sequence.clear(Some(BaseAllocator::incremental(12 * 1024)));
    assert!(sequence.is_empty());
    sequence.append(40);
    assert_eq!(sequence[1], 40);
}
#[test]
fn NCollection_SequenceTest__MoveOperations() {
    let mut first = seq(&[10, 20, 30]);
    let second = Sequence::move_from(&mut first);
    assert!(first.is_empty());
    equal(&second, &[10, 20, 30]);
    let mut third = seq(&[40]);
    let mut fourth = Sequence::default();
    fourth.move_assign(&mut third);
    assert!(third.is_empty());
    equal(&fourth, &[40]);
}
#[test]
fn NCollection_SequenceTest__OCC26448_PrependEmptySequence() {
    let mut first = Sequence::default();
    let mut second = Sequence::default();
    first.append(11.0);
    first.prepend_sequence(&mut second);
    assert_eq!(first.len(), 1);
    assert_eq!(first.first(), Some(&11.0));
    let mut third = Sequence::default();
    let mut fourth = Sequence::default();
    third.append(11.0);
    third.prepend_sequence(&mut fourth);
    assert_eq!(third.len(), 1);
    assert_eq!(third.first(), Some(&11.0));
}
