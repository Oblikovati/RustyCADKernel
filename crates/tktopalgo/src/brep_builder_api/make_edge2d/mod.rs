// Rust translation of BRepBuilderAPI_MakeEdge2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_MakeEdge2d`: an edge on a 2d curve of the plane
//! `BRepBuilderAPI::Plane()`, built by the wrapped `BRepLib_MakeEdge2d`.
mod constructors;

use super::errors::BRepBuilderAPIEdgeError;
use super::make_shape::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
use super::wrap::{self, wrapped_updates};
use crate::brep_lib::BRepLibMakeEdge2d;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::gp::Point2;

/// `BRepBuilderAPI_MakeEdge2d` (docs/topalgo-design.md section 10): each constructor
/// and Init runs the `BRepLib_MakeEdge2d` member of the same Rust name; `IsDone` and
/// `Error` are those of the BRepLib maker. There is no empty constructor, as natively.
///
/// ```
/// use rustycad_tkmath::gp::Point2;
/// use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge2d, BRepBuilderAPIMakeShape};
/// let maker = BRepBuilderAPIMakeEdge2d::from_points(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0))?;
/// assert!(maker.is_done());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepBuilderAPIMakeEdge2d {
    base: BRepBuilderAPIMakeShapeBase,
    my_make_edge2d: BRepLibMakeEdge2d,
}

impl BRepBuilderAPIMakeShape for BRepBuilderAPIMakeEdge2d {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `IsDone()`: the flag of the BRepLib maker; `maker.is_done()`.
    fn is_done(&self) -> bool {
        self.my_make_edge2d.is_done()
    }
}

impl BRepBuilderAPIMakeEdge2d {
    /// Wraps a constructed BRepLib maker and takes its shape when it is done.
    fn wrap(my_make_edge2d: BRepLibMakeEdge2d) -> Result<Self, Failure> {
        let mut maker = Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_make_edge2d,
        };
        maker.adopt()?;
        Ok(maker)
    }

    /// The step after each BRepLib call: done and the shape when the BRepLib maker is.
    fn adopt(&mut self) -> Result<(), Failure> {
        wrap::adopt(&mut self.base, &self.my_make_edge2d)
    }

    wrapped_updates!(BRepBuilderAPIMakeEdge2d, my_make_edge2d;
        init_curve(curve: &Handle<dyn Geom2dCurve>) = "Init(C)";
        init_curve_params(curve: &Handle<dyn Geom2dCurve>, p1: f64, p2: f64) = "Init(C, p1, p2)";
        init_curve_points(curve: &Handle<dyn Geom2dCurve>, p1: Point2, p2: Point2) = "Init(C, P1, P2)";
        init_curve_vertices(curve: &Handle<dyn Geom2dCurve>, v1: &Vertex, v2: &Vertex) = "Init(C, V1, V2)";
        init_curve_points_params(curve: &Handle<dyn Geom2dCurve>, point1: Point2, point2: Point2, p1: f64, p2: f64)
            = "Init(C, P1, P2, p1, p2)";
        init_curve_vertices_params(curve: &Handle<dyn Geom2dCurve>, v1: &Vertex, v2: &Vertex, p1: f64, p2: f64)
            = "Init(C, V1, V2, p1, p2)";
    );

    /// `Error()`: the status of the BRepLib maker, through the native switch;
    /// `maker.error()`.
    pub fn error(&self) -> BRepBuilderAPIEdgeError {
        BRepBuilderAPIEdgeError::from(self.my_make_edge2d.error())
    }

    /// `Edge()` and `operator TopoDS_Edge()`: the edge of the BRepLib maker;
    /// StdFail_NotDone when it is not done; `maker.edge()?`.
    pub fn edge(&self) -> Result<Edge, Failure> {
        self.my_make_edge2d.edge()
    }

    /// `Vertex1()`: the first vertex of the BRepLib maker; StdFail_NotDone when it is not
    /// done; `maker.vertex1()?`.
    pub fn vertex1(&self) -> Result<&Vertex, Failure> {
        self.my_make_edge2d.vertex1()
    }

    /// `Vertex2()`: the last vertex of the BRepLib maker; StdFail_NotDone when it is not
    /// done; `maker.vertex2()?`.
    pub fn vertex2(&self) -> Result<&Vertex, Failure> {
        self.my_make_edge2d.vertex2()
    }
}
