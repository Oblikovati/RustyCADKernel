//! Typed failure inspection preserves primary causes and stable observation evidence.
use crate::*;
use rustycad_tkservice::aspect::*;
fn owned_change() -> RenderGuiSurfaceChange {
    let phase = RenderGuiSurfacePhase::Present;
    RenderGuiSurfaceChange::out_of_date(phase, [64, 64])
}
fn require_send_sync<T: Send + Sync>() {
    let instance: Option<T> = None;
    assert!(instance.is_none());
}
#[test]
fn gui_surface_public_values_and_render_error_are_send_sync() {
    require_send_sync::<RenderError>();
    require_send_sync::<RenderGuiSurfacePolicy>();
    require_send_sync::<RenderGuiSurfacePhase>();
    require_send_sync::<RenderGuiFreshPacket>();
    require_send_sync::<RenderGuiSurfaceCause>();
    require_send_sync::<RenderGuiSurfaceChange>();
    require_send_sync::<RenderGuiSurfaceOutcome<'static>>();
}
#[test]
fn surface_only_failure_is_distinct_from_an_ordinary_device_fault() {
    let surface = RenderError::surface_changed(owned_change());
    assert!(matches!(
        surface.gui_surface_outcome(),
        Some(RenderGuiSurfaceOutcome::SurfaceChanged(_))
    ));
    assert!(std::error::Error::source(&surface).is_none());
    assert!(
        RenderError::rejected("device lost", "successful submit")
            .gui_surface_outcome()
            .is_none()
    );
    assert_eq!(owned_change().behavior_version(), 2);
}
#[test]
fn compound_window_fault_preserves_typed_getter_and_source_chain() {
    let closure = AspectNativeWindowClosure {
        original_handle: 91,
        cause: AspectNativeWindowClosureCause::DestroyNotify,
        behavior_version: 1,
    };
    let cause = closure.failure(AspectNativeWindowOperation::Redraw);
    let primary = RenderError::from(cause);
    let combined = primary.clone().with_surface_change(owned_change());
    assert_eq!(combined.native_window_failure(), Some(&cause));
    assert_eq!(combined.to_string(), primary.to_string());
    let source = std::error::Error::source(&combined).unwrap();
    assert_eq!(source.downcast_ref::<RenderError>(), Some(&primary));
    assert!(source.source().unwrap().is::<AspectNativeWindowFailure>());
}
#[test]
fn compound_display_fault_preserves_admission_payload() {
    let cause = AspectDisplayConnection::acquire_lease(
        &Default::default(),
        AspectDisplayLeasePurpose::VulkanInstance,
    )
    .err()
    .unwrap();
    let primary = RenderError::from(cause.clone());
    let combined = primary.clone().with_surface_change(owned_change());
    assert_eq!(combined.native_display_failure(), Some(&cause));
    let source = std::error::Error::source(&combined).unwrap();
    assert_eq!(source.downcast_ref::<RenderError>(), Some(&primary));
    assert!(
        source
            .source()
            .unwrap()
            .is::<AspectDisplayLifecycleFailure>()
    );
}
#[test]
fn repeated_observation_keeps_first_phase_and_primary_without_nested_faults() {
    let primary = RenderError::rejected(7, "successful submit");
    let first = primary.clone().with_surface_change(owned_change());
    let mut newer =
        RenderGuiSurfaceChange::extent_mismatch(RenderGuiSurfacePhase::Capture, [64, 64], [80, 72]);
    newer.fresh_packet = RenderGuiFreshPacket::AlreadyCurrent;
    newer.target_revision_changed = true;
    let combined = first.with_surface_change(newer);
    let Some(RenderGuiSurfaceOutcome::FaultWithSurfaceChange { fault, change }) =
        combined.gui_surface_outcome()
    else {
        panic!("expected compound outcome")
    };
    assert_eq!(fault, &primary);
    assert_eq!(change.phase, RenderGuiSurfacePhase::Present);
    assert_eq!(change.cause, RenderGuiSurfaceCause::OutOfDate);
    assert_eq!(change.observed_extent, Some([80, 72]));
    assert!(change.target_revision_changed);
    assert_eq!(change.fresh_packet, RenderGuiFreshPacket::AlreadyCurrent);
}

