use super::*;
use std::cell::RefCell;

#[derive(Debug)]
struct PanicOwner<'a>(&'a RefCell<Option<Box<u64>>>);
impl Clone for PanicOwner<'_> {
    fn clone(&self) -> Self {
        std::panic::panic_any(self.0.borrow_mut().take().unwrap());
    }
}

#[test]
fn test_observation_resumes_the_identical_owned_panic_payload() {
    let token = Box::new(42_u64);
    let identity = std::ptr::from_ref(&*token);
    let state = RefCell::new(Some(token));
    let mut source = LinearVector::with_capacity(1);
    source.append(PanicOwner(&state));
    let abandoned = Cell::new(None);
    let failure = catch_unwind(AssertUnwindSafe(|| {
        source.construct_copy(|vector| {
            abandoned.set(Some((vector.data_mut_ptr(), vector.capacity())));
        })
    }))
    .unwrap_err();
    let token = failure.downcast::<Box<u64>>().unwrap();
    assert_eq!(std::ptr::from_ref(&**token), identity);
    let (pointer, capacity) = abandoned.get().unwrap();
    // SAFETY: the first constructor panicked; this allocation has no initialized values.
    unsafe { drop(Vec::from_raw_parts(pointer, 0, capacity)) };
}

#[derive(Debug, Copy)]
struct FailingZero;
#[allow(clippy::non_canonical_clone_impl)]
impl Clone for FailingZero {
    fn clone(&self) -> Self {
        panic!("zero-sized copy: expected failure probe");
    }
}

#[test]
fn zero_sized_failed_copy_has_no_allocation_and_keeps_valid_source() {
    let mut source = LinearVector::with_capacity(1);
    source.append(FailingZero);
    let abandoned = Cell::new(None);
    let failure = catch_unwind(AssertUnwindSafe(|| {
        source.construct_copy(|vector| {
            assert_eq!(vector.len(), 0);
            abandoned.set(Some((vector.data_mut_ptr(), vector.capacity())));
        })
    }));
    assert!(failure.is_err());
    assert_eq!(source.len(), 1);
    let (pointer, capacity) = abandoned.get().unwrap();
    assert_eq!(capacity, usize::MAX);
    // SAFETY: ZST Vec storage is a valid dangling aligned pointer with no live prefix.
    unsafe { drop(Vec::from_raw_parts(pointer, 0, capacity)) };
}
