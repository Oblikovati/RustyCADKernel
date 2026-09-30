// Rust translation of the steps shared by GeomFill_PolynomialConvertor.cxx and
// GeomFill_QuasiAngularConvertor.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The steps both arc convertors repeat: the monomial-to-Bezier basis matrix of
//! their `Init`, the planar frame `M` of the arc and the 1-based math vectors.
use super::sweep_failures::matrix_with_bounds;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::convert::CompPolynomialToPoles;
use rustycad_tkmath::gp::{Matrix3, Vector3, Xyz};
use rustycad_tkmath::math::{Matrix, Vector};

/// The Bezier poles of the monomials `1, t, ..., t^(ordre-1)` on `[-1, 1]`, one
/// column per monomial, with the terms within 1e-9 of +-1 snapped to +-1 (the
/// `Init` body of both convertors); `basis_matrix(7)?`.
pub(crate) fn basis_matrix(ordre: i32) -> Result<Matrix<'static>, Failure> {
    let mut coeffs: Array1<'static, f64> = one_based_reals(ordre * ordre)?;
    coeffs.init(0.0);
    for ii in 1..=ordre {
        coeffs[ii + (ii - 1) * ordre] = 1.0;
    }
    let mut inter: Array1<'static, f64> = one_based_reals(2)?;
    inter[1] = -1.0;
    inter[2] = 1.0;
    let converter = CompPolynomialToPoles::from_single_span(
        ordre,
        ordre - 1,
        ordre - 1,
        &coeffs,
        &inter,
        &inter,
    )?;
    let poles = converter.poles()?;
    let mut result = matrix_with_bounds(1, ordre, 1, ordre)?;
    for jj in 1..=ordre {
        for ii in 1..=ordre {
            result[(ii, jj)] = snapped(poles[(ii, jj)]);
        }
    }
    Ok(result)
}

// A term within 1e-9 of 1 (or -1) becomes exactly 1 (or -1).
fn snapped(term: f64) -> f64 {
    let mut term = term;
    if (term - 1.0).abs() < 1.0e-9 {
        term = 1.0;
    }
    if (term + 1.0).abs() < 1.0e-9 {
        term = -1.0;
    }
    term
}

fn one_based_reals(count: i32) -> Result<Array1<'static, f64>, Failure> {
    rustycad_tkg2d::geom2d::arrays::one_based(count)
}

/// The planar frame of an arc, `gp_Mat(V1.X(), V2.X(), 0, V1.Y(), V2.Y(), 0,
/// V1.Z(), V2.Z(), 0)`; `arc_frame(v1, v2)`.
pub(crate) fn arc_frame(v1: Vector3, v2: Vector3) -> Matrix3 {
    Matrix3::new([
        [v1.x(), v2.x(), 0.0],
        [v1.y(), v2.y(), 0.0],
        [v1.z(), v2.z(), 0.0],
    ])
}

/// A 1-based `math_Vector` holding `values`; `math_vector(&[1.0, 2.0])?`.
pub(crate) fn math_vector(values: &[f64]) -> Result<Vector<'static>, Failure> {
    let mut vector = Vector::with_bounds(1, values.len() as i32)?;
    for (index, value) in values.iter().enumerate() {
        vector[index as i32 + 1] = *value;
    }
    Ok(vector)
}

/// `gp_XYZ(Px(ii), Py(ii), 0)` from the products `x` and `x + 1` of `products`.
pub(crate) fn pole_xyz(products: &[Vector<'static>], x: usize, ii: i32) -> Xyz {
    Xyz::new(products[x][ii], products[x + 1][ii], 0.0)
}
