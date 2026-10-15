//! PrsDim relations: constructor state, colors, geometry helpers and computed attach points.
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkmath::gp::{self, Axis2, Axis3, Circle3, Line3, Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeVertex};
use rustycad_tkservice::graphic3d::Graphic3dAspectsMembers as _;
use rustycad_tkv3d::dsg_prs::DsgPrsArrowSide;
use rustycad_tkv3d::prs_dim::*;
use rustycad_tkv3d::prs_mgr::{PrsMgrPresentableObject, PrsMgrPresentableObjectMembers};
use std::f64::consts::PI;

fn xoy() -> Handle<GeomPlane> {
    Handle::new(GeomPlane::from_pln(Plane3::from_position(Axis3::from_axis2(gp::xoy()))))
}
fn segment(a: Point3, b: Point3) -> Shape {
    let edge: Edge = BRepBuilderAPIMakeEdge::from_points(a, b).unwrap().edge().unwrap();
    edge.into()
}
fn vertex(p: Point3) -> Shape {
    BRepBuilderAPIMakeVertex::new(p).unwrap().vertex().unwrap().into()
}

#[test]
fn relations_start_from_the_native_constructor_state() {
    let p = Point3::new;
    let parallel = PrsDimParallelRelation::new(
        &segment(p(0., 0., 0.), p(10., 0., 0.)),
        &segment(p(2., 5., 0.), p(12., 5., 0.)),
        &xoy(),
    );
    assert_eq!(parallel.arrow_size(), 0.01);
    assert_eq!(parallel.symbol_prs(), DsgPrsArrowSide::BothAr);
    assert!(parallel.is_movable());
    assert_eq!(parallel.kind_of_dimension(), PrsDimKindOfDimension::None);
    let offset = PrsDimOffsetDimension::new(&vertex(p(0., 0., 0.)), &vertex(p(1., 0., 0.)), 400.0, &ExtendedString::default());
    // |400 / 10| limited to [15, 30].
    assert_eq!(offset.arrow_size(), 30.0);
    assert_eq!(offset.kind_of_dimension(), PrsDimKindOfDimension::Offset);
    let fix = PrsDimFixRelation::new(&vertex(p(1., 2., 0.)), &xoy(), None);
    assert_eq!(fix.arrow_size(), 5.0);
    assert!(fix.automatic_position());
    let symmetric = PrsDimSymmetricRelation::new(
        &segment(p(0., -1., 0.), p(0., 1., 0.)),
        &vertex(p(1., 0., 0.)),
        &vertex(p(-1., 0., 0.)),
        &xoy(),
    )
    .unwrap();
    assert_eq!(symmetric.position(), gp::origin());
    let equal = PrsDimEqualDistanceRelation::new(
        &vertex(p(0., 0., 0.)),
        &vertex(p(1., 0., 0.)),
        &vertex(p(0., 1., 0.)),
        &vertex(p(1., 1., 0.)),
        &xoy(),
    );
    assert_eq!(equal.arrow_size(), 3.0);
}

#[test]
fn relation_colors_follow_the_drawer_aspects() {
    let p = Point3::new;
    let tangent = PrsDimTangentRelation::new(
        &segment(p(-5., 3., 0.), p(5., 3., 0.)),
        &segment(p(0., 0., 0.), p(1., 0., 0.)),
        &xoy(),
        2,
    );
    assert_eq!(tangent.extern_ref(), 2);
    let red = Color::named(NamedColor::RED);
    tangent.set_color(red).unwrap();
    assert!(tangent.has_color());
    let drawer = tangent.attributes();
    assert!(drawer.has_own_line_aspect());
    assert_eq!(drawer.line_aspect().aspect().interior_color(), red);
    tangent.unset_color().unwrap();
    assert!(!tangent.has_color());
}

#[test]
fn midpoint_marks_stay_on_the_curves() {
    let p = Point3::new;
    // The mid point projects on a segment, the marks toward its ends.
    let (attach, first, second) = points_on_segment(&p(2., -3., 0.), &p(2., 3., 0.), &gp::origin()).unwrap();
    assert_eq!(attach, p(2., 0., 0.));
    assert!(first.y() < 0.0 && second.y() > 0.0);
    // An infinite line marks a tenth of the distance around the projection.
    let line = Line3::new(p(2., 0., 0.), gp::dy());
    let (attach, first, second) = points_on_line(&line, &gp::origin()).unwrap();
    assert_eq!(attach, p(2., 0., 0.));
    assert!((first.y() - 0.2).abs() < 1e-12 && (second.y() + 0.2).abs() < 1e-12);
    // A whole circle marks a fifth of pi around the mid point parameter.
    let circle = Circle3::new(Axis2::new(gp::origin(), gp::dz()).unwrap(), 3.0).unwrap();
    let (attach, first, _) = points_on_conic(&circle, gp::origin(), &p(5., 0., 0.), &p(3., 0., 0.), &p(3., 0., 0.)).unwrap();
    assert!(attach.distance(&p(3., 0., 0.)) < 1e-12);
    let angle = first.y().atan2(first.x());
    assert!((angle + PI / 5.0).abs() < 1e-12);
}

#[test]
fn fix_relation_finds_the_connected_edges_of_a_wire() {
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeWire;
    let p = Point3::new;
    let as_vertex = |s: Shape| rustycad_tkbrep::topods::vertex(&s).unwrap();
    let (v0, v1, v2) = (as_vertex(vertex(p(0., 0., 0.))), as_vertex(vertex(p(4., 0., 0.))), as_vertex(vertex(p(4., 3., 0.))));
    let e1 = BRepBuilderAPIMakeEdge::from_vertices(&v0, &v1).unwrap().edge().unwrap();
    let e2 = BRepBuilderAPIMakeEdge::from_vertices(&v1, &v2).unwrap().edge().unwrap();
    let wire = BRepBuilderAPIMakeWire::from_two_edges(&e1, &e2).unwrap().wire().unwrap();
    assert!(PrsDimFixRelation::connected_edges(&wire, &v1).unwrap().is_some());
    // An end vertex has a single edge.
    assert!(PrsDimFixRelation::connected_edges(&wire, &v0).unwrap().is_none());
}
