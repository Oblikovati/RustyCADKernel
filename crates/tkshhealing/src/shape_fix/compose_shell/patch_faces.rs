// Rust translation of ShapeFix_ComposeShell::SplitByGrid, DispatchWires and MakeFacesOnPatch
// (the public entry points) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The face-building entry points of `ShapeFix_ComposeShell` used by `Perform`.
use super::dispatch_wires::dispatch_wires;
use super::{ShapeFixComposeShell, ShapeFixWireSegment};
use crate::shape_fix::FixRoot;
use crate::shape_fix::face::make_faces_on_patch::make_faces_on_patch;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;

impl ShapeFixComposeShell {
    /// `SplitByGrid(seqw)`: splits the loaded segments by every joint line of the grid and
    /// adds the interior joint segments; `composer.split_by_grid(&mut segments)?`.
    pub fn split_by_grid(
        &self,
        segments: &mut Sequence<ShapeFixWireSegment>,
    ) -> Result<(), Failure> {
        super::split_by_grid::split_by_grid(self, segments)
    }

    /// `DispatchWires(faces, wires)`: moves every collected wire onto the patch that holds
    /// its middle point and appends the faces built on each patch to `faces`.
    /// Example: `composer.dispatch_wires(&mut faces, &mut wires)?`.
    pub fn dispatch_wires(
        &self,
        faces: &mut Sequence<Shape>,
        wires: &mut Sequence<ShapeFixWireSegment>,
    ) -> Result<(), Failure> {
        for group in dispatch_wires(self, wires)? {
            let mut loops = Sequence::default();
            for shape in group.loops {
                loops.append(shape);
            }
            self.make_faces_on_patch(faces, &group.surface, &mut loops)?;
        }
        Ok(())
    }

    /// `MakeFacesOnPatch(faces, surf, loops)`: builds the faces of one patch from its loops
    /// (outer wires with their holes) and appends them to `faces`.
    /// Example: `composer.make_faces_on_patch(&mut faces, &patch, &mut loops)?`.
    pub fn make_faces_on_patch(
        &self,
        faces: &mut Sequence<Shape>,
        surface: &Handle<dyn GeomSurface>,
        loops: &mut Sequence<Shape>,
    ) -> Result<(), Failure> {
        let (_, original, location, _) = self.collection_inputs();
        let invert_edge_status = self.fields.read().invert_edge_status;
        let built = make_faces_on_patch(
            surface,
            loops,
            &original,
            &location,
            &self.context(),
            invert_edge_status,
        )?;
        for face in built {
            faces.append(face.into_shape());
        }
        Ok(())
    }
}
