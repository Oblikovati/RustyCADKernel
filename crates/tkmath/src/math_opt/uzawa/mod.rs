//! Modern Uzawa constrained least squares, MathOpt_Uzawa.hxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871 (distinct from the classic `math::Uzawa`).
use super::utils::{is_finite_matrix, is_finite_vector};
use crate::math::{Matrix, Vector};
use crate::math_utils::Status;

mod direct;
mod iterative;

/// Uzawa outcome with optional primal, dual and auxiliary data; `MathOpt::UzawaResult`.
///
/// ```
/// let result = rustycad_tkmath::math_opt::UzawaResult::default();
/// assert!(!result.is_done() && !bool::from(&result));
/// ```
#[derive(Default)]
pub struct UzawaResult {
    /// Outcome, `NotConverged` until computed; `Status`.
    pub status: Status,
    /// Solution X; `Solution`.
    pub solution: Option<Vector<'static>>,
    /// Dual (Lagrange) variables; `Dual`.
    pub dual: Option<Vector<'static>>,
    /// X - X0; `Error`.
    pub error: Option<Vector<'static>>,
    /// C*X0 - S; `InitialError`.
    pub initial_error: Option<Vector<'static>>,
    /// (C*C^T)^-1, set by the equality-only direct path; `InverseCTC`.
    pub inverse_ctc: Option<Matrix<'static>>,
    /// Iterations performed; `NbIterations`.
    pub iterations: u32,
}

impl UzawaResult {
    /// True when the status is `Ok`; `UzawaResult::IsDone`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
}

impl From<&UzawaResult> for bool {
    /// Native `explicit operator bool`, equal to `IsDone`.
    fn from(result: &UzawaResult) -> bool {
        result.is_done()
    }
}

/// Uzawa tolerances and iteration budget; `MathOpt::UzawaConfig`.
///
/// ```
/// let config = rustycad_tkmath::math_opt::UzawaConfig::default();
/// assert_eq!((config.eps_lix, config.eps_lic, config.max_iterations), (1e-6, 1e-6, 500));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct UzawaConfig {
    /// Primal change tolerance; `EpsLix`.
    pub eps_lix: f64,
    /// Dual change tolerance; `EpsLic`.
    pub eps_lic: f64,
    /// Iteration limit; `MaxIterations`.
    pub max_iterations: u32,
}

impl Default for UzawaConfig {
    fn default() -> Self {
        Self {
            eps_lix: 1.0e-6,
            eps_lic: 1.0e-6,
            max_iterations: 500,
        }
    }
}

/// Minimizes `||X - X0||^2` subject to the first `nce` rows of `C*X = S` as equalities and
/// the last `nci` rows as `C*X <= S`: Crout inversion of `C*C^T` when `nci == 0`, else the
/// projected dual iteration; `MathOpt::Uzawa`.
///
/// ```
/// use rustycad_tkmath::{math::{Matrix, Vector}, math_opt::{uzawa, UzawaConfig}};
/// let c = Matrix::filled(0, 0, 0, 1, 1.0).unwrap();
/// let s = Vector::filled(0, 0, 2.0).unwrap();
/// let result = uzawa(&c, &s, &Vector::with_size(2), 1, 0, &UzawaConfig::default());
/// assert!(result.is_done());
/// assert_eq!(result.solution.unwrap()[0], 1.0);
/// ```
pub fn uzawa(
    cont: &Matrix<'_>,
    secont: &Vector<'_>,
    starting_point: &Vector<'_>,
    nce: usize,
    nci: usize,
    config: &UzawaConfig,
) -> UzawaResult {
    if !is_valid_input(cont, secont, starting_point, nce, nci, config) {
        return UzawaResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let problem: Problem<'_, '_, '_> = Problem {
        cont,
        start: starting_point,
        rows: cont.row_number() as usize,
        cols: cont.col_number() as usize,
        nce,
    };
    let initial_error: Vector<'static> = problem.initial_error(secont);
    if nci == 0 {
        return direct::solve(&problem, initial_error);
    }
    iterative::solve(&problem, initial_error, config)
}

