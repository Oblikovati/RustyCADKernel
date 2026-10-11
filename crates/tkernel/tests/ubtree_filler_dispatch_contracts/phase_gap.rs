use super::{allocation_trace::*, observed_key::Key};
use rustycad_tkernel::ncollection::{BaseAllocator, UBTree, UBTreeFiller};
#[test]
fn native_dispatch_does_not_claim_observable_self_assignment_parity() {
    let allocator = BaseAllocator::accumulating(65536).unwrap();
    let mut tree = UBTree::<Key, Payload<'B'>>::with_allocator(allocator.clone());
    let mut target = tree.native_insertion();
    let mut filler = UBTreeFiller::new(&mut target, None, true);
    filler.add_ref(&Key(Payload(1)), &Payload(10));
    start(&allocator, ('X', 'X', 0));
    assert_eq!(filler.fill(), 1);
    let rust = finish();
    let native =
        include_str!("../../../../tools/oracle/tkernel_ubtree_filler_phase_observations.expected")
            .lines()
            .next()
            .unwrap()
            .split_whitespace()
            .nth(6)
            .unwrap();
    assert_eq!(rust, "CB10:1,CO1:1,DB10:1,DO1:1,");
    assert_eq!(native, "CB10:1,CO1:1,AO1:1,AB10:1,DB10:1,DO1:1,");
}
