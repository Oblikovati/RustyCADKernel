#[test]
fn NCollection_LinearVectorTest__EraseSingle() {
    let mut vector = integers(&[10, 20, 30, 40]);
    vector.erase(1).unwrap();
    same_values(&vector, &[10, 30, 40]);
    vector.erase(0).unwrap();
    same_values(&vector, &[30, 40]);
}
#[test]
fn NCollection_LinearVectorTest__EraseRange() {
    let mut vector = integers(&[0, 10, 20, 30, 40, 50]);
    vector.erase_range(1, 4).unwrap();
    same_values(&vector, &[0, 40, 50]);
}
#[test]
fn NCollection_LinearVectorTest__EraseSingle_NonTrivial_HeapBuffers() {
    let mut vector = LinearVector::default();
    for prefix in ["alpha", "beta", "gamma", "delta"] {
        vector.append(AsciiString::new(&format!(
            "{prefix}-string-forces-heap-allocation"
        )));
    }
    vector.erase_with_move(1, AsciiString::move_assign).unwrap();
    assert_eq!(vector.len(), 3);
    for (index, prefix) in ["alpha", "gamma", "delta"].into_iter().enumerate() {
        assert_eq!(
            vector[index].as_bytes(),
            format!("{prefix}-string-forces-heap-allocation").as_bytes()
        );
    }
    vector.erase_with_move(0, AsciiString::move_assign).unwrap();
    assert_eq!(vector.len(), 2);
    for (index, prefix) in ["gamma", "delta"].into_iter().enumerate() {
        assert_eq!(
            vector[index].as_bytes(),
            format!("{prefix}-string-forces-heap-allocation").as_bytes()
        );
    }
}
#[test]
fn NCollection_LinearVectorTest__EraseRange_NonTrivial_HeapBuffers() {
    let mut vector = LinearVector::default();
    for index in 0..6 {
        let mut entry = AsciiString::new("payload-with-heap-allocation-");
        entry.append(&AsciiString::from_integer(index));
        vector.append(entry);
    }
    vector.erase_range_with_move(1, 4, AsciiString::move_assign).unwrap();
    assert_eq!(vector.len(), 3);
    for (index, value) in [0, 4, 5].into_iter().enumerate() {
        assert_eq!(
            vector[index].as_bytes(),
            format!("payload-with-heap-allocation-{value}").as_bytes()
        );
    }
}
#[test]
fn NCollection_LinearVectorTest__InsertBefore() {
    let mut vector = integers(&[10, 30]);
    vector.insert_before(0, 5).unwrap();
    same_values(&vector, &[5, 10, 30]);
    vector.insert_before(2, 20).unwrap();
    same_values(&vector, &[5, 10, 20, 30]);
    vector.insert_before(4, 40).unwrap();
    assert_eq!(vector.len(), 5);
    assert_eq!(vector[4], 40);
}
#[test]
fn NCollection_LinearVectorTest__InsertAfter() {
    let mut vector = integers(&[10, 30]);
    vector.insert_after(0, 20).unwrap();
    same_values(&vector, &[10, 20, 30]);
}
#[test]
fn NCollection_LinearVectorTest__Clear() {
    let mut vector = integers(&[1, 2, 3]);
    let capacity = vector.capacity();
    vector.clear(false);
    assert_eq!(vector.len(), 0);
    assert!(vector.is_empty());
    assert_eq!(vector.capacity(), capacity);
    vector.append(10);
    vector.clear(true);
    assert_eq!(vector.len(), 0);
    assert_eq!(vector.capacity(), 0);
}
#[test]
fn NCollection_LinearVectorTest__IteratorRangeFor() {
    let mut vector = integers(&[1, 2, 3]);
    assert_eq!(vector.iter().sum::<i32>(), 6);
    for value in &mut vector {
        *value *= 10;
    }
    same_values(&vector, &[10, 20, 30]);
}
#[test]
fn NCollection_LinearVectorTest__STLAlgorithms() {
    let mut vector = integers(&[30, 10, 20]);
    vector.as_mut_slice().sort();
    same_values(&vector, &[10, 20, 30]);
    assert_eq!(vector.iter().find(|v| **v == 20), Some(&20));
    vector.as_mut_slice().reverse();
    same_values(&vector, &[30, 20, 10]);
    let shared = &vector;
    assert_eq!(shared.iter().find(|v| **v == 20), Some(&20));
}
#[test]
fn NCollection_LinearVectorTest__TrivialTypeGrowth() {
    let mut vector = LinearVector::default();
    for index in 0..100 {
        vector.append(f64::from(index));
    }
    assert_eq!(vector.len(), 100);
    for index in 0..100 {
        assert!((vector[index] - index as f64).abs() < 1e-15);
    }
}
#[test]
#[allow(unsafe_code)]
fn NCollection_LinearVectorTest__NonTrivialType() {
    let vector = strings(&["hello", "world", "test"]);
    same_values(&vector, &["hello".into(), "world".into(), "test".into()]);
    let mut copy = vector.clone();
    assert_eq!(copy.len(), 3);
    assert_eq!(copy[0], "hello");
    let mut moved = LinearVector::default();
    moved.move_from(&mut copy);
    assert_eq!(moved.len(), 3);
    assert_eq!(moved[0], "hello");
    assert!(copy.is_empty());
    moved.erase(1).unwrap();
    same_values(&moved, &["hello".into(), "test".into()]);
    let inserted = String::from("inserted");
    // SAFETY: String destruction, taking owned buffers and moving the donor cannot unwind.
    unsafe {
        moved.insert_before_with_constructors(1, std::mem::take, std::mem::take, || inserted)
            .unwrap();
    }
    assert_eq!(moved.len(), 3);
    assert_eq!(moved[1], "inserted");
}
#[test]
fn NCollection_LinearVectorTest__EmptyOperations() {
    let mut vector = LinearVector::<i32>::default();
    vector.erase_last();
    assert_eq!(vector.len(), 0);
    vector.clear(false);
    assert_eq!(vector.len(), 0);
    vector.clear(true);
    assert_eq!(vector.len(), 0);
    assert_eq!(vector.capacity(), 0);
}
#[test]
fn NCollection_LinearVectorTest__LargeAppend() {
    let mut vector = LinearVector::default();
    for index in 0..10000 {
        vector.append(index);
    }
    assert_eq!(vector.len(), 10000);
    for index in 0..10000 {
        assert_eq!(vector[index], index);
    }
}
#[test]
fn NCollection_LinearVectorTest__MoveOnlyType() {
    let mut vector = LinearVector::default();
    for value in [1, 2, 3] {
        vector.append(MoveOnly { value });
    }
    assert_eq!(vector.len(), 3);
    for index in 0..3 {
        assert_eq!(vector[index].value, index as i32 + 1);
    }
    let mut moved = LinearVector::default();
    moved.move_from(&mut vector);
    assert_eq!(moved.len(), 3);
    assert_eq!(moved[0].value, 1);
    assert!(vector.is_empty());
}
#[test]
fn NCollection_LinearVectorTest__ResizeNonTrivial() {
    let mut vector = strings(&["a", "b"]);
    vector.resize(4);
    same_values(&vector, &["a".into(), "b".into(), "".into(), "".into()]);
    vector.resize(1);
    same_values(&vector, &["a".into()]);
}
#[test]
fn NCollection_LinearVectorTest__Resize_WithValue_NonTrivial() {
    let mut vector = strings(&["x"]);
    vector.resize_with_value(4, "hello".into());
    same_values(
        &vector,
        &["x".into(), "hello".into(), "hello".into(), "hello".into()],
    );
    vector.resize_with_value(2, "ignored".into());
    same_values(&vector, &["x".into(), "hello".into()]);
}
