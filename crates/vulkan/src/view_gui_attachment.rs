//! Replacement validates first; retained frame snapshots own their atlases through GPU completion.
use crate::view_gui_layer::{GuiLayerPacket, validate_gui_layer};
use crate::{Graphic3dRenderView, RenderError, RenderLayer, error::rejected};
use rustycad_tkservice::aspect::AspectNativeWindowOperation;
use rustycad_tkservice::graphic3d::{Graphic3dCView, Graphic3dCViewMembers};
impl Graphic3dRenderView {
    /// Attach or atomically replace an owned GUI packet; `view.attach_gui_layer([800,600], layer)?`.
    pub fn attach_gui_layer(
        &self,
        expected_extent: [u32; 2],
        layer: RenderLayer,
    ) -> Result<(), RenderError> {
        let packet = self.validated_gui_packet(expected_extent, layer)?;
        self.install_gui_packet(Some(packet));
        Ok(())
    }
    fn validated_gui_packet(
        &self,
        extent: [u32; 2],
        layer: RenderLayer,
    ) -> Result<GuiLayerPacket, RenderError> {
        self.require_gui_live_view(AspectNativeWindowOperation::AttachGui)?;
        validate_gui_layer(&layer)?;
        let actual = crate::view_target::capture_extent(self, self.limit)?;
        if actual != extent {
            return Err(rejected(extent, &format!("GUI target extent {actual:?}")));
        }
        Ok(GuiLayerPacket { extent, layer })
    }
    fn install_gui_packet(&self, packet: Option<GuiLayerPacket>) {
        let mut slot = self.gui_layers.write();
        let retired = slot.replace(packet);
        self.invalidate_gui_capture();
        drop(slot);
        drop(retired);
    }
    /// Clear UI while retaining model content; `view.clear_gui_layer()?; view.try_redraw()?`.
    pub fn clear_gui_layer(&self) -> Result<(), RenderError> {
        self.require_gui_live_view(AspectNativeWindowOperation::ClearGui)?;
        self.install_gui_packet(None);
        Ok(())
    }
    fn require_gui_live_view(
        &self,
        operation: AspectNativeWindowOperation,
    ) -> Result<(), RenderError> {
        self.require_native_target(operation)?;
        if self.is_removed() || !self.is_defined() {
            return Err(rejected(
                "undefined or removed view",
                "a defined live GUI composition target",
            ));
        }
        Ok(())
    }
    pub(crate) fn invalidate_gui_capture(&self) {
        *self.outcome.write() = Some(Err(rejected(
            "GUI composition changed",
            "a successful redraw of the current composition",
        )));
        self.retained.invalidate();
    }
}
