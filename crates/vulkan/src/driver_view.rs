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
    retained: Graphic3dRetainedView,
    device: SharedRenderDevice,
    limit: u32,
    vertical_sync: std::sync::Arc<SharedState<bool>>,
    presenter: SharedState<Option<crate::backend::WindowPresenter>>,
    window_error: SharedState<Option<RenderError>>,
    outcome: SharedState<Option<Result<FrameCapture, RenderError>>>,
    raster_feedback: crate::raster_feedback::RasterFailureFeedback,
    stencil_feedback: crate::stencil_feedback::StencilFeedback,
    fragment_alpha_feedback: crate::fragment_alpha_feedback::FragmentAlphaFeedback,
}
rustycad_tkernel::impl_transient!(Graphic3dRenderView, "Martensite_RenderView", StandardTransient, transient, interfaces:[dyn Graphic3dCView, dyn Graphic3dDataStructureManager]);
impl Graphic3dRenderView {
    pub(crate) fn create(
        manager: &Handle<Graphic3dStructureManager>,
        layers: &[Handle<Graphic3dLayer>],
        device: SharedRenderDevice,
        limit: u32,
        vertical_sync: std::sync::Arc<SharedState<bool>>,
    ) -> Result<Handle<Self>, Failure> {
        let view = Handle::new(Self {
            transient: StandardTransient::default(),
            retained: Graphic3dRetainedView::new(manager, layers)?,
            device,
            limit,
            vertical_sync,
            presenter: SharedState::default(),
            window_error: SharedState::default(),
            outcome: SharedState::default(),
            raster_feedback: Default::default(),
            stencil_feedback: Default::default(),
            fragment_alpha_feedback: Default::default(),
        });
        graphic3d_cview_register(&view.cast())?;
        Ok(view)
    }
    /// Inspect this view's completed stencil context; `render_view.stencil_context_state()`.
    pub fn stencil_context_state(&self) -> crate::RenderStencilState {
        self.stencil_feedback.current()
    }
    /// Read the last frame or its error; `render_view.capture()?` never conceals a failed redraw.
    pub fn capture(&self) -> Result<FrameCapture, RenderError> {
        if self.is_removed() {
            return Err(rejected("removed view", "a live view capture"));
        }
        self.outcome
            .read()
            .clone()
            .unwrap_or_else(|| Err(rejected("no redraw", "a successfully rendered view")))
    }
    /// Render the current view and retain errors for void Graphic3d callbacks; `render_view.try_redraw()?`.
    pub fn try_redraw(&self) -> Result<(), RenderError> {
        let outcome = self.render_current();
        let result = outcome.as_ref().map(|_| ()).map_err(Clone::clone);
        *self.outcome.write() = Some(outcome);
        if result.is_ok() {
            self.retained.redraw();
        } else {
            self.retained.invalidate();
        }
        result
    }
    fn render_current(&self) -> Result<FrameCapture, RenderError> {
        if !self.is_defined() || self.is_removed() {
            return Err(rejected("undefined or removed view", "a defined live view"));
        }
        if let Some(error) = self.window_error.read().as_ref() {
            return Err(error.clone());
        }
        let extent = capture_extent(self, self.limit)?;
        let mut frame = frame_for_view(self, extent, self.retained.render_clip_planes())?;
        frame.attach_raster_feedback(&self.raster_feedback);
        frame.attach_stencil_context(&self.stencil_feedback);
        frame.attach_fragment_alpha(&self.fragment_alpha_feedback);
        self.render_target(&frame)
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
    fn release_presenter(&self) {
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
        graphic3d_cview_remove(self);
    }
    fn redraw(&self) {
        let _ = self.try_redraw();
    }
    fn redraw_immediate(&self) {
        let _ = self.try_redraw();
    }
    fn resized(&self) {
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
        self.invalidate();
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
