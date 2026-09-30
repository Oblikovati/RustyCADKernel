// Rust translation of IntCurveSurface_ThePolyhedronToolOfHInter.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurveSurface_ThePolyhedronToolOfHInter`: static access to the polyhedron,
//! the `ToolPolyh` of the Intf interference.
use super::ThePolyhedronOfHInter;
use crate::intf::PolyhedronTool;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point3;

/// `IntCurveSurface_ThePolyhedronToolOfHInter`: every member forwards to the
/// polyhedron.
///
/// ```
/// use rustycad_tkgeomalgo::int_curve_surface::{ThePolyhedronOfHInter, ThePolyhedronToolOfHInter};
/// fn triangles(p: &ThePolyhedronOfHInter) -> i32 {
///     ThePolyhedronToolOfHInter::nb_triangles(p)
/// }
/// ```
pub struct ThePolyhedronToolOfHInter;

impl ThePolyhedronToolOfHInter {
    /// `Bounding(thePolyh)`; `ThePolyhedronToolOfHInter::bounding(&p)`.
    pub fn bounding(polyh: &ThePolyhedronOfHInter) -> &BoundingBox3 {
        polyh.bounding()
    }
    /// `ComponentsBounding(thePolyh)`; `ThePolyhedronToolOfHInter::components_bounding(&p)`.
    pub fn components_bounding(polyh: &ThePolyhedronOfHInter) -> &Array1<'static, BoundingBox3> {
        polyh.components_bounding()
    }
    /// `DeflectionOverEstimation(thePolyh)`;
    /// `ThePolyhedronToolOfHInter::deflection_over_estimation(&p)`.
    pub fn deflection_over_estimation(polyh: &ThePolyhedronOfHInter) -> f64 {
        polyh.deflection_over_estimation()
    }
    /// `NbTriangles(thePolyh)`; `ThePolyhedronToolOfHInter::nb_triangles(&p)`.
    pub fn nb_triangles(polyh: &ThePolyhedronOfHInter) -> i32 {
        polyh.nb_triangles()
    }
    /// `Triangle(thePolyh, Index, P1, P2, P3)`; `ThePolyhedronToolOfHInter::triangle(&p, 1)`.
    pub fn triangle(polyh: &ThePolyhedronOfHInter, index: i32) -> [i32; 3] {
        polyh.triangle(index)
    }
    /// `Point(thePolyh, Index)`; `ThePolyhedronToolOfHInter::point(&p, 1)?`.
    pub fn point(polyh: &ThePolyhedronOfHInter, index: i32) -> Result<Point3, Failure> {
        polyh.point(index)
    }
    /// `TriConnex(thePolyh, Triang, Pivot, Pedge, TriCon, OtherP)`;
    /// `ThePolyhedronToolOfHInter::tri_connex(&p, 1, 1, 2)`.
    pub fn tri_connex(
        polyh: &ThePolyhedronOfHInter,
        triang: i32,
        pivot: i32,
        pedge: i32,
    ) -> (i32, i32, i32) {
        polyh.tri_connex(triang, pivot, pedge)
    }
    /// `IsOnBound(thePolyh, Index1, Index2)`; `ThePolyhedronToolOfHInter::is_on_bound(&p, 1, 2)`.
    pub fn is_on_bound(polyh: &ThePolyhedronOfHInter, index1: i32, index2: i32) -> bool {
        polyh.is_on_bound(index1, index2)
    }
    /// `GetBorderDeflection(thePolyh)`; `ThePolyhedronToolOfHInter::get_border_deflection(&p)`.
    pub fn get_border_deflection(polyh: &ThePolyhedronOfHInter) -> f64 {
        polyh.get_border_deflection()
    }
    /// `Dump(thePolyh)`: the polyhedron's Dump, which writes nothing;
    /// `ThePolyhedronToolOfHInter::dump(&p)`.
    pub fn dump(polyh: &ThePolyhedronOfHInter) {
        polyh.dump();
    }
}

impl PolyhedronTool for ThePolyhedronToolOfHInter {
    type Polyhedron = ThePolyhedronOfHInter;
    fn bounding(polyhedron: &ThePolyhedronOfHInter) -> &BoundingBox3 {
        polyhedron.bounding()
    }
    fn components_bounding(polyhedron: &ThePolyhedronOfHInter) -> &Array1<'static, BoundingBox3> {
        polyhedron.components_bounding()
    }
    fn deflection_over_estimation(polyhedron: &ThePolyhedronOfHInter) -> f64 {
        polyhedron.deflection_over_estimation()
    }
    fn triangle(polyhedron: &ThePolyhedronOfHInter, index: i32) -> Result<[i32; 3], Failure> {
        Ok(polyhedron.triangle(index))
    }
    fn point(polyhedron: &ThePolyhedronOfHInter, index: i32) -> Result<Point3, Failure> {
        polyhedron.point(index)
    }
    fn tri_connex(
        polyhedron: &ThePolyhedronOfHInter,
        triang: i32,
        pivot: i32,
        pedge: i32,
    ) -> Result<(i32, i32, i32), Failure> {
        Ok(polyhedron.tri_connex(triang, pivot, pedge))
    }
    fn is_on_bound(
        polyhedron: &ThePolyhedronOfHInter,
        index1: i32,
        index2: i32,
    ) -> Result<bool, Failure> {
        Ok(polyhedron.is_on_bound(index1, index2))
    }
    fn get_border_deflection(polyhedron: &ThePolyhedronOfHInter) -> f64 {
        polyhedron.get_border_deflection()
    }
}
