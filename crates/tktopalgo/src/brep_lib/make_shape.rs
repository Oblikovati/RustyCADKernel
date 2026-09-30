// Rust translation of BRepLib_MakeShape.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib_MakeShape`: the shape built by a maker and the face lists of its history.
use super::command::BRepLibCommand;
use super::errors::BRepLibShapeModification;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use std::ops::{Deref, DerefMut};

/// `BRepLib_MakeShape` (docs/topalgo-design.md section 7): the command, the built shape
/// and the lists that the history members return. Every maker holds one and
/// dereferences to it; the native constructor is protected.
///
/// ```
/// use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;
/// let maker = BRepLibMakeEdge::new();
/// assert_eq!(maker.nb_surfaces(), 0);
/// assert_eq!(maker.shape().unwrap_err().exception_type(), "StdFail_NotDone");
/// ```
#[derive(Clone, Default)]
pub struct BRepLibMakeShape {
    command: BRepLibCommand,
    /// `myShape`: the shape under construction.
    pub(crate) my_shape: Shape,
    my_gen_faces: List<Shape>,
    my_new_faces: List<Shape>,
    my_edg_faces: List<Shape>,
}

impl Deref for BRepLibMakeShape {
    type Target = BRepLibCommand;
    fn deref(&self) -> &BRepLibCommand {
        &self.command
    }
}

impl DerefMut for BRepLibMakeShape {
    fn deref_mut(&mut self) -> &mut BRepLibCommand {
        &mut self.command
    }
}

impl BRepLibMakeShape {
    /// `BRepLib_MakeShape()`: not done, a null shape and empty lists.
    pub(crate) fn new() -> Self {
        Self {
            command: BRepLibCommand::new(),
            ..Self::default()
        }
    }

    /// `Build()`: does nothing (the native member is not virtual); `maker.build()`.
    pub fn build(&mut self) {}

    /// `Shape()` and `operator TopoDS_Shape()`: the shape; StdFail_NotDone when the
    /// maker is not done (Build does nothing); `maker.shape()?`.
    pub fn shape(&self) -> Result<&Shape, Failure> {
        if !self.is_done() {
            self.check()?;
        }
        Ok(&self.my_shape)
    }

    /// `FaceStatus(F)`: `BRepLib_Trimmed` for every face; `maker.face_status(&face)`.
    pub fn face_status(&self, _face: &Face) -> BRepLibShapeModification {
        BRepLibShapeModification::Trimmed
    }

    /// `HasDescendants(F)`: true for every face; `maker.has_descendants(&face)`.
    pub fn has_descendants(&self, _face: &Face) -> bool {
        true
    }

    /// `DescendantFaces(F)`: the (always empty) generated faces;
    /// `maker.descendant_faces(&face)`.
    pub fn descendant_faces(&mut self, _face: &Face) -> &List<Shape> {
        &self.my_gen_faces
    }

    /// `NbSurfaces()`: 0; `maker.nb_surfaces()`.
    pub fn nb_surfaces(&self) -> i32 {
        0
    }

    /// `NewFaces(I)`: the (always empty) faces of surface `I`; `maker.new_faces(1)`.
    pub fn new_faces(&mut self, _index: i32) -> &List<Shape> {
        &self.my_new_faces
    }

    /// `FacesFromEdges(E)`: the (always empty) faces built from `E`;
    /// `maker.faces_from_edges(&edge)`.
    pub fn faces_from_edges(&mut self, _edge: &Edge) -> &List<Shape> {
        &self.my_edg_faces
    }
}
