use super::*;
use std::panic::{AssertUnwindSafe, catch_unwind};
#[test]
fn failed_target_copy_keeps_previous_chain_and_releases_unpublished_lease() {
    let (mut filter, probe) = filled(0);
    probe.borrow_mut().fail_clone = Some(4);
    assert!(
        catch_unwind(AssertUnwindSafe(
            || filter.add(&Target::source(4, &probe), &[0.])
        ))
        .is_err()
    );
    probe.borrow_mut().cell = None;
    let mut row = String::new();
    remaining(&mut row, &filter);
    assert_eq!(row, " R3,2,1, cells1");
    assert_eq!(probe.borrow().live_leases(), 4);
}
#[test]
fn destructor_panic_unlinks_destroyed_object_before_resuming() {
    let (mut filter, probe) = filled(0);
    probe.borrow_mut().fail_drop = Some(3);
    assert!(
        catch_unwind(AssertUnwindSafe(
            || filter.remove(&Target::source(3, &probe), &[0.])
        ))
        .is_err()
    );
    let mut row = String::new();
    remaining(&mut row, &filter);
    assert_eq!(row, " R2,1, cells1");
    probe.borrow_mut().cell = None;
}
#[test]
fn reset_drop_panic_drains_all_targets_and_releases_arena_leases() {
    let (mut filter, probe) = filled(0);
    probe.borrow_mut().fail_drop = Some(3);
    assert!(catch_unwind(AssertUnwindSafe(|| filter.reset(2., None))).is_err());
    assert!(filter.cells.is_empty());
    probe.borrow_mut().cell = None;
    assert!(probe.borrow().arena.reset(true).is_ok());
}
