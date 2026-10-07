//! One-based NCollection rows shared by the GeomFill filling targets.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::gp::Point3;

/// One-based pole row; `row(&[(0., 0., 0.)])[1]` is the origin.
pub fn row(points: &[(f64, f64, f64)]) -> Array1<'static, Point3> {
    let mut row = Array1::with_bounds(1, points.len() as i32).unwrap();
    for (k, &(x, y, z)) in points.iter().enumerate() {
        row[k as i32 + 1] = Point3::new(x, y, z);
    }
    row
}

/// One-based real row; `reals(&[1., 2.])[2] == 2.`.
pub fn reals(values: &[f64]) -> Array1<'static, f64> {
    let mut row = Array1::with_bounds(1, values.len() as i32).unwrap();
    row.as_mut_slice().copy_from_slice(values);
    row
}
