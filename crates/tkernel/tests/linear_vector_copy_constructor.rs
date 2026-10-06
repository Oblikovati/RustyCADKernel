//! Public successful construction and retained-assignment guards complement failure-oracle probes.
use rustycad_tkernel::ncollection::LinearVector;
use std::cell::Cell;

#[test]
fn empty_and_reserved_empty_sources_copy_to_unallocated_results() {
    for source in [
        LinearVector::<String>::default(),
        LinearVector::with_capacity(8),
    ] {
        let copied = source.clone();
        assert_eq!((copied.len(), copied.capacity()), (0, 0));
        assert!(copied.data_ptr().is_null());
    }
}

#[test]
fn successful_copy_allocates_source_length_and_independent_owned_payloads() {
    let mut source = LinearVector::with_capacity(8);
    source.append(Box::new(11));
    source.append(Box::new(22));
    let copied = source.clone();
    assert_eq!((copied.len(), copied.capacity()), (2, 2));
    assert_eq!(copied.as_slice(), source.as_slice());
    assert_ne!(copied.data_ptr(), source.data_ptr());
    assert!(!std::ptr::eq(&*copied[0], &*source[0]));
}

#[test]
fn borrowed_lifetimes_covariance_and_public_traits_remain_available() {
    fn traits<T: Send + Sync + Unpin + std::panic::UnwindSafe + std::panic::RefUnwindSafe>() {}
    fn shorten<'a, 'b: 'a>(vector: LinearVector<&'b str>) -> LinearVector<&'a str> {
        vector
    }
    traits::<LinearVector<String>>();
    let owned = String::from("borrowed");
    let mut source = LinearVector::with_capacity(2);
    source.append(owned.as_str());
    let copied = shorten(source.clone());
    assert_eq!(copied.as_slice(), [owned.as_str()]);
}

#[test]
fn zero_sized_empty_and_nonempty_copy_keep_valid_rust_lengths() {
    let empty = LinearVector::<()>::default().clone();
    assert!(empty.is_empty());
    let source = LinearVector::with_value(3, ());
    let copied = source.clone();
    assert_eq!(copied.len(), 3);
    assert_eq!(copied.capacity(), usize::MAX);
}

struct AssignmentOnly<'a> {
    value: i32,
    assignments: &'a Cell<usize>,
}
impl Clone for AssignmentOnly<'_> {
    fn clone(&self) -> Self {
        panic!("retained equal-size assignment: expected no copy constructor");
    }
    fn clone_from(&mut self, source: &Self) {
        self.value = source.value;
        self.assignments.set(self.assignments.get() + 1);
    }
}

#[test]
fn sufficient_capacity_assignment_keeps_its_existing_copy_assignment_path() {
    let assignments = Cell::new(0);
    let mut source = LinearVector::with_capacity(1);
    let mut target = LinearVector::with_capacity(4);
    source.append(AssignmentOnly {
        value: 9,
        assignments: &assignments,
    });
    target.append(AssignmentOnly {
        value: 1,
        assignments: &assignments,
    });
    let before = target.data_ptr();
    target.assign(&source);
    assert_eq!(target[0].value, 9);
    assert_eq!(target.data_ptr(), before);
    assert_eq!(assignments.get(), 1);
}
