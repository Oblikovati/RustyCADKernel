//! Pinned native ShapeAnalysis package rows, separate from legacy declarations.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp,
    topods::{self, Edge, Face, Shape, Wire},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis3, Plane3, Point3};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysis;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon,
};
fn assert_native(name: &str, actual: &[f64]) {
    let expected =
        include_str!("../../../tools/oracle/tkshhealing_shape_analysis_package.expected");
    let row = expected
        .lines()
        .find(|s| s.starts_with(&format!("{name} ")))
        .unwrap();
    let expected: Vec<f64> = row
        .split_whitespace()
        .skip(1)
        .map(|s| s.parse().unwrap())
        .collect();
    assert_eq!(actual, expected, "{name}");
}
fn rectangle_face() -> Face {
    BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0., 2., 0., 3.))
        .unwrap()
        .face()
        .unwrap()
}
fn natural_face(cylinder: bool) -> Face {
    let surface: Handle<dyn GeomSurface> = if cylinder {
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.).unwrap()).cast()
    } else {
        Handle::new(GeomPlane::new(Axis3::default())).cast()
    };
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}
fn assert_face(name: &str, face: &Face) {
    let Ok((uf, ul, vf, vl)) = ShapeAnalysis::get_face_uv_bounds(face) else {
        assert_native(name, &[0.]);
        return;
    };
    let outer = ShapeAnalysis::is_outer_bound(face).unwrap();
    assert_native(name, &[1., uf, ul, vf, vl, f64::from(outer)]);
}
fn polygon(points: &[Point3], closed: bool) -> Wire {
    let mut maker = BRepBuilderAPIMakePolygon::new();
    for &point in points {
        maker.add_point(point).unwrap();
    }
    if closed {
        maker.close().unwrap();
    }
    maker.wire().unwrap()
}
fn rectangle_wire(closed: bool) -> Wire {
    polygon(
        &[
            Point3::new(0., 0., 0.),
            Point3::new(2., 0., 0.),
            Point3::new(2., 3., 0.),
            Point3::new(0., 3., 0.),
        ],
        closed,
    )
}
fn assert_endpoints(name: &str, shape: &Shape) {
    let (first, last) = ShapeAnalysis::find_bounds(shape).unwrap();
    let mut actual = vec![f64::from(!first.is_null()), f64::from(!last.is_null())];
    if !first.is_null() {
        let a = BRepTool::pnt(&first).unwrap();
        let b = BRepTool::pnt(&last).unwrap();
        actual.extend([a.x(), a.y(), a.z(), b.x(), b.y(), b.z()]);
    }
    assert_native(name, &actual);
}
#[test]
fn package_face_bounds_and_single_wire_orientation_match_native() {
    assert_face("natural-plane", &natural_face(false));
    let rectangle = rectangle_face();
    assert_face("rectangle", &rectangle);
    assert_face(
        "reversed-face",
        &topods::face(&rectangle.reversed()).unwrap(),
    );
    let mut inner = natural_face(false);
    let wire = ShapeAnalysis::outer_wire(&rectangle).unwrap();
    BRepBuilder::default()
        .add(&mut inner, &wire.reversed())
        .unwrap();
    assert_face("inner-only", &inner);
}
#[test]
fn package_multiwire_classification_and_missing_pcurve_match_native() {
    let mut face = rectangle_face();
    let hole = polygon(
        &[
            Point3::new(0.5, 0.5, 0.),
            Point3::new(0.5, 1., 0.),
            Point3::new(1., 1., 0.),
            Point3::new(1., 0.5, 0.),
        ],
        true,
    );
    BRepBuilder::default().add(&mut face, &hole).unwrap();
    assert_face("with-hole", &face);
    let mut face = natural_face(true);
    let (mut edge, mut wire) = (Edge::default(), Wire::default());
    let builder = BRepBuilder::default();
    builder.make_edge(&mut edge).unwrap();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &edge).unwrap();
    builder.add(&mut face, &wire).unwrap();
    assert_face("missing-pcurve", &face);
}
#[test]
fn package_spatial_area_and_wire_endpoints_match_native() {
    for closed in [false, true] {
        let wire = rectangle_wire(closed);
        let reversed = topods::wire(&wire.reversed()).unwrap();
        let name = if closed { "closed" } else { "open" };
        assert_native(
            &format!("area-{name}"),
            &[
                ShapeAnalysis::contour_area(&wire).unwrap(),
                ShapeAnalysis::contour_area(&reversed).unwrap(),
            ],
        );
        assert_endpoints(&format!("ends-{name}"), &wire);
        assert_endpoints(&format!("ends-{name}-reversed"), &reversed);
    }
}
#[test]
fn package_empty_contours_and_vertex_edge_endpoints_match_native() {
    let mut wire = Wire::default();
    BRepBuilder::default().make_wire(&mut wire);
    assert_native("area-empty", &[ShapeAnalysis::contour_area(&wire).unwrap()]);
    assert_endpoints("ends-empty", &wire);
    assert_endpoints("ends-face", &natural_face(false));
    let edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(1., 2., 3.), Point3::new(4., 5., 6.))
            .unwrap()
            .edge()
            .unwrap();
    assert_endpoints("ends-edge", &edge);
    assert_endpoints("ends-edge-reversed", &edge.reversed());
    assert_endpoints("ends-vertex", &topexp::first_vertex(&edge, false));
}
