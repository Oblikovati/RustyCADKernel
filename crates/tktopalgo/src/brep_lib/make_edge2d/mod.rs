// Rust translation of BRepLib_MakeEdge2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib_MakeEdge2d`: an edge on a 2d curve of the current `BRepLib::Plane()`.
mod constructors;
mod init;
mod project;

use super::errors::BRepLibEdgeError;
use super::make_shape::BRepLibMakeShape;
use rustycad_tkbrep::topods::{self, Edge, Vertex};
use rustycad_tkernel::standard::Failure;
use std::ops::{Deref, DerefMut};

/// `BRepLib_MakeEdge2d` (docs/topalgo-design.md section 7): builds an edge on a 2d
/// curve of `BRepLib::Plane()`, with vertices of tolerance Precision::Confusion() at
/// the plane points of the curve ends. The native member `myError` is not set by a
/// successful construction (it is uninitialized natively); it keeps its previous
/// value, `BRepLib_EdgeDone` before any failure.
///
/// ```
/// use rustycad_tkmath::gp::Point2;
/// use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge2d;
/// let maker = BRepLibMakeEdge2d::from_points(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0))?;
/// assert!(maker.is_done());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepLibMakeEdge2d {
    base: BRepLibMakeShape,
    my_error: BRepLibEdgeError,
    my_vertex1: Vertex,
    my_vertex2: Vertex,
}

impl Deref for BRepLibMakeEdge2d {
    type Target = BRepLibMakeShape;
    fn deref(&self) -> &BRepLibMakeShape {
        &self.base
    }
}

impl DerefMut for BRepLibMakeEdge2d {
    fn deref_mut(&mut self) -> &mut BRepLibMakeShape {
        &mut self.base
    }
}

impl BRepLibMakeEdge2d {
    /// The maker before its Init (the native class has no default constructor).
    fn blank() -> Self {
        Self {
            base: BRepLibMakeShape::new(),
            my_error: BRepLibEdgeError::EdgeDone,
            my_vertex1: Vertex::default(),
            my_vertex2: Vertex::default(),
        }
    }

    /// A new maker on which `init` runs, as each native constructor calls one Init.
    fn initialized(init: impl FnOnce(&mut Self) -> Result<(), Failure>) -> Result<Self, Failure> {
        let mut maker = Self::blank();
        init(&mut maker)?;
        Ok(maker)
    }

    /// Records a failed construction; the shape and done flag are kept.
    fn record(&mut self, error: BRepLibEdgeError) -> Result<(), Failure> {
        self.my_error = error;
        Ok(())
    }

    /// `Error()`: the last recorded failure (see the type documentation);
    /// `maker.error()`.
    pub fn error(&self) -> BRepLibEdgeError {
        self.my_error
    }

    /// `Edge()` and `operator TopoDS_Edge()`: the edge; StdFail_NotDone when not done;
    /// `maker.edge()?`.
    pub fn edge(&self) -> Result<Edge, Failure> {
        topods::edge(self.shape()?)
    }

    /// `Vertex1()`: the FORWARD vertex; StdFail_NotDone when not done;
    /// `maker.vertex1()?`.
    pub fn vertex1(&self) -> Result<&Vertex, Failure> {
        self.check()?;
        Ok(&self.my_vertex1)
    }

    /// `Vertex2()`: the REVERSED vertex; StdFail_NotDone when not done;
    /// `maker.vertex2()?`.
    pub fn vertex2(&self) -> Result<&Vertex, Failure> {
        self.check()?;
        Ok(&self.my_vertex2)
    }
}
