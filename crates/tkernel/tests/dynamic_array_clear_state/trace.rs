use super::payload::*;
use rustycad_tkernel::ncollection::DynamicArrayClearState;
use std::panic::{AssertUnwindSafe, catch_unwind};
pub(crate) fn clear_rows(release: bool, fail: i32) -> Vec<String> {
    let (mut state, probe) = fixture();
    probe.borrow_mut().fail = fail;
    let caught = catch_unwind(AssertUnwindSafe(|| clear(&mut state, &probe, release)));
    let mut rows = vec![state_row(&state, &probe, release, fail, caught.is_err())];
    if let Err(payload) = caught {
        assert_eq!(
            payload.downcast_ref::<ClearPanic>(),
            Some(&ClearPanic(fail))
        );
        rows.extend(repair_rows(&mut state, &probe, fail));
    }
    let array = state
        .try_into_array()
        .unwrap_or_else(|_| panic!("Clear completion: expected no holes"));
    assert_eq!(array.size(), 0);
    rows
}
fn state_row(
    state: &DynamicArrayClearState<ClearItem>,
    probe: &SharedProbe,
    release: bool,
    fail: i32,
    caught: bool,
) -> String {
    let observer = probe.borrow();
    format!(
        "clear release={} fail={fail} caught={} size={} blocks={} ended={} events={}",
        u8::from(release),
        u8::from(caught),
        state.size(),
        live_blocks(&observer.allocator),
        observer.ended.len(),
        observer.events
    )
}
fn repair_rows(
    state: &mut DynamicArrayClearState<ClearItem>,
    probe: &SharedProbe,
    fail: i32,
) -> Vec<String> {
    assert_eq!(state.dead_slots(), 0..fail as usize);
    assert!(state.live_value(fail as usize - 1).is_none());
    let tail = state.live_value(fail as usize).map_or(0, |slot| slot.value);
    let mut rows = vec![format!("tail fail={fail} value={tail}")];
    repair(state, probe);
    clear(state, probe, true);
    let observer = probe.borrow();
    rows.push(format!(
        "repair size={} blocks={} events={}",
        state.size(),
        live_blocks(&observer.allocator),
        observer.events
    ));
    rows
}
