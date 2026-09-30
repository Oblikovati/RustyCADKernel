//! MathLin_EigenSearch.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::utils;
use crate::math::{Matrix, Vector};
use crate::math_utils::Status;
/// Tridiagonal eigenpairs in native iteration order; `TridiagonalResult::default()`.
#[derive(Default)]
pub struct TridiagonalResult {
    /// Native completion status; `result.status`.
    pub status: Status,
    /// Optional eigenvalues, even after final residual rejection; `result.eigen_values`.
    pub eigen_values: Option<Vector<'static>>,
    /// Eigenvectors stored as columns; `result.eigen_vectors`.
    pub eigen_vectors: Option<Matrix<'static>>,
    /// Validated input dimension; `result.dimension`.
    pub dimension: usize,
}
impl TridiagonalResult {
    /// Reports native success; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
}
impl From<&TridiagonalResult> for bool {
    fn from(r: &TridiagonalResult) -> bool {
        r.is_done()
    }
}
struct TridiagonalState {
    diag: Vector<'static>,
    sub: Vector<'static>,
    vectors: Matrix<'static>,
}
impl TridiagonalState {
    fn new(diag: &Vector<'_>, sub: &Vector<'_>) -> Self {
        let n: usize = diag.length() as usize;
        let mut d: Vector<'static> = Vector::with_size(n);
        d.initialized(diag).unwrap();
        let mut e: Vector<'static> = Vector::with_size(n);
        let offset: usize = usize::from(sub.length() == diag.length());
        for i in 0..n - 1 {
            *e.change_at(i).unwrap() = *sub.at(i + offset).unwrap();
        }
        let mut v: Matrix<'static> = Matrix::filled(0, n as i32 - 1, 0, n as i32 - 1, 0.0).unwrap();
        for i in 0..n as i32 {
            v[(i, i)] = 1.0;
        }
        Self {
            diag: d,
            sub: e,
            vectors: v,
        }
    }
    fn length(&self) -> usize {
        self.diag.length() as usize
    }
    fn finite(&self) -> bool {
        self.diag
            .values()
            .iter()
            .chain(self.sub.values())
            .all(|x| x.is_finite())
            && utils::is_finite_matrix(&self.vectors)
    }
    /// Native `do { ... } while (aEnd != aStart)` loop for one start row.
    fn converge(&mut self, start: usize, limit: u32) -> Status {
        let n: usize = self.length();
        for iteration in 0..=limit {
            let end: usize = utils::find_submatrix_end(&self.diag, &self.sub, start, n);
            if end == start {
                return Status::Ok;
            }
            if iteration == limit {
                return Status::MaxIterations;
            }
            let shift: f64 = utils::compute_wilkinson_shift(&self.diag, &self.sub, start, end);
            if !shift.is_finite()
                || !utils::perform_ql_step(
                    &mut self.diag,
                    &mut self.sub,
                    &mut self.vectors,
                    start,
                    end,
                    shift,
                    n,
                )
                || !self.finite()
            {
                return Status::NumericalError;
            }
        }
        unreachable!("the loop returns by iteration == limit")
    }
}
/// Accepts natural or legacy subdiagonal layout; `eigen_tridiagonal(&diag,&sub,30)`.
pub fn eigen_tridiagonal(diag: &Vector<'_>, sub: &Vector<'_>, limit: u32) -> TridiagonalResult {
    let n: usize = diag.length() as usize;
    if n == 0
        || limit == 0
        || !(sub.length() == diag.length() || sub.length() + 1 == diag.length())
        || !diag
            .values()
            .iter()
            .chain(sub.values())
            .all(|x| x.is_finite())
    {
        return TridiagonalResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let original: TridiagonalState = TridiagonalState::new(diag, sub);
    let mut state: TridiagonalState = TridiagonalState::new(diag, sub);
    for start in 0..n {
        let status: Status = state.converge(start, limit);
        if status != Status::Ok {
            return TridiagonalResult {
                status,
                dimension: n,
                ..Default::default()
            };
        }
    }
    let status: Status = if state.finite()
        && utils::validate_eigenpairs(&original.diag, &original.sub, &state.diag, &state.vectors)
    {
        Status::Ok
    } else {
        Status::NumericalError
    };
    TridiagonalResult {
        status,
        dimension: n,
        eigen_values: Some(state.diag),
        eigen_vectors: Some(state.vectors),
    }
}
/// Copies a zero-based column; absent vectors produce the native zero singleton; `get_eigen_vector(&result,0)`.
pub fn get_eigen_vector(
    result: &TridiagonalResult,
    index: usize,
) -> Result<Vector<'static>, rustycad_tkernel::standard::Failure> {
    let Some(v) = result.eigen_vectors.as_ref() else {
        return Ok(Vector::with_size(1));
    };
    v.col(index as i32)
}
