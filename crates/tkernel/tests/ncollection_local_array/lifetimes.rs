use std::cell::Cell;
thread_local! {
    static TRACKED_CTORS: Cell<i32> = const { Cell::new(0) };
    static TRACKED_DTORS: Cell<i32> = const { Cell::new(0) };
}
// Rust moves transfer ownership without a user-defined move constructor. The
// upstream ctor+move == dtor checks become ctor == dtor for native Rust moves.
struct LocalTracked { value: i32 }
impl Default for LocalTracked {
    fn default() -> Self {
        TRACKED_CTORS.set(TRACKED_CTORS.get() + 1);
        Self { value: 0 }
    }
}
impl Drop for LocalTracked {
    fn drop(&mut self) { TRACKED_DTORS.set(TRACKED_DTORS.get() + 1); }
}
fn reset_tracked() { TRACKED_CTORS.set(0); TRACKED_DTORS.set(0); }
fn tracked_fill<const N: usize>(array: &mut LocalArray<LocalTracked, N>, multiplier: i32, offset: i32) {
    for (index, item) in array.iter_mut().enumerate() { item.value = index as i32 * multiplier + offset; }
}
fn tracked_check<const N: usize>(array: &LocalArray<LocalTracked, N>, count: usize, multiplier: i32, offset: i32) {
    for i in 0..count { assert_eq!(i as i32 * multiplier + offset, array[i].value); }
}

#[test]
fn NCollection_LocalArrayTest__NonTrivial_SmallAllocation() {
    reset_tracked();
    {
        let mut array = LocalArray::<LocalTracked, 8>::new(4);
        assert_eq!(4, array.len()); assert_eq!(4, TRACKED_CTORS.get());
        array[0].value = 10; array[1].value = 20; array[2].value = 30; array[3].value = 40;
        assert_eq!(30, array[2].value);
    }
    assert_eq!(4, TRACKED_DTORS.get());
}

#[test]
fn NCollection_LocalArrayTest__NonTrivial_HeapAllocation() {
    reset_tracked();
    {
        let mut array = LocalArray::<LocalTracked, 4>::new(8);
        assert_eq!(8, array.len());
        tracked_fill(&mut array, 100, 0); tracked_check(&array, 8, 100, 0);
    }
    assert_eq!(TRACKED_CTORS.get(), TRACKED_DTORS.get());
}

#[test]
fn NCollection_LocalArrayTest__NonTrivial_ReallocateWithCopy_InlineToInline() {
    let mut array = LocalArray::<LocalTracked, 16>::new(4);
    tracked_fill(&mut array, 1, 1);
    array.reallocate(8, true);
    assert_eq!(8, array.len());
    tracked_check(&array, 4, 1, 1);
}

#[test]
fn NCollection_LocalArrayTest__NonTrivial_ReallocateWithCopy_InlineToHeap() {
    let mut array = LocalArray::<LocalTracked, 4>::new(4);
    tracked_fill(&mut array, 10, 0);
    array.reallocate(16, true);
    assert_eq!(16, array.len());
    tracked_check(&array, 4, 10, 0);
}

#[test]
fn NCollection_LocalArrayTest__NonTrivial_ReallocateWithCopy_HeapToHeap() {
    let mut array = LocalArray::<LocalTracked, 4>::new(8);
    tracked_fill(&mut array, 5, 0);
    array.reallocate(16, true);
    assert_eq!(16, array.len());
    tracked_check(&array, 8, 5, 0);
}

#[test]
fn NCollection_LocalArrayTest__NonTrivial_Shrink_DestroysExcess() {
    reset_tracked();
    {
        let mut array = LocalArray::<LocalTracked, 8>::new(8);
        assert_eq!(8, TRACKED_CTORS.get());
        array.reallocate(3, true);
        assert_eq!(3, array.len());
        assert_eq!(5, TRACKED_DTORS.get());
    }
    assert_eq!(8, TRACKED_DTORS.get());
}

#[test]
fn NCollection_LocalArrayTest__NonTrivial_MoveConstructor_FromInline() {
    let mut source = LocalArray::<LocalTracked, 8>::new(4);
    tracked_fill(&mut source, 7, 0);
    let dest = std::mem::take(&mut source);
    assert_eq!(4, dest.len());
    tracked_check(&dest, 4, 7, 0);
    assert_eq!(0, source.len());
}

#[test]
fn NCollection_LocalArrayTest__NonTrivial_MoveConstructor_FromHeap() {
    let mut source = LocalArray::<LocalTracked, 4>::new(8);
    tracked_fill(&mut source, 3, 0);
    let dest = std::mem::take(&mut source);
    assert_eq!(8, dest.len());
    tracked_check(&dest, 8, 3, 0);
    assert_eq!(0, source.len());
}

#[test]
fn NCollection_LocalArrayTest__NonTrivial_MoveAssignment() {
    let mut source = LocalArray::<LocalTracked, 8>::new(4);
    tracked_fill(&mut source, 1, 100);
    let mut dest = LocalArray::<LocalTracked, 8>::new(2);
    dest.move_from(&mut source);
    assert_eq!(4, dest.len());
    tracked_check(&dest, 4, 1, 100);
    assert_eq!(0, source.len());
}

#[test]
fn NCollection_LocalArrayTest__NonTrivial_DestructorCallBalance() {
    reset_tracked();
    {
        let mut array = LocalArray::<LocalTracked, 4>::new(4);
        array.reallocate(8, true);
        array.reallocate(2, true);
        array.reallocate(16, true);
    }
    assert_eq!(TRACKED_CTORS.get(), TRACKED_DTORS.get());
}
