//! Pending results cannot cross an explicit rebind, including equal native IDs and extents.
use super::*;
use crate::{Graphic3dRenderDriver, RenderBackend, RenderFrame, RenderLayer};

struct FixedBindingBackend;
impl RenderBackend for FixedBindingBackend {
    fn render(&mut self, _frame: &RenderFrame) -> Result<FrameCapture, RenderError> {
        Ok(binding_capture())
    }
}
fn binding_capture() -> FrameCapture {
    FrameCapture {
        width: 1,
        height: 1,
        rgba: vec![0, 255, 0, 255],
        depth: vec![1.],
    }
}
fn binding_fixture() -> (Handle<Graphic3dRenderView>, Handle<AspectNeutralWindow>) {
    binding_fixture_for_policy(crate::RenderGuiSurfacePolicy::LegacyV1)
}
fn binding_fixture_for_policy(
    policy: crate::RenderGuiSurfacePolicy,
) -> (Handle<Graphic3dRenderView>, Handle<AspectNeutralWindow>) {
    let driver =
        Graphic3dRenderDriver::with_backend_and_gui_surface_policy(FixedBindingBackend, 64, policy)
            .unwrap();
    let manager = Handle::new(Graphic3dStructureManager::new(&driver.cast()).unwrap());
    let view = driver
        .create_view(&manager)
        .unwrap()
        .cast::<Graphic3dRenderView>();
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_virtual(true);
    window.set_native_handle(91);
    window.set_size(1, 1);
    view.set_window(&Handle::default(), &window.cast(), 0);
    (view, window)
}
fn attach_binding_gui(view: &Graphic3dRenderView) {
    let packet = RenderLayer::new(Vec::new(), false, false, false);
    view.attach_gui_layer([1, 1], packet).unwrap();
}
fn configure_binding_gui(view: &Graphic3dRenderView, with_gui: bool) {
    if with_gui {
        attach_binding_gui(view);
    }
}
fn assert_pending_binding_refused(with_gui: bool) {
    let (view, window) = binding_fixture();
    configure_binding_gui(&view, with_gui);
    let pending_epoch = view.gui_layers.read().epoch.clone();
    view.set_window(&Handle::default(), &window.cast(), 0);
    assert_eq!(view.window().native_handle(), 91);
    assert_eq!(view.window().size(), (1, 1));
    assert!(
        view.publish_gui_outcome(&pending_epoch, Ok(binding_capture()))
            .is_err()
    );
    assert!(view.capture().is_err());
    configure_binding_gui(&view, with_gui);
    view.try_redraw().unwrap();
    assert_eq!(view.capture().unwrap().rgba, binding_capture().rgba);
}
#[test]
fn no_gui_pending_capture_cannot_cross_equal_handle_rebind() {
    assert_pending_binding_refused(false);
}
#[test]
fn gui_pending_capture_cannot_cross_equal_handle_rebind() {
    assert_pending_binding_refused(true);
}

#[test]
fn checked_gui_pending_capture_cannot_cross_equal_handle_rebind() {
    let (view, window) = binding_fixture_for_policy(crate::RenderGuiSurfacePolicy::CheckedV2);
    attach_binding_gui(&view);
    let pending = view.gui_layers.read().target_stamp();
    view.set_window(&Handle::default(), &window.cast(), 0);
    let failure = view
        .publish_checked_gui(pending, Ok(binding_capture()))
        .unwrap_err();
    let Some(crate::RenderGuiSurfaceOutcome::SurfaceChanged(change)) =
        failure.gui_surface_outcome()
    else {
        panic!("expected typed binding change")
    };
    assert!(change.target_revision_changed);
    assert!(!change.composition_revision_changed);
    assert_eq!(change.observed_extent, Some(change.accepted_extent));
    assert!(view.capture().is_err());
    verify_binding_recovery(&view);
}
#[test]
fn checked_policy_keeps_model_only_pending_rebind_refusal() {
    let (view, window) = binding_fixture_for_policy(crate::RenderGuiSurfacePolicy::CheckedV2);
    let pending = view.gui_layers.read().epoch.clone();
    view.set_window(&Handle::default(), &window.cast(), 0);
    assert!(
        view.publish_gui_outcome(&pending, Ok(binding_capture()))
            .is_err()
    );
    assert!(view.capture().is_err());
    view.try_redraw().unwrap();
    assert_eq!(view.capture().unwrap().rgba, binding_capture().rgba);
}

fn verify_binding_recovery(view: &Graphic3dRenderView) {
    attach_binding_gui(view);
    view.try_redraw().unwrap();
    assert_eq!(view.capture().unwrap().rgba, binding_capture().rgba);
}
