//! Context replacement cannot retarget a previously returned MainSelector slot.
use super::*;
use crate::{select_mgr::*, v3d::V3dViewer};
use rustycad_tkernel::standard::{Handle, HandleReference};
use rustycad_tkservice::null_driver::NullGraphicDriver;
fn selector_reference_context() -> Handle<AisInteractiveContext> {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    AisInteractiveContext::new(&viewer).unwrap()
}
fn selector_reference_row(
    phase: i32,
    context: &AisInteractiveContext,
    reference: &AisMainSelectorRef,
    first: &Handle<SelectMgrSelectionManager>,
    second: &Handle<SelectMgrSelectionManager>,
) -> [i32; 8] {
    [
        phase,
        i32::from(reference.reference_address() == std::ptr::from_ref(first.selector()).cast()),
        i32::from(reference.object_address() == first.selector().object_address()),
        first.ref_count(),
        second.ref_count(),
        reference.observe().ref_count(),
        second.selector().ref_count(),
        i32::from(context.fields.read().selection_manager == *first),
    ]
}
fn selector_reference_native_rows() -> Vec<Vec<i32>> {
    include_str!("../../tests/fixtures/context-main-selector-reference-native.txt")
        .lines()
        .map(|line| {
            line.split_whitespace()
                .map(|value| value.parse().unwrap())
                .collect()
        })
        .collect()
}
#[test]
fn original_manager_slot_reference_matches_native_replacement_null_and_restore_counts() {
    let context = selector_reference_context();
    let first = context.selection_manager().clone();
    let selector = Handle::new(SelectMgrViewerSelector::new());
    let second = Handle::new(SelectMgrSelectionManager::new(&selector.cast()));
    let reference = context.main_selector();
    let expected = selector_reference_native_rows();
    assert_eq!(
        selector_reference_row(0, &context, &reference, &first, &second).as_slice(),
        expected[0]
    );
    for phase in 1..=3 {
        replace_selector_manager(&context, &first, &second, phase);
        assert_eq!(
            selector_reference_row(phase, &context, &reference, &first, &second).as_slice(),
            expected[phase as usize]
        );
    }
}
#[test]
fn selector_observation_releases_context_lock_before_reentrant_state_changes() {
    let context = selector_reference_context();
    let first = context.selection_manager().clone();
    let reference = context.main_selector();
    let observed = reference.observe();
    assert_eq!(first.ref_count(), 2);
    let mut fields = context
        .fields
        .try_write()
        .expect("MainSelector must release context read lease");
    fields.selection_manager = Handle::default();
    drop(fields);
    assert_eq!(reference.object_address(), observed.object_address());
    assert_eq!(first.ref_count(), 1);
    assert_eq!(reference.clone_handle().ref_count(), 2);
}

fn replace_selector_manager(
    context: &AisInteractiveContext,
    first: &Handle<SelectMgrSelectionManager>,
    second: &Handle<SelectMgrSelectionManager>,
    phase: i32,
) {
    context.fields.write().selection_manager = match phase {
        1 => second.clone(),
        2 => Handle::default(),
        _ => first.clone(),
    };
}
