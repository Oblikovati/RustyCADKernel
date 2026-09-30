use super::direct::error_maximum;
use super::{Failure, Matrix, Uzawa, Vector};
impl Uzawa {
    pub(super) fn iterate(
        &mut self,
        matrix: &Matrix<'_>,
        start: &Vector<'_>,
        equalities: i32,
        tolerances: [f64; 2],
        limit: i32,
    ) -> Result<(), Failure> {
        for row in 1..=matrix.row_number() {
            self.dual[row] = if row <= equalities { 0.0 } else { 1.0 };
        }
        let rho: f64 = (1.0 / 2.0_f64.sqrt()) / matrix_norm2(matrix);
        self.iterations = 1;
        while self.iterations <= limit {
            let error: f64 = self.update_error(matrix);
            let dual_error: f64 = self.update_dual(matrix, equalities, rho);
            if self.iterations > 1 && error <= tolerances[0] {
                return self.finish(start, dual_error, tolerances[1]);
            }
            self.iterations += 1;
        }
        self.done = false;
        Ok(())
    }
    fn update_dual(&mut self, matrix: &Matrix<'_>, equalities: i32, rho: f64) -> f64 {
        let mut maximum: f64 = 0.0;
        for row in 1..=matrix.row_number() {
            let residual: f64 = self.residual(matrix, row);
            let change: f64 = self.dual_step(row, equalities, rho * residual);
            maximum = error_maximum(maximum, change, row);
        }
        maximum
    }
    fn residual(&self, matrix: &Matrix<'_>, row: i32) -> f64 {
        let mut value: f64 = matrix[(row, 1)] * self.error[1] + self.initial[row];
        for column in 2..=matrix.col_number() {
            value += matrix[(row, column)] * self.error[column];
        }
        value
    }
    fn dual_step(&mut self, row: i32, equalities: i32, step: f64) -> f64 {
        if row <= equalities {
            self.dual[row] += step;
            return step.abs();
        }
        let previous: f64 = self.dual[row];
        let candidate: f64 = previous + step;
        self.dual[row] = if 0.0 < candidate { candidate } else { 0.0 };
        (self.dual[row] - previous).abs()
    }
    fn finish(&mut self, start: &Vector<'_>, error: f64, tolerance: f64) -> Result<(), Failure> {
        if error
            .partial_cmp(&tolerance)
            .is_none_or(|order| order == std::cmp::Ordering::Greater)
        {
            self.done = false;
            return Ok(());
        }
        self.store_solution(start)
    }
}
fn matrix_norm2(matrix: &Matrix<'_>) -> f64 {
    let mut norm: f64 = 0.0;
    for row in 1..=matrix.row_number() {
        norm += row_norm2(matrix, row);
    }
    norm
}
fn row_norm2(matrix: &Matrix<'_>, row: i32) -> f64 {
    let mut norm: f64 = matrix[(row, 1)] * matrix[(row, 1)];
    for column in 2..=matrix.col_number() {
        norm += matrix[(row, column)] * matrix[(row, column)];
    }
    norm
}