/// Equality-only convenience form, `Uzawa(C, S, X0, C.RowSize(), 0)`; `MathOpt::UzawaEquality`.
///
/// ```
/// use rustycad_tkmath::{math::{Matrix, Vector}, math_opt::{uzawa_equality, UzawaConfig}};
/// let c = Matrix::filled(0, 0, 0, 1, 1.0).unwrap();
/// let s = Vector::filled(0, 0, 2.0).unwrap();
/// assert!(uzawa_equality(&c, &s, &Vector::with_size(2), &UzawaConfig::default()).is_done());
/// ```
pub fn uzawa_equality(
    cont: &Matrix<'_>,
    secont: &Vector<'_>,
    starting_point: &Vector<'_>,
    config: &UzawaConfig,
) -> UzawaResult {
    let rows: usize = cont.row_number() as usize;
    uzawa(cont, secont, starting_point, rows, 0, config)
}

/// Native dimension and finiteness validation (unsigned `nci != rows - nce` after `nce <= rows`).
fn is_valid_input(
    cont: &Matrix<'_>,
    secont: &Vector<'_>,
    start: &Vector<'_>,
    nce: usize,
    nci: usize,
    config: &UzawaConfig,
) -> bool {
    let rows: usize = cont.row_number() as usize;
    let cols: usize = cont.col_number() as usize;
    nce <= rows
        && nci == rows - nce
        && secont.length() as usize == rows
        && start.length() as usize == cols
        && is_finite_matrix(cont)
        && is_finite_vector(secont)
        && is_finite_vector(start)
        && config.max_iterations != 0
        && config.eps_lix.is_finite()
        && config.eps_lix > 0.0
        && config.eps_lic.is_finite()
        && config.eps_lic > 0.0
}

/// Validated inputs shared by the direct and iterative paths.
struct Problem<'c, 's, 'v> {
    cont: &'c Matrix<'s>,
    start: &'c Vector<'v>,
    rows: usize,
    cols: usize,
    nce: usize,
}

impl Problem<'_, '_, '_> {
    fn c(&self, row: usize, col: usize) -> f64 {
        *self.cont.at(row, col).unwrap()
    }

    /// `C*X0 - S`, each row summed from zero before subtracting `S`.
    fn initial_error(&self, secont: &Vector<'_>) -> Vector<'static> {
        let mut error: Vector<'static> = Vector::with_size(self.rows);
        for row in 0..self.rows {
            let mut sum: f64 = 0.0;
            for col in 0..self.cols {
                sum += self.c(row, col) * self.start.at(col).unwrap();
            }
            *error.change_at(row).unwrap() = sum - secont.at(row).unwrap();
        }
        error
    }

    /// `X0 + E`, the native `aResul`.
    fn solution(&self, shift: &Vector<'_>) -> Vector<'static> {
        let mut result: Vector<'static> = Vector::with_size(self.cols);
        for col in 0..self.cols {
            *result.change_at(col).unwrap() = self.start.at(col).unwrap() + shift.at(col).unwrap();
        }
        result
    }

    /// `-C^T * dual`, accumulated by subtraction from zero as native code does.
    fn primal_shift(&self, dual: &Vector<'_>, col: usize) -> f64 {
        let mut sum: f64 = 0.0;
        for row in 0..self.rows {
            sum -= self.c(row, col) * dual.at(row).unwrap();
        }
        sum
    }
}

/// Stores the primal, dual and shift vectors with the native finiteness verdict.
fn finish(
    result: &mut UzawaResult,
    solution: Vector<'static>,
    dual: Vector<'static>,
    error: Vector<'static>,
) {
    let finite: bool =
        is_finite_vector(&solution) && is_finite_vector(&dual) && is_finite_vector(&error);
    result.status = if finite {
        Status::Ok
    } else {
        Status::NumericalError
    };
    result.solution = Some(solution);
    result.dual = Some(dual);
    result.error = Some(error);
}
