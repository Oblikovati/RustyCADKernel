//! Native First aliases and diagnostics inspect live actual handle fields without owning copies.
use super::clip_sequence_storage::ClipPlaneNodeLease;
use super::{
    Graphic3dClipPlane, Graphic3dSequenceOfHClipPlane, Graphic3dSequenceOfHClipPlaneIterator,
};
use crate::json_dump::{field_dumped, field_integer};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::no_such_object;
use rustycad_tkernel::standard::{Failure, Handle};
use std::io;
impl Graphic3dSequenceOfHClipPlane {
    /// Borrow the actual first handle field; `let alias=planes.first_ref()?`.
    pub fn first_ref(&self) -> Result<ClipPlaneNodeLease, Failure> {
        self.clip_plane_items.read().first().map(|field|field.storage_lease()).ok_or_else(|| {
            no_such_object("NCollection_Sequence::First - empty clipping sequence; expected a live first field")
        })
    }
    /// Explicitly owning convenience snapshot of First; `planes.first()?`.
    pub fn first(&self) -> Result<Handle<Graphic3dClipPlane>, Failure> {
        // Native const-reference coverage belongs to first_ref, never this copied convenience.
        Ok(self
            .first_ref()?
            .observe()
            .observe_as::<Graphic3dClipPlane>()
            .clone())
    }
    /// Dispatch real plane diagnostics and read Next after callbacks; `planes.dump_json(&mut stream,-1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        field_integer(
            stream,
            "myToOverrideGlobal",
            i32::from(self.to_override_global()),
        )?;
        // OCCT 3d097a0 depth-zero macros cannot invoke child callbacks or dereference planes.
        if depth == 0 {
            return Ok(());
        }
        let mut cursor = Graphic3dSequenceOfHClipPlaneIterator::new(self);
        while cursor.more() {
            dump_clip_current(&cursor, stream, depth)?;
            cursor.next();
        }
        Ok(())
    }
}
fn dump_clip_current(
    cursor: &Graphic3dSequenceOfHClipPlaneIterator,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    let field = cursor
        .value_ref()
        .map_err(|failure| io::Error::other(failure.to_string()))?;
    let plane = field.observe();
    let Some(plane) = plane.get() else {
        // Null fields are valid sequence entries and native diagnostics simply omit them.
        return Ok(());
    };
    field_dumped(stream, depth, "aClipPlane.get()", &|output, nesting| {
        plane.dump_json(output, nesting)
    })
}
