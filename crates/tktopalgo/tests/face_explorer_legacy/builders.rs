use super::*;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomCylindricalSurface, GeomSurface};
use rustycad_tkmath::gp::{Axis2, Axis3, Plane3, Point3, dz, origin};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};

pub(super) fn rectangle(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> Wire {
    let points = [
        Point3::new(xmin, ymin, 0.0),
        Point3::new(xmax, ymin, 0.0),
        Point3::new(xmax, ymax, 0.0),
        Point3::new(xmin, ymax, 0.0),
    ];
    let mut builder = BRepBuilderAPIMakeWire::new();
    for index in 0..4 {
        let edge =
            BRepBuilderAPIMakeEdge::from_points(points[index], points[(index + 1) % 4]).unwrap();
        assert!(edge.is_done(), "Edge creation failed");
        builder.add_edge(&edge.edge().unwrap()).unwrap();
    }
    assert!(builder.is_done(), "Wire creation failed");
    builder.wire().unwrap()
}

pub(super) fn wire_face(wire: &Wire) -> Face {
    let maker = BRepBuilderAPIMakeFace::from_wire(wire, true).unwrap();
    assert!(maker.is_done());
    maker.face().unwrap()
}

pub(super) fn square_face() -> Face {
    let wire = rectangle(0.0, 0.0, 2.0, 2.0);
    assert!(!wire.is_null());
    wire_face(&wire)
}

pub(super) fn holed_face() -> Face {
    let inner = rectangle(3.0, 3.0, 7.0, 7.0);
    assert!(!inner.is_null());
    let mut maker =
        BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0.0, 10.0, 0.0, 10.0))
            .unwrap();
    assert!(maker.is_done());
    maker
        .add(&topods::wire(&inner.reversed()).unwrap())
        .unwrap();
    assert!(maker.is_done());
    maker.face().unwrap()
}

pub(super) fn cylinder_face() -> Face {
    let frame = Axis3::from_normal(origin(), dz()).unwrap();
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(frame, 2.0).unwrap()).cast();
    let maker = BRepBuilderAPIMakeFace::from_surface_bounds(
        &surface,
        (0.0, 2.0 * PI, 0.0, 5.0),
        precision::confusion(),
    )
    .unwrap();
    assert!(maker.is_done());
    maker.face().unwrap()
}

pub(super) fn disc_face() -> Face {
    let frame = Axis2::new(origin(), dz()).unwrap();
    let circle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::new(frame, 2.0).unwrap()).cast();
    let edge = BRepBuilderAPIMakeEdge::from_curve(&circle).unwrap();
    assert!(edge.is_done());
    let wire = BRepBuilderAPIMakeWire::from_edge(&edge.edge().unwrap()).unwrap();
    assert!(wire.is_done());
    wire_face(&wire.wire().unwrap())
}
