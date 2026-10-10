//! Shared native pipe-shell fixture construction.
use rustycad_tkbrep::topods::Wire;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;

/// `MakeSquareProfile(theCenter, theHalfSide)`.
pub fn make_square_profile(the_center: Point3, the_half_side: f64) -> Result<Wire, Failure> {
    let mut a_polygon = BRepBuilderAPIMakePolygon::new();
    let (x, y, z) = (the_center.x(), the_center.y(), the_center.z());
    for (dx, dy) in [(-1., -1.), (1., -1.), (1., 1.), (-1., 1.)] {
        a_polygon.add_point(Point3::new(
            x + dx * the_half_side,
            y + dy * the_half_side,
            z,
        ))?;
    }
    a_polygon.close()?;
    a_polygon.wire()
}

/// The polyline spine through the points.
pub fn polyline(points: &[(f64, f64, f64)]) -> Result<Wire, Failure> {
    let mut a_spine_polygon = BRepBuilderAPIMakePolygon::new();
    for &(x, y, z) in points {
        a_spine_polygon.add_point(Point3::new(x, y, z))?;
    }
    a_spine_polygon.wire()
}
