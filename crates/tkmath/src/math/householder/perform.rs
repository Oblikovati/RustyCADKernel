use super::setup::allocate;
use super::{Failure, Householder, Matrix};
impl Householder {
    /// Repeats native protected reduction on existing storage; `solver.perform(&a,&b,1e-20)?`.
    /// OCCT clears `Done` when a small pivot stops the reduction before back substitution.
    pub fn perform(
        &mut self,
        matrix: &Matrix<'_>,
        rhs: &Matrix<'_>,
        epsilon: f64,
    ) -> Result<(), Failure> {
        let rows = self.q.row_number();
        let mut transformed = allocate(1, rows, 1, rhs.col_number())?;
        for row in 1..=rows {
            self.copy_input_row(matrix, rhs, &mut transformed, row)?;
        }
        self.reduce(&mut transformed, epsilon)
    }
}
