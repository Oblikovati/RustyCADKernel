#[test]
fn NCollection_SequenceTest__BasicFunctions() {
    let mut sequence = Sequence::default();
    assert!(sequence.is_empty());
    assert_eq!(sequence.len(), 0);
    assert_eq!(sequence.length(), 0);
    for value in [10, 20, 30] {
        sequence.append(value);
    }
    equal(&sequence, &[10, 20, 30]);
    assert!(!sequence.is_empty());
    assert_eq!(sequence.first(), Some(&10));
    assert_eq!(sequence.last(), Some(&30));
    assert_eq!(sequence.lower(), 1);
    assert_eq!(sequence.upper(), 3);
}
#[test]
fn NCollection_SequenceTest__StandardIteratorContract() {
    let mut sequence = seq(&[1, 2]);
    let constant = &sequence;
    assert!(sequence.iter() == constant.iter());
    assert!(sequence.iter().nth(2).is_none());
    assert!(constant.iter().nth(2).is_none());
    for value in sequence.iter_mut() {
        *value += 1;
    }
    equal(&sequence, &[2, 3]);
    assert_eq!(sequence.iter().find(|&&value| value == 3), Some(&3));
    let mut iter = sequence.iter_mut();
    std::mem::swap(iter.next().unwrap(), iter.next_back().unwrap());
    equal(&sequence, &[3, 2]);
}
#[test]
fn NCollection_SequenceTest__ModifyingOperations() {
    let mut sequence = Sequence::default();
    sequence.prepend(100);
    sequence.prepend(200);
    assert_eq!(sequence.len(), 2);
    assert_eq!(sequence.first(), Some(&200));
    assert_eq!(sequence.last(), Some(&100));
    sequence.set_value(1, 210).unwrap();
    assert_eq!(sequence[1], 210);
    *sequence.change_value(2).unwrap() = 110;
    assert_eq!(sequence[2], 110);
    sequence.insert_before(1, 300).unwrap();
    equal(&sequence, &[300, 210, 110]);
    sequence.insert_after(2, 400).unwrap();
    equal(&sequence, &[300, 210, 400, 110]);
    sequence.remove(3).unwrap();
    equal(&sequence, &[300, 210, 110]);
    sequence.append(500);
    sequence.append(600);
    assert_eq!(sequence.len(), 5);
    sequence.remove_range(2, 4).unwrap();
    equal(&sequence, &[300, 600]);
}
#[test]
fn NCollection_SequenceTest__IteratorFunctions() {
    let mut sequence = seq(&[10, 20, 30]);
    let mut cursor = sequence.cursor();
    for value in [10, 20, 30] {
        assert!(cursor.more());
        assert_eq!(cursor.value(&sequence), Some(&value));
        cursor.next(&sequence);
    }
    assert!(!cursor.more());
    let mut cursor = sequence.cursor();
    *cursor.change_value(&mut sequence).unwrap() = 15;
    cursor.next(&sequence);
    *cursor.change_value(&mut sequence).unwrap() = 25;
    equal(&sequence, &[15, 25, 30]);
    for (index, value) in sequence.iter().enumerate() {
        assert_eq!(*value, [15, 25, 30][index]);
    }
    assert_eq!(sequence.iter().count(), 3);
}
#[test]
fn NCollection_SequenceTest__AdvancedOperations() {
    let mut sequence = seq(&[10, 20, 30, 40, 50]);
    sequence.exchange(2, 4).unwrap();
    equal(&sequence, &[10, 40, 30, 20, 50]);
    sequence.reverse();
    equal(&sequence, &[50, 20, 30, 40, 10]);
    let mut second = Sequence::default();
    sequence.split(2, &mut second).unwrap();
    equal(&sequence, &[50]);
    equal(&second, &[20, 30, 40, 10]);
    sequence.clear(None);
    assert!(sequence.is_empty());
    assert_eq!(sequence.len(), 0);
}
#[derive(Clone)]
struct TestClass(i32, String);
#[test]
fn NCollection_SequenceTest__ComplexTypeSequence() {
    let mut sequence = Sequence::default();
    for (id, name) in [(1, "First"), (2, "Second"), (3, "Third")] {
        sequence.append(TestClass(id, name.into()));
    }
    assert_eq!(sequence.len(), 3);
    for (index, name) in [(1, "First"), (2, "Second"), (3, "Third")] {
        assert_eq!(sequence[index].0, index as i32);
        assert_eq!(sequence[index].1, name);
    }
    *sequence.change_value(2).unwrap() = TestClass(22, "Modified".into());
    assert_eq!(sequence[2].0, 22);
    assert_eq!(sequence[2].1, "Modified");
    sequence.remove(1).unwrap();
    assert_eq!(sequence.len(), 2);
    assert_eq!(sequence.first().unwrap().0, 22);
}
#[test]
fn NCollection_SequenceTest__STLAlgorithmCompatibility_MinMax() {
    let values = random_values();
    let sequence = seq(&values);
    assert_eq!(sequence.iter().min(), values.iter().min());
    assert_eq!(sequence.iter().max(), values.iter().max());
}
#[test]
fn NCollection_SequenceTest__STLAlgorithmCompatibility_Replace() {
    let mut values = random_values();
    let mut sequence = seq(&values);
    let target = *values.last().unwrap();
    for value in sequence.iter_mut() {
        if *value == target {
            *value = -1;
        }
    }
    for value in &mut values {
        if *value == target {
            *value = -1;
        }
    }
    equal(&sequence, &values);
}
#[test]
fn NCollection_SequenceTest__STLAlgorithmCompatibility_Reverse() {
    let mut sequence: Sequence<i32> = (0..100).collect();
    let mut values: Vec<i32> = (0..100).collect();
    let mut iter = sequence.iter_mut();
    while iter.len() > 1 {
        std::mem::swap(iter.next().unwrap(), iter.next_back().unwrap());
    }
    values.reverse();
    equal(&sequence, &values);
}
