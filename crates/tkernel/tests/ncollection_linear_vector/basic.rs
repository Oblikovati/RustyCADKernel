#[test]
fn NCollection_LinearVectorTest__DefaultConstructor() {
    let vector = LinearVector::<i32>::default();
    assert_eq!(vector.len(), 0);
    assert!(vector.is_empty());
    assert_eq!(vector.capacity(), 0);
}
#[test]
fn NCollection_LinearVectorTest__ReserveConstructor() {
    let vector = LinearVector::<i32>::with_capacity(100);
    assert_eq!(vector.len(), 0);
    assert!(vector.is_empty());
    assert!(vector.capacity() >= 100);
}
#[test]
fn NCollection_LinearVectorTest__Append() {
    let vector = integers(&[10, 20, 30]);
    same_values(&vector, &[10, 20, 30]);
    assert!(!vector.is_empty());
}
#[test]
fn NCollection_LinearVectorTest__AppendMove() {
    let mut vector = LinearVector::default();
    let mut source = "hello".to_owned();
    vector.emplace_append_with_relocation(std::mem::take, || std::mem::take(&mut source));
    assert_eq!(vector.len(), 1);
    assert_eq!(vector[0], "hello");
    assert!(source.is_empty());
}
#[test]
fn NCollection_LinearVectorTest__Appended() {
    let mut vector = LinearVector::default();
    *vector.appended() = 42;
    same_values(&vector, &[42]);
}
#[test]
fn NCollection_LinearVectorTest__EmplaceAppend() {
    let mut vector = LinearVector::default();
    let reference = vector.emplace_append(|| MultiArg {
        integer: 5,
        real: 3.14,
    });
    assert_eq!(reference.integer, 5);
    assert!((reference.real - 3.14).abs() < 1e-10);
    assert_eq!(vector.len(), 1);
}
#[test]
fn NCollection_LinearVectorTest__SetValue() {
    let mut vector = integers(&[10, 20]);
    vector.set_value(0, 99);
    assert_eq!(vector[0], 99);
    vector.set_value(5, 55);
    assert_eq!(vector.len(), 6);
    assert_eq!(vector[5], 55);
    assert_eq!(vector[3], 0);
}
#[test]
fn NCollection_LinearVectorTest__ValueAccess() {
    let mut vector = integers(&[10, 20, 30]);
    for (index, value) in [10, 20, 30].into_iter().enumerate() {
        assert_eq!(*vector.value(index).unwrap(), value);
    }
    *vector.change_value(1).unwrap() = 99;
    assert_eq!(vector[1], 99);
    for index in 0..3 {
        assert_eq!(*vector.value(index).unwrap(), vector[index]);
    }
    vector[2] = 77;
    assert_eq!(*vector.value(2).unwrap(), 77);
}
#[test]
fn NCollection_LinearVectorTest__FirstLast() {
    let mut vector = integers(&[10, 20, 30]);
    assert_eq!(*vector.first().unwrap(), 10);
    assert_eq!(*vector.last().unwrap(), 30);
    *vector.change_first().unwrap() = 99;
    *vector.change_last().unwrap() = 88;
    assert_eq!(vector[0], 99);
    assert_eq!(vector[2], 88);
}
#[test]
fn NCollection_LinearVectorTest__CopyConstructor() {
    let vector = integers(&[1, 2, 3]);
    let mut copy = vector.clone_trivial();
    same_values(&copy, &[1, 2, 3]);
    copy[0] = 99;
    assert_eq!(vector[0], 1);
}
#[test]
fn NCollection_LinearVectorTest__CopyAssignment() {
    let vector = integers(&[1, 2]);
    let mut copy = integers(&[100]);
    copy.assign_trivial(&vector);
    same_values(&copy, &[1, 2]);
    copy[0] = 99;
    assert_eq!(vector[0], 1);
}
fn check_moved_values(destination: &LinearVector<i32>, source: &LinearVector<i32>) {
    same_values(destination, &[1, 2]);
    assert!(source.is_empty());
    assert_eq!(source.len(), 0);
}
#[test]
fn NCollection_LinearVectorTest__MoveConstructor() {
    let mut source = integers(&[1, 2]);
    let destination = LinearVector::from_moved(&mut source);
    check_moved_values(&destination, &source);
}
#[test]
fn NCollection_LinearVectorTest__MoveAssignment() {
    let mut source = integers(&[1, 2]);
    let mut destination = integers(&[100]);
    destination.move_from(&mut source);
    check_moved_values(&destination, &source);
}
#[test]
fn NCollection_LinearVectorTest__Reserve() {
    let mut vector = integers(&[1, 2]);
    vector.reserve(100);
    same_values(&vector, &[1, 2]);
    assert!(vector.capacity() >= 100);
    vector.reserve(5);
    assert!(vector.capacity() >= 100);
}
#[test]
fn NCollection_LinearVectorTest__Resize() {
    let mut vector = integers(&[1, 2, 3]);
    vector.resize(5);
    same_values(&vector, &[1, 2, 3, 0, 0]);
    vector.resize(2);
    same_values(&vector, &[1, 2]);
}
#[test]
fn NCollection_LinearVectorTest__Resize_WithValue() {
    let mut vector = integers(&[1, 2]);
    vector.resize_with_value(5, -1);
    same_values(&vector, &[1, 2, -1, -1, -1]);
    vector.resize_with_value(3, -1);
    same_values(&vector, &[1, 2, -1]);
}
#[test]
fn NCollection_LinearVectorTest__EraseLast() {
    let mut vector = integers(&[1, 2, 3]);
    vector.erase_last();
    same_values(&vector, &[1, 2]);
}