fn source_chain_length(error: &(dyn std::error::Error + 'static)) -> usize {
    let mut current = error.source();
    let mut count = 0;
    while let Some(source) = current {
        count += 1;
        assert!(count < 8, "expected bounded acyclic error causes");
        current = source.source();
    }
    count
}
#[test]
fn repeated_resize_fault_closure_observations_keep_bounded_typed_cause_chain() {
    let display = display_admission_failure();
    let closure = capture_closure();
    let mut error = RenderError::from(display.clone())
        .with_surface_change(owned_change())
        .with_window_target(closure);
    let initial_length = source_chain_length(&error);
    for _ in 0..64 {
        error = error
            .with_window_target(closure)
            .with_surface_change(owned_change());
        assert_eq!(source_chain_length(&error), initial_length);
        assert_eq!(error.native_window_failure(), Some(&closure));
        assert_eq!(error.native_display_failure(), Some(&display));
        assert!(matches!(
            error.gui_surface_outcome(),
            Some(RenderGuiSurfaceOutcome::FaultWithSurfaceChange { .. })
        ));
    }
}

#[test]
fn later_actual_fault_supersedes_surface_only_recovery_classification() {
    let surface = RenderError::surface_changed(owned_change());
    let device = RenderError::rejected("ERROR_DEVICE_LOST", "successful observation");
    let failure = surface.with_observed_failure(device.clone());
    let Some(RenderGuiSurfaceOutcome::FaultWithSurfaceChange { fault, change }) =
        failure.gui_surface_outcome()
    else {
        panic!("expected actual observation fault")
    };
    assert_eq!(fault, &device);
    assert_eq!(change.cause, RenderGuiSurfaceCause::OutOfDate);
}
#[test]
fn varied_observed_faults_replace_companion_without_growing_primary_chain() {
    let primary = RenderError::rejected("submit failure", "successful submission");
    let mut error = primary.clone();
    for index in 0..64 {
        let observed = RenderError::rejected(index, "authoritative positive extent");
        error = error.with_observed_failure(observed.clone());
        assert_eq!(error.primary_failure(), Some(&primary));
        assert_eq!(error.observed_failure(), Some(&observed));
        assert_eq!(source_chain_length(&error), 1);
    }
}

fn latest_observed_in_chain(error: &RenderError) -> Option<&RenderError> {
    let immediate = error.observed_failure();
    immediate.or_else(|| error.primary_failure().and_then(latest_observed_in_chain))
}
#[test]
fn varied_observations_keep_nested_surface_closure_and_original_display_bounded() {
    let display = display_admission_failure();
    let closure = capture_closure();
    let error = RenderError::from(display.clone()).with_surface_change(owned_change());
    verify_varied_nested_observations(error.with_window_target(closure), &display, closure);
}
fn verify_varied_nested_observations(
    mut error: RenderError,
    display: &AspectDisplayLifecycleFailure,
    closure: AspectNativeWindowFailure,
) {
    for index in 0..64 {
        let observed = RenderError::rejected(index, "authoritative positive extent");
        error = error
            .with_observed_failure(observed.clone())
            .with_window_target(closure);
        assert_eq!(latest_observed_in_chain(&error), Some(&observed));
        assert_eq!(source_chain_length(&error), 4);
        assert_eq!(error.native_window_failure(), Some(&closure));
        assert_eq!(error.native_display_failure(), Some(display));
    }
}
#[test]
fn native_target_display_fault_promotes_surface_only_to_actual_fault() {
    let display = display_admission_failure();
    let result: Result<(), RenderError> = crate::error::gui_native_failure::reconcile_native_target(
        Err(RenderError::surface_changed(owned_change())),
        Err(RenderError::from(display.clone())),
    );
    let error = result.unwrap_err();
    assert_eq!(error.native_display_failure(), Some(&display));
    assert!(matches!(
        error.gui_surface_outcome(),
        Some(RenderGuiSurfaceOutcome::FaultWithSurfaceChange { .. })
    ));
}

#[test]
fn compound_observed_fault_retains_its_primary_closure_and_surface_metadata() {
    let primary = RenderError::rejected("submit device fault", "successful submit");
    let display = display_admission_failure();
    let closure = capture_closure();
    let observed = RenderError::from(display.clone())
        .with_window_target(closure)
        .with_surface_change(owned_change());
    let combined = primary.clone().with_observed_failure(observed.clone());
    assert_eq!(combined.primary_failure(), Some(&primary));
    assert_eq!(combined.observed_failure(), Some(&observed));
    assert_eq!(combined.native_window_failure(), Some(&closure));
    assert_eq!(combined.native_display_failure(), Some(&display));
    assert!(observed.gui_surface_outcome().is_some());
    assert_eq!(source_chain_length(&combined), 1);
}

fn display_admission_failure() -> AspectDisplayLifecycleFailure {
    AspectDisplayConnection::acquire_lease(
        &Default::default(),
        AspectDisplayLeasePurpose::VulkanInstance,
    )
    .err()
    .unwrap()
}
fn capture_closure() -> AspectNativeWindowFailure {
    AspectNativeWindowClosure {
        original_handle: 91,
        cause: AspectNativeWindowClosureCause::DestroyNotify,
        behavior_version: 1,
    }
    .failure(AspectNativeWindowOperation::Capture)
}
