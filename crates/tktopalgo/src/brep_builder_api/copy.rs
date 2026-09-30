// Rust translation of BRepBuilderAPI_Copy.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_Copy`: a copy of a shape with new topology, and new geometry and mesh
//! on request (`BRepTools_CopyModification`).
use super::make_shape::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
use super::modify_shape::BRepBuilderAPIModifyShape;
use rustycad_tkbrep::brep_tools::{BRepToolsModification, CopyModification};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use std::ops::{Deref, DerefMut};

/// `BRepBuilderAPI_Copy` (docs/topalgo-design.md section 10): every sub-shape of the
/// copy is new; with `copy_geom` the curves and surfaces are copied too, with
/// `copy_mesh` the triangulations and polygons.
///
/// ```
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_builder_api::{
///     BRepBuilderAPICopy, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeVertex,
/// };
/// let vertex = BRepBuilderAPIMakeVertex::new(Point3::new(1.0, 2.0, 3.0))?.vertex()?;
/// let mut copy = BRepBuilderAPICopy::with_shape(&vertex, true, false)?;
/// assert!(copy.is_done());
/// assert!(!copy.shape()?.is_same(&vertex));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepBuilderAPICopy {
    modify: BRepBuilderAPIModifyShape,
}

impl Default for BRepBuilderAPICopy {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BRepBuilderAPICopy {
    type Target = BRepBuilderAPIModifyShape;
    fn deref(&self) -> &BRepBuilderAPIModifyShape {
        &self.modify
    }
}

impl DerefMut for BRepBuilderAPICopy {
    fn deref_mut(&mut self) -> &mut BRepBuilderAPIModifyShape {
        &mut self.modify
    }
}

impl BRepBuilderAPIMakeShape for BRepBuilderAPICopy {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        self.modify.make_shape_base()
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        self.modify.make_shape_base_mut()
    }

    /// `Modified(S)` (inherited from ModifyShape): the image of `S`;
    /// `copy.modified(&face)?`.
    fn modified(&mut self, shape: &Shape) -> Result<&List<Shape>, Failure> {
        self.modify.modified(shape)
    }
}

/// `new BRepTools_CopyModification(copyGeom, copyMesh)`.
fn copy_modification(copy_geom: bool, copy_mesh: bool) -> Handle<dyn BRepToolsModification> {
    Handle::new(CopyModification::new(copy_geom, copy_mesh)).cast()
}

impl BRepBuilderAPICopy {
    /// `BRepBuilderAPI_Copy()`: a copy of the geometry without the mesh, no shape yet;
    /// `BRepBuilderAPICopy::new()`.
    pub fn new() -> Self {
        Self {
            modify: BRepBuilderAPIModifyShape::with_modification(&copy_modification(true, false)),
        }
    }

    /// `BRepBuilderAPI_Copy(S, copyGeom, copyMesh)`: copies `S`;
    /// `BRepBuilderAPICopy::with_shape(&s, true, false)?`.
    pub fn with_shape(shape: &Shape, copy_geom: bool, copy_mesh: bool) -> Result<Self, Failure> {
        let mut copy = Self {
            modify: BRepBuilderAPIModifyShape::with_modification(&copy_modification(
                copy_geom, copy_mesh,
            )),
        };
        copy.modify.do_modif_shape(shape)?;
        Ok(copy)
    }

    /// `Perform(S, copyGeom, copyMesh)`: a new modification, then a forced copy of `S`
    /// (the command is marked not done first, so the same shape is copied again);
    /// `copy.perform(&s, true, false)?`.
    pub fn perform(
        &mut self,
        shape: &Shape,
        copy_geom: bool,
        copy_mesh: bool,
    ) -> Result<(), Failure> {
        self.modify.my_modification = copy_modification(copy_geom, copy_mesh);
        // on force la copie si on vient deja d`en faire une
        self.modify.set_not_done();
        self.modify.do_modif_shape(shape)
    }
}
