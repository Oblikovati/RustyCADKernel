//! Nonvirtual V3d_View reference members act on the genuine embedded base.
use super::V3dViewSource;
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkservice::graphic3d::{Graphic3dCView, Graphic3dCamera};
/// Borrow retained native slots separately from owning convenience copies; `view.camera_ref().observe()`.
pub trait V3dViewMembers: V3dViewSource {
    /// Forward the real virtual DSM Camera getter; `let alias=view.camera_ref()`.
    fn camera_ref(&self) -> Box<dyn HandleReference<Graphic3dCamera> + '_> {
        self.view_base().camera_ref()
    }
    /// Borrow the actual default-camera handle; `view.default_camera_ref().observe()`.
    fn default_camera_ref(&self) -> &dyn HandleReference<Graphic3dCamera> {
        self.view_base().default_camera_ref()
    }
    /// Borrow the real retained driver-view field; `view.view_ref().observe()`.
    fn view_ref(&self) -> &dyn HandleReference<dyn Graphic3dCView> {
        self.view_base().view_ref()
    }
}
impl<V: V3dViewSource + ?Sized> V3dViewMembers for V {}
