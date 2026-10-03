//! Shared native-shaped fixtures for face orientation and natural-bound tests.
#![allow(dead_code)] // Each integration-test binary uses a different subset.

use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::edge;
use rustycad_tkbrep::topods::{Builder, Face, ShapeIterator, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis3, Direction3, Point2, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon};

pub(super) struct WireFixtureSpec {
    pub(super) reversed: bool,
    pub(super) bounds: (f64, f64, f64, f64),
}

pub(super) fn square(spec: &WireFixtureSpec) -> rustycad_tkbrep::topods::Wire {
    let (x0, y0, x1, y1) = spec.bounds;
    let points = if spec.reversed {
        [
            Point3::new(x0, y0, 0.0),
            Point3::new(x0, y1, 0.0),
            Point3::new(x1, y1, 0.0),
            Point3::new(x1, y0, 0.0),
        ]
    } else {
        [
            Point3::new(x0, y0, 0.0),
            Point3::new(x1, y0, 0.0),
            Point3::new(x1, y1, 0.0),
            Point3::new(x0, y1, 0.0),
        ]
    };
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for point in points {
        polygon.add_point(point).unwrap();
    }
    polygon.close().unwrap();
    polygon.wire().unwrap()
}

pub(super) fn face_with_wires(specs: &[WireFixtureSpec]) -> Face {
    let plane = plane_surface();
    let mut face = BRepBuilderAPIMakeFace::from_surface(&plane, 1e-7)
        .unwrap()
        .face()
        .unwrap();
    for spec in specs {
        Builder.add(&mut face, &square(spec).into_shape()).unwrap();
    }
    face
}

pub(super) fn plane_surface() -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::new(
        Axis3::new(
            Point3::default(),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
            Direction3::new(1.0, 0.0, 0.0).unwrap(),
        )
        .unwrap(),
    ))
    .cast::<dyn GeomSurface>()
}

pub(super) fn count_wires(face: &Face) -> usize {
    ShapeIterator::new(&face.clone().into_shape(), false, true)
        .filter(|child| child.shape_type().ok() == Some(ShapeEnum::Wire))
        .count()
}

pub(super) fn wire_orientations(face: &Face) -> Vec<Orientation> {
    ShapeIterator::new(&face.clone().into_shape(), false, true)
        .filter_map(|child| {
            (child.shape_type().ok() == Some(ShapeEnum::Wire)).then_some(child.orientation())
        })
        .collect()
}

pub(super) fn assert_uv_close(point: Point2, expected: (f64, f64)) {
    assert!(
        (point.x() - expected.0).abs() < 1e-7,
        "unexpected U: {}",
        point.x()
    );
    assert!(
        (point.y() - expected.1).abs() < 1e-7,
        "unexpected V: {}",
        point.y()
    );
}

pub(super) fn assert_pcurves_rebuild_the_same_3d_edges(face: &Face, wires: &[Wire]) {
    let (surface, _) = BRepTool::surface_with_location(face).unwrap();
    let surface = surface.get().unwrap();
    for wire in wires {
        for edge_shape in ShapeIterator::new(&wire.clone().into_shape(), false, true) {
            let edge = edge(&edge_shape).unwrap();
            let pcurve = BRepTool::curve_on_face(&edge, face).unwrap();
            let pcurve_mid = pcurve
                .pcurve
                .value((pcurve.first + pcurve.last) * 0.5)
                .unwrap();
            let expected = surface.value(pcurve_mid.x(), pcurve_mid.y()).unwrap();
            let (curve, first, last) = BRepTool::curve(&edge).unwrap();
            let actual = curve.value((first + last) * 0.5).unwrap();
            assert!(
                actual.distance(&expected) < 1e-5,
                "edge {:?} gap {}",
                edge.orientation(),
                actual.distance(&expected)
            );
        }
    }
}

pub(super) fn cylinder_surface() -> Handle<dyn GeomSurface> {
    Handle::new(
        GeomCylindricalSurface::new(
            Axis3::new(
                Point3::default(),
                Direction3::new(0.0, 0.0, 1.0).unwrap(),
                Direction3::new(1.0, 0.0, 0.0).unwrap(),
            )
            .unwrap(),
            1.0,
        )
        .unwrap(),
    )
    .cast::<dyn GeomSurface>()
}

pub(super) fn wire_shapes(face: &Face) -> Vec<Wire> {
    ShapeIterator::new(&face.clone().into_shape(), false, true)
        .filter_map(|child| {
            (child.shape_type().ok() == Some(ShapeEnum::Wire))
                .then(|| rustycad_tkbrep::topods::wire(&child).unwrap())
        })
        .collect()
}

pub(super) fn patch_wire(surface: &Handle<dyn GeomSurface>, bounds: (f64, f64, f64, f64)) -> Wire {
    let face = BRepBuilderAPIMakeFace::from_surface_bounds(surface, bounds, 1e-7)
        .unwrap()
        .face()
        .unwrap();
    let wire_shape = ShapeIterator::new(&face.clone().into_shape(), false, true)
        .find(|child| child.shape_type().ok() == Some(ShapeEnum::Wire))
        .unwrap();
    rustycad_tkbrep::topods::wire(&wire_shape).unwrap()
}

pub(super) fn first_pcurve_midpoint(face: &Face, wire: &Wire) -> Point2 {
    let edges = ShapeIterator::new(&wire.clone().into_shape(), false, true);
    assert!(edges.more());
    let first = edges.value().unwrap();
    let pcurve =
        BRepTool::curve_on_face(&rustycad_tkbrep::topods::edge(first).unwrap(), face).unwrap();
    pcurve
        .pcurve
        .value((pcurve.first + pcurve.last) * 0.5)
        .unwrap()
}
