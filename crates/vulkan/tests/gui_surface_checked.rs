//! Independent checked outcomes for real fault plus target change, Oblikovati#3630.
#[path = "support/gui_surface_fixture.rs"]
mod fixture;
use fixture::*;
use martensite_vulkan::*;
use std::sync::atomic::Ordering;
fn checked_fixture(with_gui: bool) -> (GuiFaultFixture, ResizeAndFailBackend) {
    let policy = Some(RenderGuiSurfacePolicy::CheckedV2);
    gui_fault_fixture(with_gui, policy)
}
fn primary_change(error: &RenderError) -> &RenderGuiSurfaceChange {
    let Some(RenderGuiSurfaceOutcome::FaultWithSurfaceChange { fault, change }) =
        error.gui_surface_outcome()
    else {
        panic!("expected retained primary fault: {error:?}")
    };
    assert_eq!(fault, &injected_primary_fault());
    assert_eq!(error.to_string(), fault.to_string());
    change
}
#[test]
fn checked_resize_retains_primary_fault_and_requires_new_packet() {
    let (scene, backend) = checked_fixture(true);
    *backend.resize.write() = Some(false);
    let error = scene.view.try_redraw().unwrap_err();
    let change = primary_change(&error);
    assert_eq!(
        (change.accepted_extent, change.observed_extent),
        ([64, 64], Some([80, 72]))
    );
    assert_eq!(change.fresh_packet, RenderGuiFreshPacket::Required);
    assert!(change.target_revision_changed);
    assert!(!change.composition_revision_changed);
    primary_change(&scene.view.capture().unwrap_err());
}
#[test]
fn checked_away_and_back_reports_revision_without_extent_mismatch() {
    let (scene, backend) = checked_fixture(true);
    *backend.resize.write() = Some(true);
    let error = scene.view.try_redraw().unwrap_err();
    let change = primary_change(&error);
    assert_eq!(change.cause, RenderGuiSurfaceCause::TargetRevision);
    assert_eq!(change.observed_extent, Some(change.accepted_extent));
    assert_eq!(change.fresh_packet, RenderGuiFreshPacket::Required);
}
#[test]
fn checked_newer_packet_stays_current_while_primary_fault_survives() {
    let (scene, backend) = checked_fixture(true);
    *backend.resize.write() = Some(false);
    backend.replace.store(true, Ordering::Relaxed);
    let error = scene.view.try_redraw().unwrap_err();
    assert_eq!(
        primary_change(&error).fresh_packet,
        RenderGuiFreshPacket::AlreadyCurrent
    );
    *backend.resize.write() = None;
    backend.succeed.store(true, Ordering::Relaxed);
    scene.view.try_redraw().unwrap();
    assert_eq!(scene.view.capture().unwrap().width, 80);
}
#[test]
fn checked_capture_observed_resize_cannot_resurrect_earlier_pixels() {
    let (scene, backend) = checked_fixture(true);
    backend.succeed.store(true, Ordering::Relaxed);
    scene.view.try_redraw().unwrap();
    scene.window.set_size(80, 72);
    let failure = scene.view.capture().unwrap_err();
    assert!(matches!(
        failure.gui_surface_outcome(),
        Some(RenderGuiSurfaceOutcome::SurfaceChanged(_))
    ));
    scene.window.set_size(64, 64);
    assert!(scene.view.capture().is_err());
    scene
        .view
        .attach_gui_layer([64, 64], empty_gui_packet())
        .unwrap();
    scene.view.try_redraw().unwrap();
    assert_eq!(scene.view.capture().unwrap().rgba, vec![91; 64 * 64 * 4]);
}
#[test]
fn checked_capture_preserves_stored_fault_after_extent_change() {
    let (scene, _) = checked_fixture(true);
    assert_eq!(
        scene.view.try_redraw().unwrap_err(),
        injected_primary_fault()
    );
    scene.window.set_size(80, 72);
    let error = scene.view.capture().unwrap_err();
    assert_eq!(primary_change(&error).phase, RenderGuiSurfacePhase::Capture);
}
#[test]
fn checked_policy_does_not_reclassify_model_faults() {
    let (scene, backend) = checked_fixture(false);
    *backend.resize.write() = Some(false);
    assert_eq!(
        scene.view.try_redraw().unwrap_err(),
        injected_primary_fault()
    );
    assert_eq!(scene.view.capture().unwrap_err(), injected_primary_fault());
}
#[test]
fn clearing_gui_requires_new_model_frame_then_allows_capture() {
    let (scene, backend) = checked_fixture(true);
    backend.succeed.store(true, Ordering::Relaxed);
    scene.view.try_redraw().unwrap();
    scene.view.clear_gui_layer().unwrap();
    assert!(scene.view.capture().is_err());
    scene.view.try_redraw().unwrap();
    assert!(scene.view.capture().is_ok());
}
fn checked_outcome_bytes() -> Vec<u8> {
    let (scene, backend) = checked_fixture(true);
    *backend.resize.write() = Some(true);
    let failure = scene.view.try_redraw().unwrap_err();
    let change = primary_change(&failure);
    format!(
        "version={} change={change:?} primary={failure}",
        change.behavior_version()
    )
    .into_bytes()
}
#[test]
fn checked_outcome_bytes_are_stable_across_independent_host_threads() {
    let expected = checked_outcome_bytes();
    let threads: Vec<_> = (0..8)
        .map(|_| std::thread::spawn(repeated_checked_bytes))
        .collect();
    for thread in threads {
        for actual in thread.join().unwrap() {
            assert_eq!(actual, expected);
        }
    }
    println!("{{\"case\":\"checked-gui-canonical\",\"bytes\":{expected:?}}}");
}
fn repeated_checked_bytes() -> Vec<Vec<u8>> {
    let repetitions = 16;
    (0..repetitions).map(|_| checked_outcome_bytes()).collect()
}
#[test]
fn published_failure_values_are_isolated_from_later_window_and_packet_mutation() {
    let (scene, backend) = checked_fixture(true);
    *backend.resize.write() = Some(true);
    let published = scene.view.try_redraw().unwrap_err();
    let pinned = format!("{:?}", primary_change(&published));
    scene.window.set_size(80, 72);
    scene
        .view
        .attach_gui_layer([80, 72], empty_gui_packet())
        .unwrap();
    backend.succeed.store(true, Ordering::Relaxed);
    *backend.resize.write() = None;
    scene.view.try_redraw().unwrap();
    assert_eq!(format!("{:?}", primary_change(&published)), pinned);
}
#[test]
fn rejected_attachment_reports_current_packet_without_invalidating_it() {
    let (scene, backend) = checked_fixture(true);
    backend.succeed.store(true, Ordering::Relaxed);
    scene.view.try_redraw().unwrap();
    let failure = scene
        .view
        .attach_gui_layer([32, 64], empty_gui_packet())
        .unwrap_err();
    let Some(RenderGuiSurfaceOutcome::SurfaceChanged(change)) = failure.gui_surface_outcome()
    else {
        panic!("expected typed extent refusal")
    };
    assert_eq!(change.phase, RenderGuiSurfacePhase::Attachment);
    assert_eq!(change.fresh_packet, RenderGuiFreshPacket::AlreadyCurrent);
    assert!(scene.view.capture().is_ok());
}
