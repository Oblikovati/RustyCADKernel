use super::algo_tools3d;
use crate::int_tools::Context;
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{Edge, Face, ShapeIterator},
};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::top_abs::{self, ShapeEnum};
use rustycad_tkmath::gp::{Point2, Point3};
/// Find the opposite oriented occurrence on a face, retaining output on failure; `get_edge_off(&edge,&face,&mut found)?`.
pub fn get_edge_off(edge: &Edge, face: &Face, found: &mut Edge) -> Result<bool, Failure> {
    for shape in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if shape.is_same(&edge.clone().into_shape())
            && shape.orientation() == top_abs::reverse(edge.orientation())
        {
            *found = Edge::unchecked(shape);
            return Ok(true);
        }
    }
    Ok(false)
}
/// Find the first edge among direct face wires, retaining output on failure; `get_edge_on_face(&edge,&face,&mut found)?`.
pub fn get_edge_on_face(edge: &Edge, face: &Face, found: &mut Edge) -> Result<bool, Failure> {
    for wire in ShapeIterator::new(face, true, true) {
        for shape in ShapeIterator::new(&wire, true, true) {
            if shape.is_same(&edge.clone().into_shape()) {
                *found = Edge::unchecked(shape);
                return Ok(true);
            }
        }
    }
    Ok(false)
}
/// Apply OCCT's interior-point projection and tolerance criterion; `are_faces_same_domain(&first,&second,&context,0.)?`.
pub fn are_faces_same_domain(
    first: &Face,
    second: &Face,
    context: &Context,
    fuzz: f64,
) -> Result<bool, Failure> {
    let mut point = Point3::default();
    let mut uv = Point2::default();
    if algo_tools3d::point_in_face(first, &mut point, &mut uv, context)? != 0 {
        return Ok(false);
    }
    let edge_tolerance = max_boundary_tolerance(first)?;
    let first_tolerance = raised_tolerance(BRepTool::face_tolerance(first)?, edge_tolerance);
    let second_tolerance = raised_tolerance(BRepTool::face_tolerance(second)?, edge_tolerance);
    let fuzz = if fuzz < precision::confusion() {
        precision::confusion()
    } else {
        fuzz
    };
    context.is_valid_point_for_face(point, second, first_tolerance + second_tolerance + fuzz)
}
fn max_boundary_tolerance(face: &Face) -> Result<f64, Failure> {
    let mut tolerance = -1.;
    for shape in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = Edge::unchecked(shape);
        if !BRepTool::degenerated(&edge)? {
            tolerance = raised_tolerance(tolerance, BRepTool::edge_tolerance(&edge)?);
        }
    }
    Ok(tolerance)
}
fn raised_tolerance(tolerance: f64, candidate: f64) -> f64 {
    if candidate > tolerance {
        candidate
    } else {
        tolerance
    }
}
