//! The linear systems of the blend functions: the derivatives of the solution along the
//! guide, `DEDX * dX/dt = -DEDT`, solved by Gauss and, when that fails, by SVD.
use super::Tensor;
use super::linalg::{matrix, vector, vget};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Gauss, Matrix, Svd, Vector};

/// The system of `IsSolution`: the Jacobian, the derivative along the guide, the
/// parametric tolerances of the residual check and the Gauss pivot.
pub(crate) struct TangentSolve<'a> {
    pub dedx: &'a Matrix<'static>,
    pub dedt: &'a Vector<'static>,
    pub tolerances: &'a Vector<'static>,
    pub max_pivot: f64,
}

/// Whether `DEDT + DEDX * solution` is within the tolerances (`controle`).
fn residual_ok(solve: &TangentSolve<'_>, solution: &Vector<'_>) -> Result<bool, Failure> {
    let controle = solve.dedt.added(&solve.dedx.multiplied_vector(solution)?)?;
    for i in 1..=4 {
        if vget(&controle, i)?.abs() > vget(solve.tolerances, i)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn values(solution: &Vector<'_>) -> Result<[f64; 4], Failure> {
    Ok([
        vget(solution, 1)?,
        vget(solution, 2)?,
        vget(solution, 3)?,
        vget(solution, 4)?,
    ])
}

/// The `IsSolution` tangent computation of BlendFunc_ConstRad: Gauss, then SVD (cutoff
/// 1e-6) when Gauss fails or its residual is out of tolerance; `None` for a tangency
/// point. Both solve into the same vector, as natively.
pub(crate) fn solve_tangents(solve: &TangentSolve<'_>) -> Result<Option<[f64; 4]>, Failure> {
    let mut solution = vector(4)?;
    let rhs = solve.dedt.opposite();
    let gauss = Gauss::with_min_pivot(solve.dedx, solve.max_pivot)?;
    if gauss.is_done() {
        gauss.solve_into(&rhs, &mut solution)?;
        if residual_ok(solve, &solution)? {
            return values(&solution).map(Some);
        }
    }
    let mut svd = Svd::new(solve.dedx)?;
    if !svd.is_done() {
        return Ok(None);
    }
    svd.solve(&rhs, &mut solution, 1.0e-6)?;
    if residual_ok(solve, &solution)? {
        return values(&solution).map(Some);
    }
    Ok(None)
}

/// The equations of a two-surface blend function and their derivatives that
/// `ComputeValues` stores (`E`, `DEDX`, `DEDT`, `D2EDX2`, `D2EDXDT`, `D2EDT2`).
/// The `IsSolution` tangent computation of BlendFunc_EvolRad with its successful Gauss
/// decomposition: `None` when the residual is out of tolerance.
pub(crate) fn solve_tangents_gauss(
    gauss: &Gauss,
    solve: &TangentSolve<'_>,
) -> Result<Option<[f64; 4]>, Failure> {
    let mut solution = vector(4)?;
    gauss.solve_into(&solve.dedt.opposite(), &mut solution)?;
    if residual_ok(solve, &solution)? {
        return values(&solution).map(Some);
    }
    Ok(None)
}

/// The first and second derivatives of the section variables along the guide.
pub(crate) type SecondDerivatives = ([f64; 4], [f64; 4]);

/// The equations of the blend function (see below).
pub(crate) struct Equations {
    pub e: Vector<'static>,
    pub dedx: Matrix<'static>,
    pub dedt: Vector<'static>,
    pub d2edx2: Tensor,
    pub d2edxdt: Matrix<'static>,
    pub d2edt2: Vector<'static>,
}

impl Equations {
    /// The 4-variable system, zero-filled (natively uninitialized).
    pub(crate) fn new() -> Result<Self, Failure> {
        Ok(Self {
            e: vector(4)?,
            dedx: matrix(4, 4)?,
            dedt: vector(4)?,
            d2edx2: Tensor::new(4, 4, 4),
            d2edxdt: matrix(4, 4)?,
            d2edt2: vector(4)?,
        })
    }
}

/// The first derivatives of the section along the guide (`Section` with `DPoles`): Gauss
/// with pivot 1e-9 unless the two points are within 1e-4 (`near`), then SVD with cutoff
/// 1e-6; `None` for a tangency point.
pub(crate) fn solve_derivatives(
    dedx: &Matrix<'static>,
    dedt: &Vector<'static>,
    near: bool,
) -> Result<Option<[f64; 4]>, Failure> {
    let mut secmember = vector(4)?;
    let rhs = dedt.opposite();
    if !near {
        let gauss = Gauss::with_min_pivot(dedx, 1.0e-9)?;
        if gauss.is_done() {
            gauss.solve_into(&rhs, &mut secmember)?;
            return values(&secmember).map(Some);
        }
    }
    let mut svd = Svd::new(dedx)?;
    if !svd.is_done() {
        return Ok(None);
    }
    svd.solve(&rhs, &mut secmember, 1.0e-6)?;
    values(&secmember).map(Some)
}

/// `-(D2EDT2 + (2 * D2EDXDT + D2DXdSdt) * sol)` with `D2DXdSdt = D2EDX2 * sol`.
fn second_member(eq: &Equations, sol: &Vector<'_>) -> Result<Vector<'static>, Failure> {
    let mut d2dxdsdt = matrix(4, 4)?;
    eq.d2edx2.multiply(sol, &mut d2dxdsdt)?;
    let coefficient = (2. * &eq.d2edxdt).added(&d2dxdsdt)?;
    Ok(eq
        .d2edt2
        .added(&coefficient.multiplied_vector(sol)?)?
        .opposite())
}

/// The first and second derivatives of the section along the guide (`Section` with
/// `D2Poles`): Gauss with pivot 1e-9 unless `near`, then SVD with cutoff 1e-6.
pub(crate) fn solve_second_derivatives(
    eq: &Equations,
    near: bool,
) -> Result<Option<SecondDerivatives>, Failure> {
    let mut sol = vector(4)?;
    let rhs = eq.dedt.opposite();
    if !near {
        // Precise tolerance!
        let gauss = Gauss::with_min_pivot(&eq.dedx, 1.0e-9)?;
        if gauss.is_done() {
            gauss.solve_into(&rhs, &mut sol)?;
            let mut secmember = second_member(eq, &sol)?;
            gauss.solve(&mut secmember)?;
            return Ok(Some((values(&sol)?, values(&secmember)?)));
        }
    }
    let mut svd = Svd::new(&eq.dedx)?;
    if !svd.is_done() {
        return Ok(None);
    }
    svd.solve(&rhs, &mut sol, 1.0e-6)?;
    let vbis = second_member(eq, &sol)?;
    let mut secmember = vector(4)?;
    svd.solve(&vbis, &mut secmember, 1.0e-6)?;
    Ok(Some((values(&sol)?, values(&secmember)?)))
}
