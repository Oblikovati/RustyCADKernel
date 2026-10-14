//! Explicit native gate: synchronous X resize at operation-local presentation seams.
use super::window_gui_contract::{NoPresentationProbe, WindowPresentationProbe};
use super::*;
use crate::{
    PrimitiveDraw, RenderGuiSurfaceOutcome, RenderGuiSurfacePhase, RenderGuiSurfacePolicy,
    RenderLayer, RenderVertex, identity_clip,
};
#[path = "../../examples/support/native_window.rs"]
mod native_window;
use native_window::NativeWindow;
struct NativeResizeProbe<'a> {
    window: &'a NativeWindow,
    selected: RenderGuiSurfacePhase,
    fault: bool,
    resize: bool,
    observed: Vec<RenderGuiSurfacePhase>,
}
impl<'a> NativeResizeProbe<'a> {
    fn new(
        window: &'a NativeWindow,
        selected: RenderGuiSurfacePhase,
        fault: bool,
        resize: bool,
    ) -> Self {
        Self {
            window,
            selected,
            fault,
            resize,
            observed: Vec::new(),
        }
    }
}
impl WindowPresentationProbe for NativeResizeProbe<'_> {
    fn observe(&mut self, phase: RenderGuiSurfacePhase) -> Result<(), RenderError> {
        self.observed.push(phase);
        if phase != self.selected {
            return Ok(());
        }
        if self.resize {
            self.window.resize(80, 72);
        }
        if self.fault {
            return Err(native_injected_fault());
        }
        Ok(())
    }
}
fn native_injected_fault() -> RenderError {
    let operation = "owned native submission fault 3630";
    RenderError::rejected(operation, "successful device submission")
}
fn native_gui_frame([width, height]: [u32; 2]) -> RenderFrame {
    let points = [[-1., -1., 0.], [1., -1., 0.], [1., 1., 0.], [-1., 1., 0.]];
    let vertices = points
        .map(|point| RenderVertex::new(point, [0., 1., 0., 1.]))
        .to_vec();
    let draw = PrimitiveDraw::new(vertices, vec![0, 1, 2, 0, 2, 3], identity_clip()).unwrap();
    let layer = RenderLayer::new(vec![draw], false, false, false);
    let mut frame = RenderFrame::with_layers(width, height, [0., 0., 1., 1.], vec![layer]).unwrap();
    frame.gui_draw_start = Some(0);
    frame
}
fn assert_native_change(error: &RenderError, phase: RenderGuiSurfacePhase, with_fault: bool) {
    let change = match error.gui_surface_outcome().unwrap() {
        RenderGuiSurfaceOutcome::SurfaceChanged(change) => {
            assert!(!with_fault);
            change
        }
        RenderGuiSurfaceOutcome::FaultWithSurfaceChange { fault, change } => {
            assert!(with_fault);
            assert_eq!(fault, &native_injected_fault());
            change
        }
    };
    assert_eq!(change.phase, phase);
    assert_eq!(
        (change.accepted_extent, change.observed_extent),
        ([64, 64], Some([80, 72]))
    );
}
fn native_boundary_case(renderer: &mut VulkanRenderer, phase: RenderGuiSurfacePhase, fault: bool) {
    let native = NativeWindow::new(64, 64);
    let mut presenter = renderer.window_presenter(&native.window.cast()).unwrap();
    let mut probe = NativeResizeProbe::new(&native, phase, fault, true);
    let result = render_native_probe(renderer, &mut presenter, &mut probe, [64, 64]);
    assert_native_change(&result.unwrap_err(), phase, fault);
    assert_eq!(probe.observed.last(), Some(&phase));
    assert!(
        presenter.chain.is_none(),
        "failed operation must retire its swapchain"
    );
    assert_native_fresh_frame(renderer, &mut presenter, [80, 72]);
    println!(
        "{{\"case\":\"native-gui-surface\",\"phase\":\"{phase:?}\",\"injected_fault\":{fault},\"status\":\"passed\"}}"
    );
}
fn assert_native_fresh_frame(
    renderer: &mut VulkanRenderer,
    presenter: &mut WindowPresenter,
    extent: [u32; 2],
) {
    let capture =
        render_native_probe(renderer, presenter, &mut NoPresentationProbe, extent).unwrap();
    let pixels = (extent[0] * extent[1]) as usize;
    assert_eq!(capture.rgba, [0, 255, 0, 255].repeat(pixels));
    assert_eq!(capture.depth, vec![1.; pixels]);
    assert!(
        renderer.validation_errors().is_empty(),
        "{:?}",
        renderer.validation_errors()
    );
}
fn render_native_probe(
    renderer: &mut VulkanRenderer,
    presenter: &mut WindowPresenter,
    probe: &mut dyn WindowPresentationProbe,
    extent: [u32; 2],
) -> Result<FrameCapture, RenderError> {
    let frame = native_gui_frame(extent);
    renderer.render_window_with_probe(
        &frame,
        presenter,
        false,
        RenderGuiSurfacePolicy::CheckedV2,
        probe,
    )
}
fn native_same_extent_fault(renderer: &mut VulkanRenderer, phase: RenderGuiSurfacePhase) {
    let native = NativeWindow::new(64, 64);
    let mut presenter = renderer.window_presenter(&native.window.cast()).unwrap();
    let mut probe = NativeResizeProbe::new(&native, phase, true, false);
    let result = render_native_probe(renderer, &mut presenter, &mut probe, [64, 64]);
    assert_eq!(result.unwrap_err(), native_injected_fault());
    assert!(
        presenter.chain.is_none(),
        "failed operation must retire signalled synchronization resources"
    );
    assert_native_fresh_frame(renderer, &mut presenter, [64, 64]);
    println!(
        "{{\"case\":\"native-gui-same-extent-fault\",\"phase\":\"{phase:?}\",\"status\":\"passed\"}}"
    );
}

#[test]
#[ignore = "requires explicit exclusive native/GPU lease and pinned validation layer"]
fn native_gui_surface_phase_gate() {
    let mut renderer = native_validation_renderer();
    for phase in [
        RenderGuiSurfacePhase::Acquire,
        RenderGuiSurfacePhase::BeforeSubmit,
        RenderGuiSurfacePhase::Present,
        RenderGuiSurfacePhase::Publication,
    ] {
        native_boundary_case(&mut renderer, phase, false);
        native_boundary_case(&mut renderer, phase, true);
        native_same_extent_fault(&mut renderer, phase);
    }
}

fn native_validation_renderer() -> VulkanRenderer {
    let renderer = VulkanRenderer::new(VulkanOptions {
        validation: true,
        device_index: None,
    })
    .unwrap();
    println!(
        "{{\"device\":{:?},\"identity\":{:?}}}",
        renderer.device_name(),
        format!("{:?}", renderer.driver_identity())
    );
    renderer
}
