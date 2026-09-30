use super::{Crout, Failure, Matrix, Uzawa, Vector};
impl Uzawa {
    pub(super) fn direct(
        &mut self,
        matrix: &Matrix<'_>,
        start: &Vector<'_>,
    ) -> Result<(), Failure> {
        self.iterations = 1;
        self.gram_lower(matrix);
        let solver: Crout = Crout::new(&self.inverse)?;
        self.inverse.initialized(solver.inverse()?)?;
        solver.solve_into(&self.initial, &mut self.dual)?;
        self.update_error(matrix);
        self.store_solution(start)
    }

    pub(super) fn store_solution(&mut self, start: &Vector<'_>) -> Result<(), Failure> {
        self.solution.initialized(start)?;
        self.solution.add(&self.error)?;
        self.done = true;
        Ok(())
    }
    fn gram_lower(&mut self, matrix: &Matrix<'_>) {
        for row in 1..=matrix.row_number() {
            for column in 1..=row {
                self.inverse[(row, column)] = gram_entry(matrix, row, column);
            }
        }
    }
    pub(super) fn update_error(&mut self, matrix: &Matrix<'_>) -> f64 {
        let mut maximum: f64 = 0.0;
        for column in 1..=matrix.col_number() {
            let previous: f64 = self.error[column];
            self.error[column] = -matrix[(1, column)] * self.dual[1];
            for row in 2..=matrix.row_number() {
                self.error[column] -= matrix[(row, column)] * self.dual[row];
            }
            if self.iterations > 1 {
                maximum = error_maximum(maximum, (self.error[column] - previous).abs(), column);
            }
        }
        maximum
    }
}
fn gram_entry(matrix: &Matrix<'_>, row: i32, column: i32) -> f64 {
    let mut value: f64 = matrix[(row, 1)] * matrix[(column, 1)];
    for index in 2..=matrix.col_number() {
        value += matrix[(row, index)] * matrix[(column, index)];
    }
    value
}
pub(super) fn error_maximum(previous: f64, value: f64, index: i32) -> f64 {
    if index == 1 || previous < value {
        value
    } else {
        previous
    }
}
