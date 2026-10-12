//! Native plane subclass transport borrows the genuine embedded plane state.
use super::Graphic3dClipPlane;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Handle, RuntimeType, RuntimeTyped, Transient};
use std::io;

/// Extend the real plane without copying its base; `source.clip_plane_base().get_equation()`.
pub trait Graphic3dClipPlaneSource: Transient {
    /// Borrow native inherited storage; `source.clip_plane_base().is_on()`.
    fn clip_plane_base(&self) -> &Graphic3dClipPlane;
    /// Dispatch native virtual cloning; `let copy=source.clone_plane()`.
    fn clone_plane(&self) -> Handle<dyn Graphic3dClipPlaneSource> {
        graphic3d_clip_plane_clone(self.clip_plane_base())
    }
    /// Dispatch native virtual diagnostics; `source.dump_json(&mut stream,-1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        self.clip_plane_base().dump_json(stream, depth)
    }
}
impl RuntimeTyped for dyn Graphic3dClipPlaneSource {
    const TYPE_NAME: &'static str = "Graphic3d_ClipPlane";
    fn static_type() -> &'static RuntimeType {
        // Concrete and source projections represent one native class descriptor.
        Graphic3dClipPlane::static_type()
    }
    fn project_from(object: &dyn Transient) -> Option<&Self> {
        rustycad_tkernel::standard::project_interface::<Self>(object)
    }
}
impl std::ops::Deref for dyn Graphic3dClipPlaneSource {
    type Target = Graphic3dClipPlane;
    fn deref(&self) -> &Self::Target {
        // Every nonvirtual query acts on the actual inherited fields, not a retained core copy.
        self.clip_plane_base()
    }
}
impl Graphic3dClipPlaneSource for Graphic3dClipPlane {
    fn clip_plane_base(&self) -> &Graphic3dClipPlane {
        self
    }
}
/// Execute the qualified native base Clone body; `graphic3d_clip_plane_clone(&base)`.
pub fn graphic3d_clip_plane_clone(
    base: &Graphic3dClipPlane,
) -> Handle<dyn Graphic3dClipPlaneSource> {
    Handle::new(Graphic3dClipPlane::copy_of(base)).into_cast()
}
