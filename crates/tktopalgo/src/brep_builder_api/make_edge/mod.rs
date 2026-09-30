// Rust translation of BRepBuilderAPI_MakeEdge.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_MakeEdge`: an edge on a 3d curve, or on a 2d curve of a surface,
//! built by the wrapped `BRepLib_MakeEdge`.
mod constructors;
mod inits;

use super::errors::BRepBuilderAPIEdgeError;
use super::make_shape::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
use super::wrap;
use crate::brep_lib::BRepLibMakeEdge;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::standard::Failure;

/// `BRepBuilderAPI_MakeEdge` (docs/topalgo-design.md section 10): each constructor and
/// Init runs the `BRepLib_MakeEdge` member of the same Rust name; `IsDone` and `Error`
/// are those of the BRepLib maker.
///
/// ```
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_builder_api::{
///     BRepBuilderAPIEdgeError, BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape,
/// };
/// let maker = BRepBuilderAPIMakeEdge::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0))?;
/// assert!(maker.is_done());
/// let same = BRepBuilderAPIMakeEdge::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 0.0))?;
/// assert_eq!(same.error(), BRepBuilderAPIEdgeError::LineThroughIdenticPoints);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepBuilderAPIMakeEdge {
    base: BRepBuilderAPIMakeShapeBase,
    my_make_edge: BRepLibMakeEdge,
}

impl Default for BRepBuilderAPIMakeEdge {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepBuilderAPIMakeShape for BRepBuilderAPIMakeEdge {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `IsDone()`: the flag of the BRepLib maker; `maker.is_done()`.
    fn is_done(&self) -> bool {
        self.my_make_edge.is_done()
    }
}

impl BRepBuilderAPIMakeEdge {
    /// `BRepBuilderAPI_MakeEdge()`: an empty BRepLib maker, not done;
    /// `BRepBuilderAPIMakeEdge::new()`.
    pub fn new() -> Self {
        Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_make_edge: BRepLibMakeEdge::new(),
        }
    }

    /// Wraps a constructed BRepLib maker and takes its shape when it is done.
    fn wrap(my_make_edge: BRepLibMakeEdge) -> Result<Self, Failure> {
        let mut maker = Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_make_edge,
        };
        maker.adopt()?;
        Ok(maker)
    }

    /// The step after each BRepLib call: done and the shape when the BRepLib maker is.
    fn adopt(&mut self) -> Result<(), Failure> {
        wrap::adopt(&mut self.base, &self.my_make_edge)
    }

    /// `Error()`: the status of the BRepLib maker, through the native switch;
    /// `maker.error()`.
    pub fn error(&self) -> BRepBuilderAPIEdgeError {
        BRepBuilderAPIEdgeError::from(self.my_make_edge.error())
    }

    /// `Edge()` and `operator TopoDS_Edge()`: the edge of the BRepLib maker;
    /// StdFail_NotDone when it is not done; `maker.edge()?`.
    pub fn edge(&self) -> Result<Edge, Failure> {
        self.my_make_edge.edge()
    }

    /// `Vertex1()`: the first vertex of the BRepLib maker; StdFail_NotDone when it is not
    /// done; `maker.vertex1()?`.
    pub fn vertex1(&self) -> Result<&Vertex, Failure> {
        self.my_make_edge.vertex1()
    }

    /// `Vertex2()`: the last vertex of the BRepLib maker; StdFail_NotDone when it is not
    /// done; `maker.vertex2()?`.
    pub fn vertex2(&self) -> Result<&Vertex, Failure> {
        self.my_make_edge.vertex2()
    }
}
