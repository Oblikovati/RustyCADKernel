use super::Tools;
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::{GeomBoundedCurve, GeomCurve};
use rustycad_tkmath::gp::{Direction3, Point3};
impl Tools {
    /// Tests either parallel direction with native default threshold; `Tools::is_dirs_coinside(d1, d2)`.
    pub fn is_dirs_coinside(first: Direction3, last: Direction3) -> bool {
        Self::is_dirs_coinside_tolerance(first, last, 0.0002)
    }
    /// Uses chord distance between unit directions, accepting opposites; `Tools::is_dirs_coinside_tolerance(d1, d2, 1e-4)`.
    pub fn is_dirs_coinside_tolerance(first: Direction3, last: Direction3, tolerance: f64) -> bool {
        let a = Point3::new(first.x(), first.y(), first.z());
        let b = Point3::new(last.x(), last.y(), last.z());
        let distance = a.distance(&b);
        distance < tolerance || (2. - distance).abs() < tolerance
    }
    /// Only bounded curves qualify, using Confusion squared; `Tools::is_closed(&curve)?`.
    pub fn is_closed(curve: &Handle<dyn GeomCurve>) -> Result<bool, Failure> {
        if curve.cast::<dyn GeomBoundedCurve>().is_null() {
            return Ok(false);
        }
        let first = curve.eval_d0(curve.first_parameter())?;
        let last = curve.eval_d0(curve.last_parameter())?;
        let tolerance = precision::confusion();
        Ok(first.square_distance(&last) < tolerance * tolerance)
    }
}
