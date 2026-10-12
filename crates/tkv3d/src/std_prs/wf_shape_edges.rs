//! Native wire/free/shared edge classification and polygon-before-curve priority.
use super::{StdPrsDeflectionCurve, StdPrsToolTriangulatedShape};
use crate::prs3d::{Prs3d, Prs3dDrawer};
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::BRepAdaptorCurve,
    topexp,
    topods::{self, Edge, Shape},
    toptools::IndexedDataMapOfShapeListOfShape,
};
use rustycad_tkernel::{
    ncollection::{HSequence, List, Sequence},
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::*;

/// Native wireframe builders; `StdPrsWfShape::add_all_edges(&shape,&drawer)?` samples real topology.
pub struct StdPrsWfShape;
impl StdPrsWfShape {
    /// Preserve wire/free/shared category ordering in one segments array; `Self::add_all_edges(&shape,&drawer)?`.
    pub fn add_all_edges(
        shape: &Shape,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<Handle<Graphic3dArrayOfPrimitives>, Failure> {
        let deflection = StdPrsToolTriangulatedShape::get_deflection(shape, drawer)?;
        let mut polylines = List::default();
        for category in classified_wf_edges(shape)? {
            append_wf_edge_polylines(&category, drawer, deflection, &mut polylines)?;
        }
        Prs3d::primitives_from_polylines(&polylines)
    }
}
pub(super) fn classified_wf_edges(shape: &Shape) -> Result<[Vec<Edge>; 3], Failure> {
    let mut map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shape, ShapeEnum::Edge, ShapeEnum::Face, &mut map)?;
    let mut categories = [Vec::new(), Vec::new(), Vec::new()];
    for index in 1..=map.extent() {
        let edge = map.find_key(index)?;
        let ancestors = map.find_from_index(index)?;
        categories[(ancestors.length() as usize).min(2)].push(topods::edge(edge)?);
    }
    Ok(categories)
}
pub(super) fn append_wf_edge_polylines(
    edges: &[Edge],
    drawer: &Handle<Prs3dDrawer>,
    deflection: f64,
    polylines: &mut List<Handle<HSequence<Point3>>>,
) -> Result<(), Failure> {
    for edge in edges {
        if BRepTool::degenerated(edge)? {
            continue;
        }
        let points = wf_edge_points(edge, drawer, deflection)?;
        if !points.is_empty() {
            polylines.append(Handle::new(HSequence::from_owned(points)));
        }
    }
    Ok(())
}
fn wf_edge_points(
    edge: &Edge,
    drawer: &Handle<Prs3dDrawer>,
    deflection: f64,
) -> Result<Sequence<Point3>, Failure> {
    let (polygon, mesh, location) = BRepTool::first_polygon_on_triangulation(edge)?;
    if !polygon.is_null() {
        return polygon_mesh_points(&polygon, &mesh, location.transformation());
    }
    let (polygon, location) = BRepTool::polygon3d(edge)?;
    if !polygon.is_null() {
        return Ok(wf_points_sequence(
            polygon
                .nodes()
                .iter()
                .map(|point| point.transformed(location.transformation())),
        ));
    }
    sampled_wf_curve(edge, drawer, deflection)
}
fn polygon_mesh_points(
    polygon: &rustycad_tkmath::poly::PolygonOnTriangulation,
    mesh: &rustycad_tkmath::poly::Triangulation,
    transform: rustycad_tkmath::gp::Transform3,
) -> Result<Sequence<Point3>, Failure> {
    let mut points = Sequence::default();
    for index in polygon.nodes().iter() {
        points.append(mesh.node(*index)?.transformed(transform));
    }
    Ok(points)
}
fn sampled_wf_curve(
    edge: &Edge,
    drawer: &Handle<Prs3dDrawer>,
    deflection: f64,
) -> Result<Sequence<Point3>, Failure> {
    let mut points = Sequence::default();
    if BRepTool::is_geometric_edge(edge)? {
        let curve = BRepAdaptorCurve::with_edge(edge)?;
        StdPrsDeflectionCurve::add_points(
            &Handle::default(),
            &curve,
            deflection,
            drawer,
            &mut points,
            false,
        )?;
    }
    Ok(points)
}
fn wf_points_sequence(points: impl Iterator<Item = Point3>) -> Sequence<Point3> {
    let mut sequence = Sequence::default();
    for point in points {
        sequence.append(point);
    }
    sequence
}
