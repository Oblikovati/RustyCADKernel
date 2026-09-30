//! The inputs shared by the AppDef behaviour tests (`app_def_behavior` and the
//! single-test `app_def_reference_cache` binary).
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::app_def::MultiLine;
use rustycad_tkgeombase::app_par_curves::{Constraint, ConstraintCouple, Constraints};
use rustycad_tkmath::gp::Point3;

/// The exception class of a result, "none" on success.
pub fn kind<T>(result: Result<T, Failure>) -> &'static str {
    result.err().map_or("none", |error| error.exception_type())
}

/// `n` 3D points on a parabolic arc (smooth, not straight).
pub fn arc_line(n: i32) -> MultiLine {
    let points: Vec<Point3> = (0..n)
        .map(|i| {
            let t: f64 = f64::from(i) / f64::from(n - 1);
            Point3::new(t, t * t, 0.1 * t * t * t)
        })
        .collect();
    MultiLine::from_points(&Array1::from_shared(&points, 1))
}

/// The constraint couples `(index, constraint)`.
pub fn constraints(pairs: &[(i32, Constraint)]) -> Constraints {
    let couples: Vec<ConstraintCouple> = pairs
        .iter()
        .map(|&(index, c)| ConstraintCouple::new(index, c))
        .collect();
    Handle::new(HArray1::from_array(&Array1::from_shared(&couples, 1)))
}
