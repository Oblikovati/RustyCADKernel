// SPDX-License-Identifier: GPL-2.0-only
//! Authored diagnostic face: its single open use is not a valid modelling region.
use super::*;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Face, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Direction2, Plane3, Point2, Point3, Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

pub(super) fn single_use_face() -> Shape {
    let builder = BRepBuilder::default();
    let mut face = planar_face();
    let edge = authored_edge();
    let pcurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(Point2::new(0., 0.), Direction2::X)).into_cast();
    builder
        .update_edge_pcurve_on_face(&edge, &pcurve, &face, 1e-7)
        .unwrap();
    builder.range_on_face(&edge, &face, 0., 1.).unwrap();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &edge).unwrap();
    builder.add(&mut face, &wire).unwrap();
    face.into_shape()
}
fn authored_edge() -> rustycad_tkbrep::topods::Edge {
    BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(1., 0., 0.))
        .unwrap()
        .edge()
        .unwrap()
}
fn planar_face() -> Face {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::from_pln(Plane3::default())).into_cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}
pub(super) fn translated(shape: &Shape) -> Shape {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(0., 0., 2.));
    shape.moved(&Location::new(transform), false).unwrap()
}
pub(super) fn imported(shape: &Shape, auto_product: bool) -> BRepGraph {
    let mut graph = BRepGraph::new();
    let options = Options {
        create_auto_product: auto_product,
        track_added_nodes: true,
        ..Options::default()
    };
    graph
        .shapes_mut()
        .add_with_options(shape, &options)
        .unwrap();
    graph
}
