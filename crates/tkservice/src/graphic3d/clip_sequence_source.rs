//! Native collection subclasses share one genuine embedded clipping sequence.
use super::Graphic3dSequenceOfHClipPlane;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{RuntimeType, RuntimeTyped, Transient};
use std::io;

/// Extend native collection diagnostics; `source.clip_sequence_base().size()`.
pub trait Graphic3dClipPlaneSequenceSource: Transient {
    /// Borrow native inherited storage; `source.clip_sequence_base().to_override_global()`.
    fn clip_sequence_base(&self) -> &Graphic3dSequenceOfHClipPlane;
    /// Dispatch actual native subclass diagnostics; `source.dump_json(&mut stream,-1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        self.clip_sequence_base().dump_json(stream, depth)
    }
}
impl RuntimeTyped for dyn Graphic3dClipPlaneSequenceSource {
    const TYPE_NAME: &'static str = "Graphic3d_SequenceOfHClipPlane";
    fn static_type() -> &'static RuntimeType {
        // Native handles and their source interfaces share descriptor identity.
        Graphic3dSequenceOfHClipPlane::static_type()
    }
    fn project_from(object: &dyn Transient) -> Option<&Self> {
        rustycad_tkernel::standard::project_interface::<Self>(object)
    }
}
impl std::ops::Deref for dyn Graphic3dClipPlaneSequenceSource {
    type Target = Graphic3dSequenceOfHClipPlane;
    fn deref(&self) -> &Self::Target {
        self.clip_sequence_base()
    }
}
impl Graphic3dClipPlaneSequenceSource for Graphic3dSequenceOfHClipPlane {
    fn clip_sequence_base(&self) -> &Graphic3dSequenceOfHClipPlane {
        self
    }
}
