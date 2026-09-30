//! OCC428 missing-vertex handling and native strict tolerance comparisons.
use super::*;
use crate::imesh_data::EdgeHandle;
use rustycad_tkbrep::{brep::BRepTool, topexp, topods::Vertex};
use rustycad_tkmath::gp::Point3;
fn same_with_either(vertex: &Vertex, pair: &(Vertex, Vertex)) -> bool {
    vertex.is_same(&pair.0) || vertex.is_same(&pair.1)
}
fn within_either(point: Point3, pair: [Point3; 2], tolerance: f64) -> bool {
    let squared = tolerance * tolerance;
    point.square_distance(&pair[0]) < squared || point.square_distance(&pair[1]) < squared
}
pub(super) fn common(first: &EdgeHandle, second: &EdgeHandle) -> Result<Vertex, Failure> {
    let edge1 = first.get_edge()?;
    let pair1 = topexp::vertices(&edge1, false);
    if pair1.0.is_null() || pair1.1.is_null() {
        return Ok(Vertex::default());
    }
    let edge2 = second.get_edge()?;
    if edge1.is_same(&edge2) {
        return Ok(if pair1.0.is_same(&pair1.1) {
            pair1.0
        } else {
            Vertex::default()
        });
    }
    common_distinct(pair1, topexp::vertices(&edge2, false))
}
fn common_distinct(pair1: (Vertex, Vertex), pair2: (Vertex, Vertex)) -> Result<Vertex, Failure> {
    if pair2.0.is_null() || pair2.1.is_null() {
        return Ok(Vertex::default());
    }
    if same_with_either(&pair1.0, &pair2) {
        return Ok(pair1.0);
    }
    if same_with_either(&pair1.1, &pair2) {
        return Ok(pair1.1);
    }
    common_by_tolerance(pair1, pair2)
}
fn common_by_tolerance(
    first: (Vertex, Vertex),
    second: (Vertex, Vertex),
) -> Result<Vertex, Failure> {
    let points1 = [BRepTool::pnt(&first.0)?, BRepTool::pnt(&first.1)?];
    let tolerances1 = [
        BRepTool::vertex_tolerance(&first.0)?,
        BRepTool::vertex_tolerance(&first.1)?,
    ];
    let points2 = [BRepTool::pnt(&second.0)?, BRepTool::pnt(&second.1)?];
    let tolerance2 = rustycad_tkg2d::geom2d::native::std_max(
        BRepTool::vertex_tolerance(&second.0)?,
        BRepTool::vertex_tolerance(&second.1)?,
    );
    match_tolerances(first, points1, tolerances1, points2, tolerance2)
}
fn match_tolerances(
    first: (Vertex, Vertex),
    points1: [Point3; 2],
    tolerances1: [f64; 2],
    points2: [Point3; 2],
    tolerance2: f64,
) -> Result<Vertex, Failure> {
    if within_either(points1[0], points2, tolerances1[0] + tolerance2) {
        return Ok(first.0);
    }
    if within_either(points1[1], points2, tolerances1[1] + tolerance2) {
        return Ok(first.1);
    }
    Ok(Vertex::default())
}
