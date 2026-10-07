use super::fixture::{Trace, populated};
use rustycad_tkernel::ncollection::BaseAllocator;
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
