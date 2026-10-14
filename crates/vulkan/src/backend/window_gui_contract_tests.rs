//! Owned Vulkan result values exercise the same classification seam as real presentation.
use super::window_gui_contract::combine_native_observation;
use super::window_gui_contract::*;
use crate::*;
use ash::vk;
fn checked_contract() -> WindowGuiContract {
    let mut frame = RenderFrame::new(64, 64, [0.; 4], Vec::new()).unwrap();
    frame.gui_draw_start = Some(0);
    WindowGuiContract::new(&frame, RenderGuiSurfacePolicy::CheckedV2)
}
#[test]
fn out_of_date_changes_gui_outcome_without_classifying_real_faults_as_resize() {
    let contract = checked_contract();
    let failure = contract
        .present_result(Err(vk::Result::ERROR_OUT_OF_DATE_KHR))
        .unwrap_err();
    assert!(matches!(
        failure.gui_surface_outcome(),
        Some(RenderGuiSurfaceOutcome::SurfaceChanged(_))
    ));
    verify_actual_present_faults(contract);
}
fn verify_actual_present_faults(contract: WindowGuiContract) {
    for fault in [
        vk::Result::ERROR_DEVICE_LOST,
        vk::Result::ERROR_SURFACE_LOST_KHR,
        vk::Result::ERROR_OUT_OF_DEVICE_MEMORY,
        vk::Result::TIMEOUT,
    ] {
        let actual = contract.present_result(Err(fault)).unwrap_err();
        assert!(actual.gui_surface_outcome().is_none());
        assert_eq!(
            actual,
            super::runtime::vk_error("present window frame", fault)
        );
    }
}
#[test]
fn legacy_gui_and_checked_model_keep_their_present_classification() {
    let mut frame = RenderFrame::new(64, 64, [0.; 4], Vec::new()).unwrap();
    let model = WindowGuiContract::new(&frame, RenderGuiSurfacePolicy::CheckedV2);
    frame.gui_draw_start = Some(0);
    let legacy = WindowGuiContract::new(&frame, RenderGuiSurfacePolicy::LegacyV1);
    for contract in [model, legacy] {
        verify_legacy_present_classification(contract);
    }
}
fn verify_legacy_present_classification(contract: WindowGuiContract) {
    assert!(
        contract
            .out_of_date(RenderGuiSurfacePhase::Acquire)
            .is_none()
    );
    assert!(
        contract
            .present_result(Err(vk::Result::ERROR_OUT_OF_DATE_KHR))
            .is_ok()
    );
    assert!(
        contract
            .validate([80, 72], RenderGuiSurfacePhase::Acquire)
            .is_ok()
    );
}

#[test]
fn checked_suboptimal_does_not_invent_a_resize() {
    let contract = checked_contract();
    assert!(contract.present_result(Ok(true)).is_ok());
    assert!(contract.present_result(Ok(false)).is_ok());
    assert!(
        contract
            .validate([64, 64], RenderGuiSurfacePhase::Acquire)
            .is_ok()
    );
}
#[test]
fn native_change_observation_preserves_the_original_fault() {
    let contract = checked_contract();
    let fault = super::runtime::vk_error("submit frame", vk::Result::ERROR_DEVICE_LOST);
    let observation = contract.validate([80, 72], RenderGuiSurfacePhase::BeforeSubmit);
    let failure = combine_native_observation::<()>(Err(fault.clone()), observation).unwrap_err();
    let Some(RenderGuiSurfaceOutcome::FaultWithSurfaceChange {
        fault: actual,
        change,
    }) = failure.gui_surface_outcome()
    else {
        panic!("expected compound native outcome")
    };
    assert_eq!(actual, &fault);
    assert_eq!(change.phase, RenderGuiSurfacePhase::BeforeSubmit);
}

#[test]
fn later_observation_fault_is_not_hidden_by_earlier_out_of_date_refusal() {
    let surface = checked_contract()
        .out_of_date(RenderGuiSurfacePhase::Acquire)
        .unwrap();
    let fault = super::runtime::vk_error("observe extent", vk::Result::ERROR_DEVICE_LOST);
    let failure = combine_native_observation::<()>(Err(surface), Err(fault.clone())).unwrap_err();
    let Some(RenderGuiSurfaceOutcome::FaultWithSurfaceChange { fault: actual, .. }) =
        failure.gui_surface_outcome()
    else {
        panic!("expected retained actual observation fault")
    };
    assert_eq!(actual, &fault);
}

#[test]
fn unknown_extent_observation_overrides_surface_only_recovery() {
    let surface = checked_contract()
        .out_of_date(RenderGuiSurfacePhase::Acquire)
        .unwrap();
    let extent = vk::Extent2D {
        width: u32::MAX,
        height: u32::MAX,
    };
    let unsupported = super::window_surface::authoritative_gui_extent(extent).unwrap_err();
    let failure =
        combine_native_observation::<()>(Err(surface), Err(unsupported.clone())).unwrap_err();
    let Some(RenderGuiSurfaceOutcome::FaultWithSurfaceChange { fault, .. }) =
        failure.gui_surface_outcome()
    else {
        panic!("expected actual observation failure")
    };
    assert_eq!(fault, &unsupported);
}
