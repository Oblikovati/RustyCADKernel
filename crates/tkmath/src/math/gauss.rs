//! math_Gauss: a stored `LU_Decompose` factorization of a square matrix.
use super::recipes::{STATUS_OK, lu_decompose_with_progress, lu_solve};
use super::{IntegerVector, Matrix, Vector, stream_real};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use std::fmt;
/// Reusable native-order Gaussian decomposition; `Gauss::new(&matrix)?`.
pub struct Gauss {
    lu: Matrix<'static>,
    index: IntegerVector<'static>,
    sign: f64,
    done: bool,
}
impl Gauss {
    /// Uses OCCT's default minimum pivot of 1e-20; `Gauss::new(&matrix)?`.
    pub fn new(matrix: &Matrix<'_>) -> Result<Self, Failure> {
        Self::with_min_pivot(matrix, 1e-20)
    }
    /// Decomposes a copy of `matrix` with `LU_Decompose`, the given pivot threshold
    /// and the default (unbound, never aborting) progress range;
    /// `Gauss::with_min_pivot(&matrix,1e-12)?`.
    pub fn with_min_pivot(matrix: &Matrix<'_>, minimum: f64) -> Result<Self, Failure> {
        Self::with_progress(matrix, minimum, &ProgressRange::default())
    }
    /// The native constructor `math_Gauss(A, MinPivot, theProgress)`: `progress` is
    /// handed to `LU_Decompose`, so a user break leaves the solver not done (status
    /// `math_Status_UserAborted`); a non-square matrix is refused before the range is
    /// consumed. `Gauss::with_progress(&matrix,1e-20,&indicator.start())?`.
    pub fn with_progress(
        matrix: &Matrix<'_>,
        minimum: f64,
        progress: &ProgressRange<'_>,
    ) -> Result<Self, Failure> {
        if matrix.row_number() != matrix.col_number() {
            return Err(super::not_square(" "));
        }
        let n = matrix.row_number();
        let mut lu = Matrix::with_bounds(1, n, 1, n).unwrap();
        lu.initialized(matrix)?;
        let mut index = IntegerVector::with_bounds(1, n)?;
        let mut sign = 0.0;
        let status = lu_decompose_with_progress(&mut lu, &mut index, &mut sign, minimum, progress)?;
        let done = status == STATUS_OK;
        Ok(Self {
            lu,
            index,
            sign,
            done,
        })
    }
    /// Reports successful decomposition; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Multiplies the permutation sign by the diagonal in order; `solver.determinant()?`.
    pub fn determinant(&self) -> Result<f64, Failure> {
        self.check_done()?;
        let mut result = self.sign;
        for index in 1..=self.lu.upper_row() {
            result *= self.lu[(index, index)];
        }
        Ok(result)
    }
    /// Solves in place with an explicit size check; `solver.solve(&mut rhs)?`.
    pub fn solve(&self, rhs: &mut Vector<'_>) -> Result<(), Failure> {
        self.check_done()?;
        if rhs.length() != self.lu.row_number() {
            return Err(Failure::with_type("Standard_DimensionError", "", ""));
        }
        lu_solve(&self.lu, &self.index, rhs)
    }
    /// Copies RHS to existing bounds then solves, preserving the overload's validation order; `solver.solve_into(&rhs,&mut solution)?`.
    pub fn solve_into(&self, rhs: &Vector<'_>, solution: &mut Vector<'_>) -> Result<(), Failure> {
        self.check_done()?;
        solution.initialized(rhs)?;
        lu_solve(&self.lu, &self.index, solution)
    }
    /// Writes the inverse using one solve per column; `solver.invert(&mut matrix)?`.
    pub fn invert(&self, inverse: &mut Matrix<'_>) -> Result<(), Failure> {
        self.check_done()?;
        let n = self.lu.row_number();
        if inverse.row_number() != n || inverse.col_number() != n {
            return Err(Failure::with_type("Standard_DimensionError", " ", ""));
        }
        let mut column = Vector::with_bounds(1, n).unwrap();
        for j in 1..=n {
            self.inverse_column(inverse, &mut column, j)?;
        }
        Ok(())
    }
    fn inverse_column(
        &self,
        inverse: &mut Matrix<'_>,
        column: &mut Vector<'_>,
        j: i32,
    ) -> Result<(), Failure> {
        column.init(0.0);
        column[j] = 1.0;
        lu_solve(&self.lu, &self.index, column)?;
        let (lower_row, lower_col) = (inverse.lower_row(), inverse.lower_col());
        for i in 1..=self.lu.row_number() {
            inverse[(i + lower_row - 1, j + lower_col - 1)] = column[i];
        }
        Ok(())
    }
    /// Writes the native status text; `solver.dump(&mut text)?`.
    ///
    /// The "determinant" line prints the stored permutation sign `D`, not the
    /// determinant, exactly as OCCT does.
    pub fn dump(&self, stream: &mut impl fmt::Write) -> fmt::Result {
        write!(stream, "math_Gauss ")?;
        if !self.done {
            return writeln!(stream, " Status = not Done ");
        }
        writeln!(stream, " Status = Done ")?;
        writeln!(stream, " Determinant of A = {}", stream_real(self.sign))
    }
    fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_type("StdFail_NotDone", " ", ""));
        }
        Ok(())
    }
}
display_via_dump!(Gauss);
