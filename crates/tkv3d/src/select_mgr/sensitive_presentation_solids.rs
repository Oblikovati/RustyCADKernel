//! Native box edge order and incremental analytic circle/cylinder sampling.
use super::sensitive_presentation::SensitivePresentationGeometry;
use crate::select3d::{Select3DSensitiveBox, Select3DSensitiveCylinder};
use rustycad_tkernel::{precision::confusion, standard::Failure};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve};
use rustycad_tkmath::gp::{Axis2, Point3, Transform3, Xyz};
pub(super) fn append_sensitive_box(
    geometry: &mut SensitivePresentationGeometry,
    value: &Select3DSensitiveBox,
    location: Transform3,
) -> Result<(), Failure> {
    let [x0, y0, z0, x1, y1, z1] = value.bnd_box().get()?;
    let corners = [
        [x0, y0, z0],
        [x1, y0, z0],
        [x1, y1, z0],
        [x0, y1, z0],
        [x0, y0, z1],
        [x1, y0, z1],
        [x1, y1, z1],
        [x0, y1, z1],
    ];
    let points = corners.map(|[x, y, z]| Point3::new(x, y, z).transformed(location));
    geometry.append_line([0, 1, 2, 3, 0].map(|i| points[i]));
    geometry.append_line([4, 5, 6, 7, 4].map(|i| points[i]));
    for i in 0..4 {
        geometry.append_line([points[i], points[i + 4]]);
    }
    Ok(())
}
pub(super) fn append_sensitive_circle(
    geometry: &mut SensitivePresentationGeometry,
    radius: f64,
    location: Transform3,
    height: f64,
) -> Result<(), Failure> {
    let circle = GeomCircle::new(Axis2::default(), radius)?;
    let offset = Xyz::new(0., 0., height);
    let mut points = Vec::new();
    let mut parameter = 0.;
    while parameter < 2. * std::f64::consts::PI + 0.1 {
        points.push(Point3::from(circle.value(parameter)?.xyz() + offset).transformed(location));
        parameter += 0.1;
    }
    geometry.append_line(points);
    Ok(())
}
pub(super) fn append_sensitive_cylinder(
    geometry: &mut SensitivePresentationGeometry,
    cylinder: &Select3DSensitiveCylinder,
    location: Transform3,
) -> Result<(), Failure> {
    let location = location.multiplied(cylinder.transformation());
    let mut first = Vec::new();
    let mut second = Vec::new();
    for index in 0..3 {
        let radius = 0.5 * f64::from(2 - index) * cylinder.bottom_radius()
            + 0.5 * f64::from(index) * cylinder.top_radius();
        let height = cylinder.height() * 0.5 * f64::from(index);
        append_cylinder_verticals(&mut first, &mut second, index, radius, height, location);
        if radius > confusion() {
            append_sensitive_circle(geometry, radius, location, height)?;
        }
    }
    geometry.append_line(first);
    geometry.append_line(second);
    Ok(())
}
fn append_cylinder_verticals(
    first: &mut Vec<Point3>,
    second: &mut Vec<Point3>,
    index: i32,
    radius: f64,
    height: f64,
    location: Transform3,
) {
    if index == 1 {
        return;
    }
    first.push(Point3::new(radius, 0., height).transformed(location));
    second.push(Point3::new(-radius, 0., height).transformed(location));
}
