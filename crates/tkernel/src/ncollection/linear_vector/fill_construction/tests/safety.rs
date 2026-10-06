use super::*;
use std::cell::RefCell;

#[derive(Debug)]
struct FillPanicOwner<'a>(&'a RefCell<Option<Box<u64>>>);
impl Clone for FillPanicOwner<'_> {
    fn clone(&self) -> Self {
        std::panic::panic_any(self.0.borrow_mut().take().unwrap());
    }
}

#[test]
fn fill_observation_preserves_owned_panic_payload_identity() {
    let token = Box::new(77_u64);
    let identity = std::ptr::from_ref(&*token);
    let state = RefCell::new(Some(token));
    let donor = FillPanicOwner(&state);
    let abandoned = Cell::new(None);
    let failure = catch_unwind(AssertUnwindSafe(|| {
        LinearVector::construct_fill(1, &donor, |vector| {
            abandoned.set(Some((vector.data_mut_ptr(), vector.capacity())));
        })
    }))
    .unwrap_err();
    let payload = failure.downcast::<Box<u64>>().unwrap();
    assert_eq!(std::ptr::from_ref(&**payload), identity);
    let (pointer, capacity) = abandoned.get().unwrap();
    assert_eq!(capacity, 2);
    // SAFETY: the first constructor failed, leaving zero live values in this allocation.
    unsafe { drop(Vec::from_raw_parts(pointer, 0, capacity)) };
}

#[derive(Debug)]
struct FailingZero;
impl Clone for FailingZero {
    fn clone(&self) -> Self {
        panic!("zero-sized fill: expected failed clone probe");
    }
}

#[test]
fn zero_sized_fill_failure_keeps_valid_empty_storage_and_donor() {
    let donor = FailingZero;
    let abandoned = Cell::new(None);
    let failure = catch_unwind(AssertUnwindSafe(|| {
        LinearVector::construct_fill(3, &donor, |vector| {
            assert_eq!(vector.len(), 0);
            abandoned.set(Some((vector.data_mut_ptr(), vector.capacity())));
        })
    }));
    assert!(failure.is_err());
    let (pointer, capacity) = abandoned.get().unwrap();
    assert_eq!(capacity, usize::MAX);
    // SAFETY: aligned dangling ZST storage has zero live values and needs no allocation free.
    unsafe { drop(Vec::from_raw_parts(pointer, 0, capacity)) };
}
