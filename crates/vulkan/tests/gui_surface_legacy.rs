//! Revision-bound legacy fault ordering for Oblikovati#3630; no native I/O.
#[path = "support/gui_surface_fixture.rs"]
mod fixture;
use fixture::*;
use martensite_vulkan::*;
fn legacy_fixture(with_gui: bool) -> (GuiFaultFixture, ResizeAndFailBackend) {
    let policy = None;
    gui_fault_fixture(with_gui, policy)
}
fn assert_legacy_resize_masks_fault(return_to_original: bool) {
    let (scene, backend) = legacy_fixture(true);
    *backend.resize.write() = Some(return_to_original);
    let failure = scene.view.try_redraw().unwrap_err();
    assert_eq!(
        failure,
        RenderError::rejected(
            "composition or window binding changed during rendering",
            "a redraw with unchanged composition and window binding",
        )
    );
    assert_legacy_stale_capture(&scene.view, return_to_original);
}
fn assert_legacy_stale_capture(view: &Graphic3dRenderView, return_to_original: bool) {
    let extent = if return_to_original {
        [64, 64]
    } else {
        [80, 72]
    };
    let failure = RenderError::rejected(
        ([64, 64], extent, true),
        "a fresh GUI packet matching the current target extent",
    );
    assert_eq!(view.capture().unwrap_err(), failure);
}
#[test]
fn legacy_delivered_resize_masks_primary_device_fault() {
    let return_to_original = false;
    assert_legacy_resize_masks_fault(return_to_original);
}
#[test]
fn legacy_away_and_back_masks_fault_at_equal_extent() {
    let return_to_original = true;
    assert_legacy_resize_masks_fault(return_to_original);
}
#[test]
fn legacy_capture_extent_check_masks_stored_primary_fault() {
    let (scene, _) = legacy_fixture(true);
    assert_eq!(
        scene.view.try_redraw().unwrap_err(),
        injected_primary_fault()
    );
    assert_eq!(scene.view.capture().unwrap_err(), injected_primary_fault());
    scene.window.set_size(80, 72);
    assert_eq!(
        scene.view.capture().unwrap_err(),
        RenderError::rejected(
            ([64, 64], [80, 72], false),
            "a fresh GUI packet matching the current target extent",
        )
    );
}
#[test]
fn legacy_model_resize_preserves_primary_device_fault() {
    let (scene, backend) = legacy_fixture(false);
    *backend.resize.write() = Some(false);
    assert_eq!(
        scene.view.try_redraw().unwrap_err(),
        injected_primary_fault()
    );
    assert_eq!(scene.view.capture().unwrap_err(), injected_primary_fault());
}
