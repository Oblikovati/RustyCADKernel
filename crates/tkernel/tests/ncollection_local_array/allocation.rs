fn local_fill<const N: usize>(array: &mut LocalArray<i32, N>, multiplier: i32, offset: i32) {
    for (index, value) in array.iter_mut().enumerate() { *value = index as i32 * multiplier + offset; }
}
fn local_check<const N: usize>(array: &LocalArray<i32, N>, count: usize, multiplier: i32, offset: i32) {
    for i in 0..count { assert_eq!(i as i32 * multiplier + offset, array[i]); }
}

#[test]
fn NCollection_LocalArrayTest__STLIterators() {
    let mut array = LocalArray::<i32, 4>::new(3);
    array[0] = 10; array[1] = 20; array[2] = 30;
    assert_eq!(60, array.iter().sum::<i32>());
    for value in &mut array { *value /= 10; }
    let borrowed = &array;
    assert_eq!(borrowed.iter().as_slice().as_ptr_range(), array.as_slice().as_ptr_range());
    let found = borrowed.iter().find(|&&v| v == 2);
    assert!(found.is_some());
    assert_eq!(2, *found.unwrap());
    array.as_mut_slice().reverse();
    assert_eq!(3, array[0]); assert_eq!(1, array[2]);
}

#[test]
fn NCollection_LocalArrayTest__ConstructorWithSize() {
    let array = LocalArray::<i32>::new(100);
    assert_eq!(100, array.len());
}

#[test]
fn NCollection_LocalArrayTest__SmallSizeAllocation() {
    let mut array = LocalArray::<i32, 1024>::new(10);
    assert_eq!(10, array.len());
    local_fill(&mut array, 10, 0);
    local_check(&array, 10, 10, 0);
}

#[test]
fn NCollection_LocalArrayTest__LargeSizeAllocation() {
    let mut array = LocalArray::<i32>::new(2000);
    assert_eq!(2000, array.len());
    local_fill(&mut array, 10, 0);
    local_check(&array, 2000, 10, 0);
}

#[test]
fn NCollection_LocalArrayTest__Reallocation() {
    let mut array = LocalArray::<i32>::new(10);
    assert_eq!(10, array.len());
    local_fill(&mut array, 1, 0);
    array.allocate(50);
    assert_eq!(50, array.len());
    local_fill(&mut array, 2, 0);
    local_check(&array, 50, 2, 0);
    array.allocate(5);
    assert_eq!(5, array.len());
    local_check(&array, 5, 2, 0);
}

#[test]
fn NCollection_LocalArrayTest__CustomMaxArraySize() {
    let small = LocalArray::<i32, 50>::new(20);
    assert_eq!(20, small.len());
    let large = LocalArray::<i32, 50>::new(100);
    assert_eq!(100, large.len());
}

#[derive(Default, PartialEq)]
struct LocalCustom { a: i32, b: f64 }

#[test]
#[allow(clippy::approx_constant)]
fn NCollection_LocalArrayTest__CustomType() {
    let mut array = LocalArray::<LocalCustom>::new(5);
    assert_eq!(5, array.len());
    array[0] = LocalCustom { a: 10, b: 3.14 };
    assert!(LocalCustom { a: 10, b: 3.14 } == array[0]);
}

#[test]
fn NCollection_LocalArrayTest__TransitionStackToHeap() {
    let mut array = LocalArray::<i32, 10>::default();
    array.allocate(5);
    local_fill(&mut array, 1, 0); local_check(&array, 5, 1, 0);
    array.allocate(20);
    local_fill(&mut array, 3, 0); local_check(&array, 20, 3, 0);
    array.allocate(5);
    local_fill(&mut array, 5, 0); local_check(&array, 5, 5, 0);
}

#[test]
fn NCollection_LocalArrayTest__ReallocateStackToStack_WithCopy() {
    let mut array = LocalArray::<i32, 64>::new(10);
    local_fill(&mut array, 100, 0);
    array.reallocate(32, true);
    assert_eq!(32, array.len());
    local_check(&array, 10, 100, 0);
}

#[test]
fn NCollection_LocalArrayTest__ReallocateStackToHeap_WithCopy() {
    let mut array = LocalArray::<i32, 8>::new(8);
    local_fill(&mut array, 1, 1);
    array.reallocate(16, true);
    assert_eq!(16, array.len());
    local_check(&array, 8, 1, 1);
}

#[test]
fn NCollection_LocalArrayTest__ReallocateHeapToHeap_WithCopy() {
    let mut array = LocalArray::<i32, 4>::new(8);
    local_fill(&mut array, 10, 0);
    array.reallocate(16, true);
    assert_eq!(16, array.len());
    local_check(&array, 8, 10, 0);
}

#[test]
fn NCollection_LocalArrayTest__ReallocateNoCopy() {
    let mut array = LocalArray::<i32, 8>::new(8);
    local_fill(&mut array, 1, 1);
    array.reallocate(16, false);
    assert_eq!(16, array.len());
}

#[test]
fn NCollection_LocalArrayTest__ReallocateShrink() {
    let mut array = LocalArray::<i32, 8>::new(16);
    local_fill(&mut array, 5, 0);
    array.reallocate(4, true);
    assert_eq!(4, array.len());
    local_check(&array, 4, 5, 0);
}
