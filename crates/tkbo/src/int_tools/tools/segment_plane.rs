use super::Tools;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Line3, Plane3, Point3};
impl Tools {
    /// Intersects using the native signed-distance early exits; `Tools::seg_pln(&line, -1., 1., 1e-7, &plane, 1e-7, &mut p, &mut t, &mut tol, &mut lo, &mut hi)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn seg_pln(
        line: &Line3,
        first: f64,
        last: f64,
        line_tol: f64,
        plane: &Plane3,
        plane_tol: f64,
        point: &mut Point3,
        parameter: &mut f64,
        tolerance: &mut f64,
        minimum: &mut f64,
        maximum: &mut f64,
    ) -> Result<i32, Failure> {
        let sum = line_tol + plane_tol;
        let a = signed_distance(plane, line_point(line, first));
        let b = signed_distance(plane, line_point(line, last));
        if a < sum && b < sum {
            return Ok(1);
        }
        if a * b > 0. {
            return Ok(2);
        }
        let t = plane_parameter(line, plane)?;
        if t < first - sum || t > last + sum {
            return Ok(3);
        }
        *parameter = t;
        *point = line_point(line, t);
        *tolerance = sum;
        *minimum = t - plane_tol;
        *maximum = t + plane_tol;
        Ok(0)
    }
}
fn line_point(line: &Line3, parameter: f64) -> Point3 {
    let origin = line.location();
    let direction = line.direction();
    Point3::new(
        origin.x() + parameter * direction.x(),
        origin.y() + parameter * direction.y(),
        origin.z() + parameter * direction.z(),
    )
}
fn signed_distance(plane: &Plane3, point: Point3) -> f64 {
    let origin = plane.location();
    let direction = plane.position().direction();
    direction.x() * (point.x() - origin.x())
        + direction.y() * (point.y() - origin.y())
        + direction.z() * (point.z() - origin.z())
}
fn plane_parameter(line: &Line3, plane: &Plane3) -> Result<f64, Failure> {
    let [a, b, c, d] = plane.coefficients()?;
    let p = line.location();
    let n = line.direction();
    let e = a * p.x() + b * p.y() + c * p.z() + d;
    let h = a * n.x() + b * n.y() + c * n.z();
    Ok(-e / h)
}
