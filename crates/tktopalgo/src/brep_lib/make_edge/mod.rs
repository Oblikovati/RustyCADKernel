// Rust translation of BRepLib_MakeEdge.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib_MakeEdge`: an edge on a 3d curve, or on a 2d curve of a surface, between
//! parameters, points or vertices.
mod constructors;
pub(crate) mod ends;
mod init_curve;
mod init_pcurve;
mod project;

use super::errors::BRepLibEdgeError;
use super::make_shape::BRepLibMakeShape;
use rustycad_tkbrep::topods::{self, Edge, Vertex};
use rustycad_tkernel::standard::Failure;
use std::ops::{Deref, DerefMut};

/// `BRepLib_MakeEdge` (docs/topalgo-design.md section 7): builds an edge on a curve.
/// The vertices are projected on the curve, or made at the curve ends with the
/// tolerance `BRepLib::Precision()`; a failure is recorded in [`Self::error`] and
/// leaves the maker not done.
///
/// ```
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_lib::{BRepLibEdgeError, BRepLibMakeEdge};
/// let maker = BRepLibMakeEdge::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0))?;
/// assert!(maker.is_done());
/// assert_eq!(maker.error(), BRepLibEdgeError::EdgeDone);
/// let same = BRepLibMakeEdge::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 0.0))?;
/// assert_eq!(same.error(), BRepLibEdgeError::LineThroughIdenticPoints);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepLibMakeEdge {
    base: BRepLibMakeShape,
    my_error: BRepLibEdgeError,
    my_vertex1: Vertex,
    my_vertex2: Vertex,
}

impl Default for BRepLibMakeEdge {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BRepLibMakeEdge {
    type Target = BRepLibMakeShape;
    fn deref(&self) -> &BRepLibMakeShape {
        &self.base
    }
}

impl DerefMut for BRepLibMakeEdge {
    fn deref_mut(&mut self) -> &mut BRepLibMakeShape {
        &mut self.base
    }
}

impl BRepLibMakeEdge {
    /// `BRepLib_MakeEdge()`: not done, with the error `BRepLib_PointProjectionFailed`;
    /// `BRepLibMakeEdge::new()`.
    pub fn new() -> Self {
        Self {
            base: BRepLibMakeShape::new(),
            my_error: BRepLibEdgeError::PointProjectionFailed,
            my_vertex1: Vertex::default(),
            my_vertex2: Vertex::default(),
        }
    }

    /// A new maker on which `init` runs, as each native constructor calls one Init.
    fn initialized(init: impl FnOnce(&mut Self) -> Result<(), Failure>) -> Result<Self, Failure> {
        let mut maker = Self::new();
        init(&mut maker)?;
        Ok(maker)
    }

    /// `Error()`: the status of the last construction; `maker.error()`.
    pub fn error(&self) -> BRepLibEdgeError {
        self.my_error
    }

    /// `Edge()` and `operator TopoDS_Edge()`: the edge; StdFail_NotDone when not done;
    /// `maker.edge()?`.
    pub fn edge(&self) -> Result<Edge, Failure> {
        topods::edge(self.shape()?)
    }

    /// `Vertex1()`: the FORWARD vertex (null at an infinite start); StdFail_NotDone
    /// when not done; `maker.vertex1()?`.
    pub fn vertex1(&self) -> Result<&Vertex, Failure> {
        self.check()?;
        Ok(&self.my_vertex1)
    }

    /// `Vertex2()`: the REVERSED vertex (null at an infinite end); StdFail_NotDone
    /// when not done; `maker.vertex2()?`.
    pub fn vertex2(&self) -> Result<&Vertex, Failure> {
        self.check()?;
        Ok(&self.my_vertex2)
    }
}
