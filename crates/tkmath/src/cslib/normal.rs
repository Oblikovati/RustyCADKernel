//! The first-, second- and magnitude-based CSLib::Normal overloads.
use super::{DerivativeStatus, NormalResult, NormalStatus};
use crate::gp::{Direction3, Vector3};
use rustycad_tkernel::standard::Failure;

/// gp::Resolution(), i.e. RealSmall().
const RESOLUTION: f64 = f64::MIN_POSITIVE;
/// RealEpsilon().
const REAL_EPSILON: f64 = f64::EPSILON;

/// CSLib::Normal(D1U, D1V, SinTol, Status, Normal): the normal as `D1U ^ D1V`.
///
/// Parallelism compares the squared sine of the angle with `sin_tol^2`. A
/// non-positive or NaN `sin_tol` can reach `gp_Dir` with a null cross product,
/// which fails as natively.
/// `normal(Vector3::new(1.0,0.0,0.0), Vector3::new(0.0,1.0,0.0), 1e-10)?.status == DerivativeStatus::Done`.
pub fn normal(
    d1u: Vector3,
    d1v: Vector3,
    sin_tol: f64,
) -> Result<NormalResult<DerivativeStatus>, Failure> {
    let d1u_mag: f64 = d1u.square_magnitude();
    let d1v_mag: f64 = d1v.square_magnitude();
    let d1u_x_d1v: Vector3 = d1u.crossed(d1v);
    if d1u_mag <= RESOLUTION && d1v_mag <= RESOLUTION {
        return Ok(NormalResult::untouched(DerivativeStatus::D1IsNull));
    }
    if d1u_mag <= RESOLUTION {
        return Ok(NormalResult::untouched(DerivativeStatus::D1uIsNull));
    }
    if d1v_mag <= RESOLUTION {
        return Ok(NormalResult::untouched(DerivativeStatus::D1vIsNull));
    }
    let sin2: f64 = d1u_x_d1v.square_magnitude() / (d1u_mag * d1v_mag);
    if sin2 < sin_tol * sin_tol {
        return Ok(NormalResult::untouched(DerivativeStatus::D1uIsParallelD1v));
    }
    let direction: Direction3 = Direction3::from_vector(d1u_x_d1v)?;
    Ok(NormalResult::written(DerivativeStatus::Done, direction))
}

/// CSLib::Normal(D1U, D1V, D2U, D2V, D2UV, SinTol, Done, Status, Normal).
///
/// Approximates the normal at a singular point from the first derivatives of
/// `N = D1U ^ D1V`; `is_done()` is the native `theDone` flag.
/// `normal_from_second_derivatives(d1u, d1v, d2u, d2v, d2uv, 1e-6).status`.
pub fn normal_from_second_derivatives(
    d1u: Vector3,
    d1v: Vector3,
    d2u: Vector3,
    d2v: Vector3,
    d2uv: Vector3,
    sin_tol: f64,
) -> NormalResult<NormalStatus> {
    // dN/du = D2U ^ D1V + D1U ^ D2UV and dN/dv = D2UV ^ D1V + D1U ^ D2V.
    let d1nu: Vector3 = d2u.crossed(d1v) + d1u.crossed(d2uv);
    let d1nv: Vector3 = d2uv.crossed(d1v) + d1u.crossed(d2v);
    classify_normal_derivatives(d1nu, d1nv, sin_tol)
}

fn classify_normal_derivatives(
    d1nu: Vector3,
    d1nv: Vector3,
    sin_tol: f64,
) -> NormalResult<NormalStatus> {
    let ld1nu: f64 = d1nu.square_magnitude();
    let ld1nv: f64 = d1nv.square_magnitude();
    if ld1nu <= REAL_EPSILON && ld1nv <= REAL_EPSILON {
        return NormalResult::untouched(NormalStatus::D1NIsNull);
    }
    // Both vectors below cannot be null: the branch above caught that case.
    if ld1nu < REAL_EPSILON {
        return written_from(NormalStatus::D1NuIsNull, d1nv);
    }
    if ld1nv < REAL_EPSILON {
        return written_from(NormalStatus::D1NvIsNull, d1nu);
    }
    if (ld1nv / ld1nu) <= REAL_EPSILON {
        return NormalResult::untouched(NormalStatus::D1NvNuRatioIsNull);
    }
    if (ld1nu / ld1nv) <= REAL_EPSILON {
        return NormalResult::untouched(NormalStatus::D1NuNvRatioIsNull);
    }
    let sin2: f64 = d1nu.crossed(d1nv).square_magnitude() / (ld1nu * ld1nv);
    if sin2 < sin_tol * sin_tol {
        return written_from(NormalStatus::D1NuIsParallelD1Nv, d1nu);
    }
    NormalResult::untouched(NormalStatus::InfinityOfSolutions)
}

// The vectors reaching here have a squared norm of at least RealEpsilon (or are
// non-finite), so gp_Dir's zero-norm check cannot fire.
fn written_from(status: NormalStatus, vector: Vector3) -> NormalResult<NormalStatus> {
    match Direction3::from_vector(vector) {
        Ok(direction) => NormalResult::written(status, direction),
        Err(_) => unreachable!("vector {vector:?} has squared norm >= RealEpsilon"),
    }
}

/// CSLib::Normal(D1U, D1V, MagTol, Status, Normal) with a magnitude tolerance.
///
/// Normalizes both tangents before crossing them, as OCCT does. A negative
/// `mag_tol` can pass a null tangent to `gp_Dir`, which fails as natively.
/// `normal_with_magnitude(d1u, d1v, 1e-10)?.status == NormalStatus::Defined`.
pub fn normal_with_magnitude(
    d1u: Vector3,
    d1v: Vector3,
    mag_tol: f64,
) -> Result<NormalResult<NormalStatus>, Failure> {
    let d1u_mag: f64 = d1u.magnitude();
    let d1v_mag: f64 = d1v.magnitude();
    let n_mag: f64 = d1u.crossed(d1v).magnitude();
    if n_mag <= mag_tol || d1u_mag <= mag_tol || d1v_mag <= mag_tol {
        return Ok(NormalResult::untouched(NormalStatus::Singular));
    }
    let d1u_dir: Direction3 = Direction3::from_vector(d1u)?;
    let d1v_dir: Direction3 = Direction3::from_vector(d1v)?;
    let direction: Direction3 = d1u_dir.crossed(d1v_dir)?;
    Ok(NormalResult::written(NormalStatus::Defined, direction))
}
