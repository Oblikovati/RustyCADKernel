// SPDX-License-Identifier: GPL-2.0-only
//! Independently authored transforms: F(W(E(t,0,0))) = (-1,t,2), UV=(-3,t).
use super::fixtures::imported_v2;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_graph::{
        CoEdgeId,
        tool::{co_edge, edge, face},
    },
    topods::{Edge, Face, Shape, Wire},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::{
    geom::{GeomPlane, GeomSurface},
    top_abs::Orientation,
};
use rustycad_tkmath::{
    gp::{Direction2, Plane3, Point2, Point3, Transform3},
    toploc::Location,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

fn location(rows: [[f64; 4]; 3]) -> Location {
    let mut transform = Transform3::default();
    transform.set_values(rows).unwrap();
    Location::new(transform)
}
fn placements() -> [Location; 3] {
    [
        location([[1., 0., 0., 2.], [0., 1., 0., 0.], [0., 0., 1., 2.]]),
        location([[0., -1., 0., 0.], [1., 0., 0., 0.], [0., 0., 1., 0.]]),
        location([[1., 0., 0., 0.], [0., 1., 0., 3.], [0., 0., 1., 0.]]),
    ]
}
fn nested_face(reverse: bool) -> Shape {
    let [face_place, wire_place, edge_place] = placements();
    let edge = nested_edge(&edge_place, reverse);
    let wire = nested_wire(&edge, &wire_place);
    let mut face = nested_plane();
    BRepBuilder::default().add(&mut face, &wire).unwrap();
    face.set_location(&face_place, false).unwrap();
    let parent = (&face_place * &wire_place).unwrap();
    let cumulative = Edge::unchecked(edge.moved(&parent, false).unwrap());
    attach_nested_pcurve(&cumulative, &face);
    face.into_shape()
}
fn nested_edge(placement: &Location, reverse: bool) -> Edge {
    let mut edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(1., 0., 0.))
            .unwrap()
            .edge()
            .unwrap();
    edge.set_location(placement, false).unwrap();
    if reverse {
        edge.reverse();
    }
    edge
}
fn nested_wire(edge: &Edge, placement: &Location) -> Wire {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, edge).unwrap();
    wire.set_location(placement, false).unwrap();
    wire
}
fn nested_plane() -> Face {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::from_pln(Plane3::default())).into_cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}
fn attach_nested_pcurve(edge: &Edge, face: &Face) {
    let pcurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(Point2::new(-3., 0.), Direction2::Y)).into_cast();
    let builder = BRepBuilder::default();
    builder
        .update_edge_pcurve_on_face(edge, &pcurve, face, 1e-7)
        .unwrap();
    builder.range_on_face(edge, face, 0., 1.).unwrap();
}
#[test]
fn cumulative_v2_preserves_noncommuting_face_wire_edge_frames() {
    for reverse in [false, true] {
        let graph = imported_v2(&nested_face(reverse), false);
        let id = CoEdgeId::new(0);
        assert_eq!(graph.topo().co_edges().nb(), 1);
        let definition = graph.topo().co_edges().definition(id);
        assert_eq!(definition.orientation.is_reversed, reverse);
        assert_eq!(co_edge::range(&graph, id), (0., 1.));
        let uv = co_edge::pcurve(&graph, id).value(0.25).unwrap();
        assert_eq!(uv, Point2::new(-3., 0.25));
        let world = edge::curve(&graph, definition.child_edge_id)
            .value(0.25)
            .unwrap();
        assert_eq!(world, Point3::new(-1., 0.25, 2.));
        assert_eq!(
            face::surface(&graph, definition.face_id)
                .value(uv.x(), uv.y())
                .unwrap(),
            world
        );
    }
}
#[test]
fn cumulative_v2_face_reversal_keeps_local_coedge_orientation() {
    let source = nested_face(false).oriented(Orientation::Reversed);
    let graph = imported_v2(&source, false);
    let definition = graph.topo().co_edges().definition(CoEdgeId::new(0));
    assert!(!definition.orientation.is_reversed);
    assert!(co_edge::has_pcurve(&graph, CoEdgeId::new(0)));
}
