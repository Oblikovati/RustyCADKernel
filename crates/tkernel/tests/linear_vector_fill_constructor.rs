//! Successful public fill construction keeps native capacity and Rust ownership boundaries.
use rustycad_tkernel::ncollection::LinearVector;
use std::cell::Cell;

#[test]
fn borrowed_fill_constructor_uses_native_initial_growth_capacity() {
    let value = String::from("fill");
    for (count, capacity) in [(0, 0), (1, 2), (2, 2), (3, 3), (5, 5)] {
        let vector = LinearVector::with_value_ref(count, &value);
        assert_eq!((vector.len(), vector.capacity()), (count, capacity));
        assert!(vector.iter().all(|item| item == &value));
    }
}

struct BorrowedFill<'a>(&'a Cell<usize>);
impl Clone for BorrowedFill<'_> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}
impl Drop for BorrowedFill<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn owned_wrapper_releases_its_donor_after_successful_copying() {
    let drops = Cell::new(0);
    let vector = LinearVector::with_value(3, BorrowedFill(&drops));
    assert_eq!(drops.get(), 1);
    assert_eq!(vector.len(), 3);
    drop(vector);
    assert_eq!(drops.get(), 4);
}

#[test]
fn fill_keeps_borrowed_lifetimes_and_public_traits() {
    fn traits<T: Send + Sync + Unpin + std::panic::UnwindSafe + std::panic::RefUnwindSafe>() {}
    fn shorten<'a, 'b: 'a>(vector: LinearVector<&'b str>) -> LinearVector<&'a str> {
        vector
    }
    traits::<LinearVector<String>>();
    let owned = String::from("borrowed");
    let vector = shorten(LinearVector::with_value_ref(2, &owned.as_str()));
    assert_eq!(vector.as_slice(), [owned.as_str(), owned.as_str()]);
}

#[test]
fn zero_sized_fill_construction_preserves_valid_rust_lengths() {
    for count in [0, 1, 3] {
        let vector = LinearVector::with_value_ref(count, &());
        assert_eq!(vector.len(), count);
        assert_eq!(vector.capacity(), usize::MAX);
    }
}
