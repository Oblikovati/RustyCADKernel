use super::{
    DROP_TRACE,
    fixture::{Trace, Tree, caught, populated},
};
use rustycad_tkernel::ncollection::BaseAllocator;
fn failing_destination_drop(kind: char) {
    let target_trace = Trace::default();
    let source_trace = Trace::default();
    let target_allocator = BaseAllocator::incremental(1024);
    let mut target = populated(&target_trace, target_allocator.clone());
    let source_allocator = BaseAllocator::incremental(1024);
    let mut source = populated(&source_trace, source_allocator.clone());
    let source_root = std::ptr::from_ref(source.root().unwrap());
    let source_last = std::ptr::from_ref(source.last_node().unwrap());
    target_trace.reject('D', kind);
    assert!(caught(|| {
        target.move_assign(&mut source);
    }));
    assert_eq!(target_trace.events(), DROP_TRACE);
    assert_failed_destination(&target, &target_allocator);
    assert_eq!(std::ptr::from_ref(source.root().unwrap()), source_root);
    assert_eq!(std::ptr::from_ref(source.last_node().unwrap()), source_last);
    assert_eq!(source_trace.events(), "");
    assert_eq!(source.allocator(), &source_allocator);
}
fn assert_failed_destination(target: &Tree, allocator: &BaseAllocator) {
    assert!(target.is_empty());
    assert_eq!(target.allocator(), allocator);
    allocator.reset(true).unwrap();
}
#[test]
fn object_drop_panic_drains_destination_before_any_source_transfer() {
    failing_destination_drop('O');
}
#[test]
fn bounds_drop_panic_drains_destination_before_any_source_transfer() {
    failing_destination_drop('B');
}
#[test]
fn successful_transfer_releases_old_leases_and_retains_transferred_ones() {
    let trace = Trace::default();
    let original = BaseAllocator::incremental(1024);
    let donor = BaseAllocator::incremental(1024);
    let mut target = populated(&trace, original.clone());
    let mut source = populated(&trace, donor.clone());
    target.move_assign(&mut source);
    original.reset(true).unwrap();
    assert!(donor.reset(true).is_err());
    assert_eq!(source.allocator(), &original);
    source.add(trace.item(9), trace.item(9));
    assert!(original.reset(true).is_err());
    assert_eq!(source.root().unwrap().object().value, 9);
    source.clear();
    target.clear();
    donor.reset(true).unwrap();
}
