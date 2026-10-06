#[test]
fn NCollection_LocalArrayTest__MoveConstructor_FromStack() {
    let mut source = LocalArray::<i32, 64>::new(10);
    local_fill(&mut source, 7, 0);
    let dest = std::mem::take(&mut source);
    assert_eq!(10, dest.len());
    local_check(&dest, 10, 7, 0);
    assert_eq!(0, source.len());
}

#[test]
fn NCollection_LocalArrayTest__MoveConstructor_FromHeap() {
    let mut source = LocalArray::<i32, 4>::new(16);
    local_fill(&mut source, 3, 0);
    let dest = std::mem::take(&mut source);
    assert_eq!(16, dest.len());
    local_check(&dest, 16, 3, 0);
    assert_eq!(0, source.len());
}

#[test]
fn NCollection_LocalArrayTest__MoveAssignment() {
    let mut source = LocalArray::<i32, 8>::new(8);
    local_fill(&mut source, 1, 100);
    let mut dest = LocalArray::<i32, 8>::new(4);
    dest.as_mut_slice().fill(-1);
    dest.move_from_trivial(&mut source);
    assert_eq!(8, dest.len());
    local_check(&dest, 8, 1, 100);
    assert_eq!(0, source.len());
}

#[test]
fn NCollection_LocalArrayTest__ReallocateAsGrowableStack() {
    let mut stack = LocalArray::<i32, 4>::new(4);
    let mut top = 0;
    for i in 0..20 {
        if top >= stack.len() { stack.reallocate(stack.len() * 2, true); }
        stack[top] = i * 3; top += 1;
    }
    assert_eq!(20, top);
    for i in (0..20).rev() { top -= 1; assert_eq!(i * 3, stack[top]); }
}

#[test]
fn NCollection_LocalArrayTest__ToArray1_SamePointer() {
    let mut source = LocalArray::<i32>::new(10);
    local_fill(&mut source, 10, 0);
    let pointer = source.as_slice().as_ptr();
    let view = source.to_array1();
    assert_eq!(10, view.len());
    assert_eq!(pointer, &view[0]);
}

#[test]
fn NCollection_LocalArrayTest__ToArray1_ModifyReflectsInSource() {
    let mut source = LocalArray::<i32>::new(5);
    local_fill(&mut source, 1, 0);
    let mut view = source.to_array1();
    view[0] = 99; view[4] = 77;
    assert_eq!(99, source[0]);
    assert_eq!(1, source[1]);
    assert_eq!(77, source[4]);
}

#[test]
fn NCollection_LocalArrayTest__ToArray1_EmptyVector() {
    let mut source = LocalArray::<i32>::default();
    let view = source.to_array1();
    assert_eq!(0, view.len());
    assert!(view.is_empty());
}

#[test]
fn NCollection_LocalArrayTest__ToArray1_HeapBuffer() {
    let mut source = LocalArray::<i32, 8>::new(16);
    local_fill(&mut source, 3, 0);
    let pointer = source.as_slice().as_ptr();
    let view = source.to_array1();
    assert_eq!(16, view.len());
    assert_eq!(pointer, &view[0]);
    assert_eq!(0, view[0]);
    assert_eq!(45, view[15]);
}

#[test]
fn NCollection_LocalArrayTest__ToArray1_ReadThrough() {
    let mut source = LocalArray::<i32>::new(7);
    local_fill(&mut source, 11, 0);
    let view = source.to_array1();
    for i in 0..7 { assert_eq!(i * 11, view[i]); }
}
