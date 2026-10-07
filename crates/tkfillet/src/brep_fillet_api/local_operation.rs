// Rust translation of BRepFilletAPI_LocalOperation.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFilletAPI_LocalOperation`: the contour interface the 3d fillet and chamfer
//! algorithms share.
use crate::chfi_ds::CircSection;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

/// `BRepFilletAPI_LocalOperation`: an algorithm built on contours of tangent edges
/// (indices are 1-based as natively). Implemented by `BRepFilletAPI_MakeFillet` and
/// `BRepFilletAPI_MakeChamfer`; `fn count(op: &dyn BRepFilletAPILocalOperation) -> i32 {
/// op.nb_contours() }`.
pub trait BRepFilletAPILocalOperation: BRepBuilderAPIMakeShape {
    /// `Add(E)`: a contour of the tangent edges through the edge.
    fn add(&mut self, e: &Edge) -> Result<(), Failure>;

    /// `ResetContour(IC)`: the contour without its information.
    fn reset_contour(&mut self, ic: i32) -> Result<(), Failure>;

    /// `NbContours()`.
    fn nb_contours(&self) -> i32;

    /// `Contour(E)`: the index of the contour of the edge, 0 without one.
    fn contour(&self, e: &Edge) -> i32;

    /// `NbEdges(I)`: the number of edges of the contour.
    fn nb_edges(&self, i: i32) -> Result<i32, Failure>;

    /// `Edge(I, J)`: the edge `J` of the contour `I`.
    fn edge(&self, i: i32, j: i32) -> Result<Edge, Failure>;

    /// `Remove(E)`: removes the contour of the edge.
    fn remove(&mut self, e: &Edge) -> Result<(), Failure>;

    /// `Length(IC)`: the length of the contour.
    fn length(&self, ic: i32) -> Result<f64, Failure>;

    /// `FirstVertex(IC)`.
    fn first_vertex(&self, ic: i32) -> Result<Vertex, Failure>;

    /// `LastVertex(IC)`.
    fn last_vertex(&self, ic: i32) -> Result<Vertex, Failure>;

    /// `Abscissa(IC, V)`: the abscissa of the vertex on the contour.
    fn abscissa(&self, ic: i32, v: &Vertex) -> Result<f64, Failure>;

    /// `RelativeAbscissa(IC, V)`: the abscissa of the vertex in [0, 1].
    fn relative_abscissa(&self, ic: i32, v: &Vertex) -> Result<f64, Failure>;

    /// `ClosedAndTangent(IC)`: whether the contour is closed and tangent.
    fn closed_and_tangent(&self, ic: i32) -> Result<bool, Failure>;

    /// `Closed(IC)`: whether the contour is closed.
    fn closed(&self, ic: i32) -> Result<bool, Failure>;

    /// `Reset()`: the state before `Build`, keeping the contours and their parameters.
    fn reset(&mut self) -> Result<(), Failure>;

    /// `Simulate(IC)`: the sections of the contour without the surfaces.
    fn simulate(&mut self, ic: i32) -> Result<(), Failure>;

    /// `NbSurf(IC)`: the number of surfaces of a simulated contour.
    fn nb_surf(&self, ic: i32) -> Result<i32, Failure>;

    /// `Sect(IC, IS)`: the sections of the surface `IS` of a simulated contour.
    fn sect(&self, ic: i32, is: i32) -> Result<Handle<HArray1<CircSection>>, Failure>;
}
