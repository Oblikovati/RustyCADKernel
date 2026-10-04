use super::{Failure, Householder, Matrix};
impl Householder {
    pub(super) fn construct(
        matrix: &Matrix<'_>,
        rhs: &Matrix<'_>,
        bounds: [i32; 4],
        q_lower_col: i32,
        epsilon: f64,
    ) -> Result<Self, Failure> {
        let [rl, ru, cl, cu] = bounds;
        let (rows, cols) = (ru - rl + 1, cu - cl + 1);
        let mut result = Self {
            solution: allocate(1, cols, 1, rhs.col_number())?,
            q: allocate(1, rows, q_lower_col, q_lower_col + cols - 1)?,
            done: false,
            input_bounds: bounds,
        };
        result.perform(matrix, rhs, epsilon)?;
        Ok(result)
    }
    pub(super) fn copy_input_row(
        &mut self,
        matrix: &Matrix<'_>,
        rhs: &Matrix<'_>,
        transformed: &mut Matrix<'_>,
        row: i32,
    ) -> Result<(), Failure> {
        for col in 1..=self.q.col_number() {
            *self.q.change_value(row, col)? = *matrix.value(
                row + self.input_bounds[0] - 1,
                col + self.input_bounds[2] - 1,
            )?;
        }
        for col in 1..=rhs.col_number() {
            transformed[(row, col)] = *rhs.value(row + rhs.lower_row() - 1, col)?;
        }
        Ok(())
    }
}
pub(super) fn allocate(rl: i32, ru: i32, cl: i32, cu: i32) -> Result<Matrix<'static>, Failure> {
    Matrix::with_bounds(rl, ru, cl, cu).map_err(|error| error.into_failure("math_Householder"))
}
