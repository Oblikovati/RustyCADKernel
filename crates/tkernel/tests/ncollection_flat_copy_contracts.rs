//! Copy construction and assignment follow OCCT policy and element lifetime order.
#[path = "flat_copy_contract/fixtures.rs"]
mod fixtures;
use fixtures::{CopyTrace, copy_dictionary, copy_set};

#[test]
fn flat_set_copies_policy_before_keys_without_hashing() {
    let trace: CopyTrace = CopyTrace::default();
    let source = copy_set(&trace, 4, 7);
    trace.borrow_mut().clear();
    let copied = source.clone();
    assert_eq!(&*trace.borrow(), "CK");
    assert_eq!(copied.capacity(), source.capacity());
    assert_eq!(copied.hasher().salt, 7);
}

#[test]
fn flat_dictionary_copies_policy_before_key_and_item() {
    let trace: CopyTrace = CopyTrace::default();
    let source = copy_dictionary(&trace, 4, 7);
    trace.borrow_mut().clear();
    let copied = source.clone();
    assert_eq!(&*trace.borrow(), "CKV");
    assert_eq!(copied.capacity(), source.capacity());
    assert_eq!(copied.hasher().salt, 7);
}

#[test]
fn flat_set_assignment_clears_then_assigns_policy_then_copies_keys() {
    let trace: CopyTrace = CopyTrace::default();
    let source = copy_set(&trace, 4, 7);
    let mut destination = copy_set(&trace, 9, 2);
    trace.borrow_mut().clear();
    destination.clone_from(&source);
    assert_eq!(&*trace.borrow(), "kAK");
    assert_eq!(destination.iter().next().unwrap().number, 4);
    assert_eq!(destination.hasher().salt, 7);
}

#[test]
fn flat_dictionary_assignment_clears_then_assigns_policy_then_copies_pairs() {
    let trace: CopyTrace = CopyTrace::default();
    let source = copy_dictionary(&trace, 4, 7);
    let mut destination = copy_dictionary(&trace, 9, 2);
    trace.borrow_mut().clear();
    destination.assign(&source);
    assert_eq!(&*trace.borrow(), "kvAKV");
    assert_eq!(destination.iter().next().unwrap().number, 40);
    assert_eq!(destination.hasher().salt, 7);
}

macro_rules! returning_copy_and_self {
    ($test:ident, $build:ident, $trace:literal) => {
        #[test]
        fn $test() {
            let trace: CopyTrace = CopyTrace::default();
            let source = $build(&trace, 4, 7);
            let mut destination = $build(&trace, 9, 2);
            let original = std::ptr::from_ref(&destination);
            trace.borrow_mut().clear();
            assert!(std::ptr::eq(
                destination.assign_returning(&source),
                original
            ));
            assert_eq!(&*trace.borrow(), $trace);
            trace.borrow_mut().clear();
            let capacity: usize = destination.capacity();
            assert!(std::ptr::eq(destination.assign_self(), original));
            assert_eq!(destination.capacity(), capacity);
            assert!(trace.borrow().is_empty());
        }
    };
}
returning_copy_and_self!(
    flat_set_returns_destination_and_preserves_self,
    copy_set,
    "kAK"
);
returning_copy_and_self!(
    flat_dictionary_returns_destination_and_preserves_self,
    copy_dictionary,
    "kvAKV"
);

macro_rules! empty_reserved_copy {
    ($test:ident, $build:ident, $trace:literal) => {
        #[test]
        fn $test() {
            let trace: CopyTrace = CopyTrace::default();
            let mut source = $build(&trace, 4, 7);
            let mut destination = $build(&trace, 9, 2);
            source.clear(false);
            assert!(source.capacity() > 0);
            trace.borrow_mut().clear();
            destination.clone_from(&source);
            assert_eq!(&*trace.borrow(), $trace);
            assert_eq!((destination.len(), destination.capacity()), (0, 0));
            assert!(source.capacity() > 0);
            assert_eq!(destination.hasher().salt, 7);
            trace.borrow_mut().clear();
            let empty_copy = source.clone();
            assert_eq!(&*trace.borrow(), "C");
            assert_eq!((empty_copy.len(), empty_copy.capacity()), (0, 0));
        }
    };
}
empty_reserved_copy!(flat_set_empty_copy_releases_reserved_slots, copy_set, "kA");
empty_reserved_copy!(
    flat_dictionary_empty_copy_releases_reserved_slots,
    copy_dictionary,
    "kvA"
);
