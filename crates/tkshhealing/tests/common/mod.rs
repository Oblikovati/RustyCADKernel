use rustycad_tkbrep::topods::{Shape, Wire};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeWire};

pub fn face_count(shape: &Shape) -> usize {
    rustycad_tkbrep::topexp::Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .count()
}

// This shared fixture is unused by some integration-test binaries.
#[allow(dead_code)]
pub fn rectangle_wire(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Wire {
    let points = [
        Point3::new(x_min, y_min, 0.0),
        Point3::new(x_max, y_min, 0.0),
        Point3::new(x_max, y_max, 0.0),
        Point3::new(x_min, y_max, 0.0),
    ];
    let mut wire = BRepBuilderAPIMakeWire::new();
    for index in 0..4 {
        let edge = BRepBuilderAPIMakeEdge::from_points(points[index], points[(index + 1) % 4])
            .unwrap()
            .edge()
            .unwrap();
        wire.add_edge(&edge).unwrap();
    }
    wire.wire().unwrap()
}
