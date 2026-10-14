//! Publication owns the last result and checks the accepted composition epoch.
use super::*;
impl Graphic3dRenderView {
    /// Read the last frame or its error; `render_view.capture()?` never conceals a failed redraw.
    pub fn capture(&self) -> Result<FrameCapture, RenderError> {
        if self.checked_gui_policy() && self.gui_layers.read().capture_stamp().extent.is_some() {
            return self.checked_gui_capture();
        }
        self.legacy_gui_capture()
    }
    fn legacy_gui_capture(&self) -> Result<FrameCapture, RenderError> {
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
        let stamp = self.gui_layers.read().target_stamp();
        if self.checked_gui_policy() && stamp.extent.is_some() {
            return self.checked_gui_redraw(stamp);
        }
        let epoch = self.gui_layers.read().epoch.clone();
        let outcome = self.render_current();
        self.publish_gui_outcome(&epoch, outcome)
    }
    pub(super) fn publish_gui_outcome(
        &self,
        epoch: &std::sync::Arc<()>,
        outcome: Result<FrameCapture, RenderError>,
    ) -> Result<(), RenderError> {
        let mut gui = self.gui_layers.write();
        gui.last_draw = None;
        let outcome = self.check_publication_target(&gui.epoch, epoch, outcome);
        let result = outcome.as_ref().map(|_| ()).map_err(Clone::clone);
        *self.outcome.write() = Some(outcome);
        self.update_gui_redraw_status(result.is_ok());
        result
    }
    pub(super) fn update_gui_redraw_status(&self, succeeded: bool) {
        if succeeded {
            self.retained.redraw();
        } else {
            self.retained.invalidate();
        }
    }
}
