//! A real Graphic3d view owns capture results while TKService owns common scene bookkeeping.
use crate::{
    FrameCapture, RenderError,
    driver_backend::SharedRenderDevice,
    error::rejected,
    view_capture::dump_capture,
    view_forward::retained_view_methods,
    view_scene::frame_for_view,
    view_target::{CaptureTarget, capture_extent},
};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::{aspect::*, graphic3d::*, image::ImagePixMap};
/// Renderer-backed Graphic3d view; `view.cast::<Graphic3dRenderView>().capture()?`.
pub struct Graphic3dRenderView {
    transient: StandardTransient,
    pub(crate) retained: Graphic3dRetainedView,
    device: SharedRenderDevice,
    pub(crate) limit: u32,
    vertical_sync: std::sync::Arc<SharedState<bool>>,
    presenter: SharedState<Option<crate::backend::WindowPresenter>>,
    window_error: SharedState<Option<RenderError>>,
    pub(crate) outcome: SharedState<Option<Result<FrameCapture, RenderError>>>,
    pub(crate) gui_layers: SharedState<crate::view_gui_layer::GuiLayerSlot>,
    raster_feedback: crate::raster_feedback::RasterFailureFeedback,
    stencil_feedback: crate::stencil_feedback::StencilFeedback,
    fragment_alpha_feedback: crate::fragment_alpha_feedback::FragmentAlphaFeedback,
}
rustycad_tkernel::impl_transient!(Graphic3dRenderView, "Martensite_RenderView", StandardTransient, transient, interfaces:[dyn Graphic3dCView, dyn Graphic3dDataStructureManager]);
type RenderViewCreationInputs = (
    StandardTransient,
    Graphic3dRetainedView,
    SharedRenderDevice,
    u32,
    std::sync::Arc<SharedState<bool>>,
);
impl Graphic3dRenderView {
    pub(crate) fn create(
        manager: &Handle<Graphic3dStructureManager>,
        layers: &[Handle<Graphic3dLayer>],
        device: SharedRenderDevice,
        limit: u32,
        vertical_sync: std::sync::Arc<SharedState<bool>>,
    ) -> Result<Handle<Self>, Failure> {
        let inputs = (
            StandardTransient::default(),
            Graphic3dRetainedView::new(manager, layers)?,
            device,
            limit,
            vertical_sync,
        );
        let view = Handle::new(Self::initialized_gui_view(inputs));
        graphic3d_cview_register(&view.cast())?;
        Ok(view)
    }
    fn initialized_gui_view(inputs: RenderViewCreationInputs) -> Self {
        let (transient, retained, device, limit, vertical_sync) = inputs;
        Self {
            transient,
            retained,
            device,
            limit,
            vertical_sync,
            presenter: SharedState::default(),
            window_error: SharedState::default(),
            outcome: SharedState::default(),
            gui_layers: SharedState::default(),
            raster_feedback: Default::default(),
            stencil_feedback: Default::default(),
            fragment_alpha_feedback: Default::default(),
        }
    }
    /// Inspect this view's completed stencil context; `render_view.stencil_context_state()`.
    pub fn stencil_context_state(&self) -> crate::RenderStencilState {
        self.stencil_feedback.current()
    }
    /// Read the last frame or its error; `render_view.capture()?` never conceals a failed redraw.
    pub fn capture(&self) -> Result<FrameCapture, RenderError> {
        self.require_capture_target()?;
        let extent = self.gui_capture_extent()?;
        let gui = self.gui_layers.read();
        if let Some(extent) = extent {
            gui.snapshot(extent)?;
        }
        let outcome = self
            .outcome
            .read()
            .clone()
            .unwrap_or_else(|| Err(rejected("no redraw", "a successfully rendered view")));
        self.require_capture_target()?;
        outcome
    }
    /// Render the current view and retain errors for void Graphic3d callbacks; `render_view.try_redraw()?`.
    pub fn try_redraw(&self) -> Result<(), RenderError> {
        let epoch = self.gui_layers.read().epoch.clone();
        let outcome = self.render_current();
        self.publish_gui_outcome(&epoch, outcome)
    }
    fn publish_gui_outcome(
        &self,
        epoch: &std::sync::Arc<()>,
        outcome: Result<FrameCapture, RenderError>,
    ) -> Result<(), RenderError> {
        let gui = self.gui_layers.read();
        let outcome = self.check_publication_target(&gui.epoch, epoch, outcome);
        let result = outcome.as_ref().map(|_| ()).map_err(Clone::clone);
        *self.outcome.write() = Some(outcome);
        self.update_gui_redraw_status(result.is_ok());
        result
    }
    fn update_gui_redraw_status(&self, succeeded: bool) {
        if succeeded {
            self.retained.redraw();
        } else {
            self.retained.invalidate();
        }
    }
    fn render_current(&self) -> Result<FrameCapture, RenderError> {
        self.require_native_target(AspectNativeWindowOperation::Redraw)?;
        if !self.is_defined() || self.is_removed() {
            return Err(rejected("undefined or removed view", "a defined live view"));
        }
        if let Some(error) = self.window_error.read().as_ref() {
            return Err(error.clone());
        }
        let mut frame = self.gui_composed_frame()?;
        frame.attach_raster_feedback(&self.raster_feedback);
        frame.attach_stencil_context(&self.stencil_feedback);
        frame.attach_fragment_alpha(&self.fragment_alpha_feedback);
        self.render_target(&frame)
    }
    fn gui_composed_frame(&self) -> Result<crate::RenderFrame, RenderError> {
        let extent = capture_extent(self, self.limit)?;
        let packet = self.gui_layers.read().snapshot(extent)?;
        let mut frame = frame_for_view(self, extent, self.retained.render_clip_planes())?;
        if let Some(packet) = packet {
            crate::view_gui_composition::append_gui_packet(&mut frame, packet)?;
        }
        Ok(frame)
    }
}
impl Graphic3dRenderView {
    fn render_target(&self, frame: &crate::RenderFrame) -> Result<FrameCapture, RenderError> {
        let window = self.window();
        let mut device = self.device.write();
        if !self.fbo().is_null() || window.is_virtual() {
            return device.render(frame);
        }
        if window.native_handle() == 0 {
            return Err(rejected(
                0,
                "a nonzero native window handle or a virtual window",
            ));
        }
        if !window.is_mapped() {
            return device.render(frame);
        }
        let mut presenter = self.presenter.write();
        if presenter.is_none() {
            *presenter = Some(device.window_presenter(&window)?);
        }
        let result = device.render_window(
            frame,
            presenter.as_mut().unwrap(),
            *self.vertical_sync.read(),
        );
        if result.is_err() {
            presenter.take();
        }
        result
    }
    pub(crate) fn release_presenter(&self) {
        let _device = self.device.write();
        self.presenter.write().take();
    }
}
impl Drop for Graphic3dRenderView {
    fn drop(&mut self) {
        self.release_presenter();
    }
}
impl Graphic3dCView for Graphic3dRenderView {
    retained_view_methods!();
    fn cview_base(&self) -> &Graphic3dCViewBase {
        self.retained.cview_base()
    }
    fn remove(&self) {
        self.release_presenter();
        let retired = self.gui_layers.write().retire_removed();
        drop(retired);
        graphic3d_cview_remove(self);
    }
    fn redraw(&self) {
        let _ = self.try_redraw();
    }
    fn redraw_immediate(&self) {
        let _ = self.try_redraw();
    }
    fn resized(&self) {
        if self.reject_closed_resize() {
            return;
        }
        if self.gui_layers.write().resized() {
            self.invalidate_gui_capture();
        }
        graphic3d_cview_resized(self);
        self.invalidate();
    }
    fn set_window(
        &self,
        parent: &Handle<dyn Graphic3dCView>,
        window: &Handle<dyn AspectWindow>,
        context: AspectRenderingContext,
    ) {
        self.release_presenter();
        *self.window_error.write() = (!parent.is_null() || context != 0).then(|| {
            rejected(
                (parent.object_address(), context),
                "a root view without an external GL context",
            )
        });
        self.retained.set_window(parent, window, context);
        self.invalidate_window_binding();
    }
    fn buffer_dump(&self, image: &ImagePixMap, kind: &Graphic3dBufferType) -> bool {
        self.capture()
            .is_ok_and(|capture| dump_capture(image, kind, &capture))
    }
    fn fbo_create(&self, width: i32, height: i32) -> Handle<StandardTransient> {
        CaptureTarget::create(width, height, self.limit)
    }
    fn fbo_release(&self, fbo: &mut Handle<StandardTransient>) {
        *fbo = Handle::default();
    }
    fn fbo_get_dimensions(&self, fbo: &Handle<StandardTransient>) -> [i32; 4] {
        fbo.cast::<CaptureTarget>()
            .get()
            .map_or([0; 4], CaptureTarget::dimensions)
    }
    fn fbo_change_viewport(&self, fbo: &Handle<StandardTransient>, width: i32, height: i32) {
        if let Some(target) = fbo.cast::<CaptureTarget>().get() {
            target.resize(width, height);
        }
    }
    fn diagnostic_information(
        &self,
        dict: &mut Graphic3dFrameStatsDict,
        flags: Graphic3dDiagnosticInfo,
    ) {
        graphic3d_cview_diagnostic_information(self, dict, flags);
        dict.add("Martensite device".into(), self.device.read().name().into());
        if let Some(Err(error)) = self.outcome.read().as_ref() {
            dict.add(
                "Martensite render error".into(),
                error.to_string().as_str().into(),
            );
        }
    }
}

#[cfg(test)]
#[path = "view_binding_tests.rs"]
mod view_binding_tests;
