//! Native Camera references stay bound to their selected field through replacement and callbacks.
use super::{Graphic3dCView, Graphic3dCViewBase, Graphic3dCViewFields, Graphic3dCamera};
use rustycad_tkernel::standard::{
    Handle, HandleReference, StandardTransient, Transient, WeakHandleFieldRef,
};
/// Borrow actual myCamera without retaining its target; `graphic3d_cview_camera_ref(view.cview_base())`.
pub fn graphic3d_cview_camera_ref(
    base: &Graphic3dCViewBase,
) -> Box<dyn HandleReference<Graphic3dCamera>> {
    // A selected alias must survive a temporary noncounting CView observation, without owning it.
    Box::new(WeakHandleFieldRef::new(&base.fields, retained_cview_camera))
}
fn retained_cview_camera(fields: &Graphic3dCViewFields) -> &Handle<Graphic3dCamera> {
    // Direct XR myCamera assignments must remain visible to this retained field alias.
    &fields.camera
}

use rustycad_tkernel::standard::dump::DumpStream;
use std::io;
impl<T: Graphic3dCView> Graphic3dDataStructureManager for T {
    /// The native virtual CView override selects the actual borrowed slot.
    fn camera_ref(&self) -> Box<dyn HandleReference<Graphic3dCamera>> {
        self.cview_camera_ref()
    }
    /// `DumpJson`: the view dump.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        self.cview_dump_json(stream, depth)
    }
}

/// `Graphic3d_DataStructureManager`: an object with a camera that structures are computed
/// for (the views).
pub trait Graphic3dDataStructureManager: Transient {
    /// Native virtual constHandle borrow; `let alias=manager.camera_ref()`.
    fn camera_ref(&self) -> Box<dyn HandleReference<Graphic3dCamera>>;
    /// Explicit owning snapshot convenience; native Camera binds camera_ref; `manager.camera()`.
    fn camera(&self) -> Handle<Graphic3dCamera> {
        self.camera_ref().clone_handle()
    }
    /// `DumpJson`: the class name; `manager.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let _ = depth;
        super::driver_dump::dump_data_structure_manager(stream)
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn Graphic3dDataStructureManager,
    "Graphic3d_DataStructureManager",
    StandardTransient
);
