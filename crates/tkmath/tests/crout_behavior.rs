//! Lower-triangle and aliasing contracts verified against unchanged math_Crout.
use rustycad_tkmath::math::{Crout, Matrix, Vector};
#[test]
fn rhs_aliasing_preserves_native_intermediate_writes() {
    let mut matrix = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    matrix[(1, 1)] = 2.0;
    matrix[(2, 1)] = 1.0;
    matrix[(2, 2)] = 2.0;
    let solver = Crout::new(&matrix).unwrap();
    let mut aliased = Vector::filled(1, 2, 3.0).unwrap();
    let mut separate = Vector::with_bounds(1, 2).unwrap();
    solver.solve_into(&aliased, &mut separate).unwrap();
    solver.solve_in_place(&mut aliased).unwrap();
    assert_eq!(separate[1], 1.0);
    assert_eq!(separate[2], 1.0);
    assert_eq!(aliased[1], 1.0);
    assert!((aliased[2] + 5.0 / 9.0).abs() < 1e-15);
}
#[test]
fn crout_inverse_uses_lower_triangle_and_not_done_guard() {
    let matrix = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    let failed = Crout::new(&matrix).unwrap();
    assert_eq!(
        failed.determinant().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert!(failed.inverse().is_err());
    assert!(failed.invert(&mut matrix.clone()).is_err());
    let mut diagonal = matrix.clone();
    diagonal.set_diag(2.0).unwrap();
    let solver = Crout::new(&diagonal).unwrap();
    let mut target = Matrix::filled(3, 4, 5, 6, 9.0).unwrap();
    solver.invert(&mut target).unwrap();
    assert_eq!(target[(3, 5)], 0.5);
    assert_eq!(target[(4, 5)], 0.0);
    assert_eq!(target[(4, 6)], 0.5);
}
