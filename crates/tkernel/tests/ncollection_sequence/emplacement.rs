struct Pair(i32, f64);
struct MoveOnly(i32);
fn check_pair(pair: &Pair, integer: i32, real: f64) {
    assert_eq!(pair.0, integer);
    assert!((pair.1 - real).abs() < 1e-10);
}
fn check_order(sequence: &Sequence<Pair>) {
    assert_eq!(sequence.len(), 3);
    assert_eq!(sequence[1].0, 10);
    assert_eq!(sequence[2].0, 20);
    assert_eq!(sequence[3].0, 30);
}
fn pair_sequence() -> Sequence<Pair> {
    let mut sequence = Sequence::default();
    sequence.emplace_append(|| Pair(10, 1.0));
    sequence.emplace_append(|| Pair(30, 3.0));
    sequence
}
#[test]
fn NCollection_SequenceTest__EmplaceAppend() {
    let mut sequence = Sequence::default();
    check_pair(sequence.emplace_append(|| Pair(42, 3.14)), 42, 3.14);
    assert_eq!(sequence.len(), 1);
    check_pair(sequence.emplace_append(|| Pair(100, 2.71)), 100, 2.71);
    assert_eq!(sequence.len(), 2);
    assert_eq!(sequence[1].0, 42);
    assert_eq!(sequence[2].0, 100);
}
#[test]
fn NCollection_SequenceTest__EmplacePrepend() {
    let mut sequence = Sequence::default();
    check_pair(sequence.emplace_prepend(|| Pair(42, 3.14)), 42, 3.14);
    assert_eq!(sequence.len(), 1);
    check_pair(sequence.emplace_prepend(|| Pair(100, 2.71)), 100, 2.71);
    assert_eq!(sequence.len(), 2);
    assert_eq!(sequence.first().unwrap().0, 100);
    assert_eq!(sequence.last().unwrap().0, 42);
}
#[test]
fn NCollection_SequenceTest__EmplaceAfterIterator() {
    let mut sequence = pair_sequence();
    let cursor = sequence.cursor();
    check_pair(
        sequence.emplace_after_cursor(&cursor, || Pair(20, 2.0)).unwrap(),
        20,
        2.0,
    );
    check_order(&sequence);
}
#[test]
fn NCollection_SequenceTest__EmplaceAfterIndex() {
    let mut sequence = pair_sequence();
    check_pair(sequence.emplace_after(1, || Pair(20, 2.0)).unwrap(), 20, 2.0);
    check_order(&sequence);
}
#[test]
fn NCollection_SequenceTest__EmplaceBeforeIndex() {
    let mut sequence = pair_sequence();
    check_pair(sequence.emplace_before(2, || Pair(20, 2.0)).unwrap(), 20, 2.0);
    check_order(&sequence);
}
#[test]
fn NCollection_SequenceTest__EmplaceWithMoveOnlyType() {
    let mut sequence = Sequence::default();
    assert_eq!(sequence.emplace_append(|| MoveOnly(42)).0, 42);
    assert_eq!(sequence.emplace_prepend(|| MoveOnly(100)).0, 100);
    assert_eq!(sequence.len(), 2);
    assert_eq!(sequence.first().unwrap().0, 100);
    assert_eq!(sequence.last().unwrap().0, 42);
}
