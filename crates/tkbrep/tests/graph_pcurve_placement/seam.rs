// SPDX-License-Identifier: GPL-2.0-only
//! Authored cylindrical seam uses; this partial face is not modelling acceptance evidence.
use super::fixtures::{imported_v2, translated};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_graph::{CoEdgeId, tool::co_edge},
    topods::{Edge, Face, Shape, Wire},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomSurface};
use rustycad_tkmath::gp::{Axis3, Direction2, Point2, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
use std::f64::consts::TAU;

fn authored_seam() -> Shape {
    let builder = BRepBuilder::default();
    let mut face = cylindrical_face();
    let edge = vertical_edge();
    builder
        .update_edge_pcurves_on_face(&edge, (&seam_curve(0.), &seam_curve(TAU)), &face, 1e-7)
        .unwrap();
    builder.range_on_face(&edge, &face, 0., 2.).unwrap();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &edge).unwrap();
    builder.add(&mut wire, &edge.reversed()).unwrap();
    builder.add(&mut face, &wire).unwrap();
    face.into_shape()
}
fn cylindrical_face() -> Face {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.).unwrap()).into_cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}
fn vertical_edge() -> Edge {
    BRepBuilderAPIMakeEdge::from_points(Point3::new(1., 0., 0.), Point3::new(1., 0., 2.))
        .unwrap()
        .edge()
        .unwrap()
}
fn seam_curve(u: f64) -> Handle<dyn Geom2dCurve> {
    let origin = Point2::new(u, 0.);
    let line = Geom2dLine::from_point(origin, Direction2::Y);
    Handle::new(line).into_cast()
}
#[test]
fn cumulative_v2_keeps_both_directed_seam_curves_and_ranges() {
    let graph = imported_v2(&translated(&authored_seam()), false);
    assert_eq!(graph.topo().co_edges().nb(), 2);
    let first = graph.topo().co_edges().definition(CoEdgeId::new(0));
    let second = graph.topo().co_edges().definition(CoEdgeId::new(1));
    assert_eq!(first.child_edge_id, second.child_edge_id);
    assert_eq!(first.face_id, second.face_id);
    assert_eq!(first.parent_wire_id, second.parent_wire_id);
    assert_ne!(first.orientation, second.orientation);
    assert_seam_curves(&graph);
}
fn assert_seam_curves(graph: &rustycad_tkbrep::brep_graph::BRepGraph) {
    for index in 0..2 {
        let id = CoEdgeId::new(index);
        let expected_u = if co_edge::is_reversed(graph, id) {
            TAU
        } else {
            0.
        };
        assert_eq!(co_edge::range(graph, id), (0., 2.));
        assert_eq!(
            co_edge::pcurve(graph, id).value(0.5).unwrap(),
            Point2::new(expected_u, 0.5)
        );
        assert!(co_edge::seam_pair(graph, id).is_valid());
    }
}
