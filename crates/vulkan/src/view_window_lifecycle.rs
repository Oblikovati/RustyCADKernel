//! Known native closure must win over cached capture and offscreen target selection.
use crate::{Graphic3dRenderView, RenderError};
use rustycad_tkservice::{
    aspect::AspectNativeWindowOperation,
    graphic3d::{Graphic3dCView, Graphic3dCViewMembers},
};

impl Graphic3dRenderView {
    pub(crate) fn require_capture_target(&self) -> Result<(), RenderError> {
        self.require_native_target(AspectNativeWindowOperation::Capture)?;
        if self.is_removed() {
            return Err(crate::error::rejected(
                "removed view",
                "a live view capture",
            ));
        }
        Ok(())
    }

    pub(crate) fn gui_capture_extent(&self) -> Result<Option<[u32; 2]>, RenderError> {
        let requires_extent = self.gui_layers.read().packet.is_some();
        requires_extent
            .then(|| crate::view_target::capture_extent(self, self.limit))
            .transpose()
    }

    pub(crate) fn check_publication_target(
        &self,
        current_epoch: &std::sync::Arc<()>,
        rendered_epoch: &std::sync::Arc<()>,
        outcome: Result<crate::FrameCapture, RenderError>,
    ) -> Result<crate::FrameCapture, RenderError> {
        self.require_native_target(AspectNativeWindowOperation::Redraw)?;
        if !std::sync::Arc::ptr_eq(current_epoch, rendered_epoch) {
            return Err(crate::error::rejected(
                "composition or window binding changed during rendering",
                "a redraw with unchanged composition and window binding",
            ));
        }
        outcome
    }

    pub(crate) fn require_native_target(
        &self,
        operation: AspectNativeWindowOperation,
    ) -> Result<(), RenderError> {
        let window = self.window();
        let closure = window
            .get()
            .and_then(|window| window.native_window_closure());
        match closure {
            Some(closure) => Err(closure.failure(operation).into()),
            None => Ok(()),
        }
    }

    pub(crate) fn reject_closed_resize(&self) -> bool {
        let Err(error) = self.require_native_target(AspectNativeWindowOperation::Resize) else {
            return false;
        };
        self.release_presenter();
        *self.outcome.write() = Some(Err(error));
        self.invalidate();
        true
    }

    pub(crate) fn invalidate_window_binding(&self) {
        self.gui_layers.write().invalidate_binding();
        *self.outcome.write() = None;
        self.invalidate();
    }
}
