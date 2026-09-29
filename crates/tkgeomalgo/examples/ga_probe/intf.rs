//! Intf probe operations, the Rust twin of tools/oracle/tkgeomalgo_probe_intf.cxx.
use super::common::{Reader, end, flag, real, xyz};
use rustycad_tkgeomalgo::intf::{contain, plane_equation};

/// `p1 p2 p3 point`: the plane equation of the triangle, then whether it contains the point.
pub fn triangle_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (p1, p2, p3, point) = (r.pnt(), r.pnt(), r.pnt(), r.pnt());
    let (normal, distance) = plane_equation(p1, p2, p3);
    xyz(normal);
    real(distance);
    flag(contain(p1, p2, p3, point));
    end();
}
