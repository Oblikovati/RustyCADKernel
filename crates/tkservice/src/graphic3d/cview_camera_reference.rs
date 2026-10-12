//! Native Camera references stay bound to their selected field through replacement and callbacks.
use super::{
    Graphic3dCView, Graphic3dCViewBase, Graphic3dCViewFields, Graphic3dCamera,
    Graphic3dDataStructureManager,
};
use rustycad_tkernel::standard::{Handle, HandleFieldRef, HandleReference};
/// Borrow actual myCamera without retaining its target; `graphic3d_cview_camera_ref(view.cview_base())`.
pub fn graphic3d_cview_camera_ref(
    base: &Graphic3dCViewBase,
) -> Box<dyn HandleReference<Graphic3dCamera> + '_> {
    Box::new(HandleFieldRef::new(&base.fields, retained_cview_camera))
}
fn retained_cview_camera(fields: &Graphic3dCViewFields) -> &Handle<Graphic3dCamera> {
    // Direct XR myCamera assignments must remain visible to this retained field alias.
    &fields.camera
}

use rustycad_tkernel::standard::dump::DumpStream;
use std::io;
impl<T: Graphic3dCView> Graphic3dDataStructureManager for T {
    /// The native virtual CView override selects the actual borrowed slot.
    fn camera_ref(&self) -> Box<dyn HandleReference<Graphic3dCamera> + '_> {
        self.cview_camera_ref()
    }
    /// `DumpJson`: the view dump.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        self.cview_dump_json(stream, depth)
    }
}
