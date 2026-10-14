// SPDX-License-Identifier: GPL-2.0-only
use super::{tests::*, *};
use crate::sketch_region::equal_occurrences;
use crate::sketch_region_candidates::tests::candidates;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Edge, Face, Shape, edge, face},
};
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg3d::{
    geom::{GeomElementarySurface, GeomPlane},
    top_abs::ShapeEnum,
};
use rustycad_tkmath::{gp::Point3, poly::Polygon3D, toploc::Location};
fn actual_cap(body: &Shape) -> Face {
    let shapes = equal_occurrences(body).unwrap();
    shapes
        .iter()
        .filter(|s| s.shape_type().unwrap() == ShapeEnum::Face)
        .map(|s| face(s).unwrap())
        .find(|f| {
            BRepTool::surface(f)
                .unwrap()
                .cast::<GeomPlane>()
                .get()
                .is_some_and(|p| p.location().z() == 0.)
        })
        .expect("expected actual original-plane cap")
}
fn actual_edge(face: &Face) -> Edge {
    let shapes = equal_occurrences(face).unwrap();
    let found = shapes
        .iter()
        .find(|s| s.shape_type().unwrap() == ShapeEnum::Edge)
        .unwrap();
    edge(found).unwrap()
}
fn attach_polygon(edge: &Edge) -> Handle<Polygon3D> {
    let point_values = [Point3::default(), Point3::new(1., 0., 0.)];
    let parameter_values = [0., 1.];
    let points = Array1::from_shared(&point_values, 1);
    let parameters = Array1::from_shared(&parameter_values, 1);
    let polygon = Handle::new(Polygon3D::from_nodes_and_parameters(&points, &parameters).unwrap());
    BRepBuilder::default()
        .update_edge_polygon3d(edge, &polygon, &Location::default())
        .unwrap();
    polygon
}
fn polygon(edge: &Edge) -> Handle<Polygon3D> {
    let (polygon, _) = BRepTool::polygon3d(edge).unwrap();
    assert!(!polygon.is_null());
    polygon
}
fn change_polygon(polygon: &Handle<Polygon3D>, coordinate: f64) {
    polygon
        .change_nodes(|nodes| nodes.set_value(2, Point3::new(coordinate, 0., 0.)))
        .unwrap();
    polygon
        .change_parameters(|parameters| parameters.set_value(2, coordinate))
        .unwrap()
        .unwrap();
    polygon.set_deflection(coordinate);
}
fn polygon_node(polygon: &Handle<Polygon3D>) -> Point3 {
    let nodes = polygon.nodes();
    *nodes.value(2).unwrap()
}
#[test]
fn candidate_plane_and_edge_polygon_are_isolated_in_both_directions() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let batch = candidates(&[source]);
    let candidate = batch.candidates[0].face.clone();
    let original = attach_polygon(&actual_edge(&candidate));
    let result = retain_selected_prism(batch, request([1., 1.]), &mut progress(None, None));
    let cap = actual_cap(&result.diagnostic.as_ref().unwrap().body);
    let copied = polygon(&actual_edge(&cap));
    assert_polygon_isolation(&original, &copied);
    assert_plane_isolation(&candidate, &cap);
}
fn assert_polygon_isolation(original: &Handle<Polygon3D>, copied: &Handle<Polygon3D>) {
    change_polygon(original, 9.);
    assert_eq!(polygon_node(copied), Point3::new(1., 0., 0.));
    assert_eq!(*copied.parameters().unwrap().value(2).unwrap(), 1.);
    change_polygon(copied, 12.);
    assert_eq!(polygon_node(original), Point3::new(9., 0., 0.));
    assert_eq!(original.deflection(), 9.);
}
fn assert_plane_isolation(original: &Face, copied: &Face) {
    let old = BRepTool::surface(original).unwrap().cast::<GeomPlane>();
    let new = BRepTool::surface(copied).unwrap().cast::<GeomPlane>();
    let before = new.get().unwrap().location();
    old.get().unwrap().set_location(Point3::new(9., 9., 9.));
    assert_eq!(new.get().unwrap().location(), before);
    new.get().unwrap().set_location(Point3::new(12., 12., 12.));
    assert_eq!(old.get().unwrap().location(), Point3::new(9., 9., 9.));
}
