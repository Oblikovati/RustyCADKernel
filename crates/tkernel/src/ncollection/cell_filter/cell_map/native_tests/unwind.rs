use super::*;
use std::panic::{AssertUnwindSafe, catch_unwind};
#[test]
fn failed_target_copy_keeps_the_previous_list() {
    let (mut filter, probe) = filled(0);
    probe.borrow_mut().fail_clone = Some(4);
    assert!(
        catch_unwind(AssertUnwindSafe(
            || filter.add(&Target::source(4, &probe), &[0.])
        ))
        .is_err()
    );
    probe.borrow_mut().recording = false;
    let mut row = String::new();
    remaining(&mut row, &filter);
    assert_eq!(row, " R3,2,1, cells1");
}
#[test]
fn destructor_panic_leaves_the_destroyed_target_unlinked() {
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
    probe.borrow_mut().recording = false;
}
