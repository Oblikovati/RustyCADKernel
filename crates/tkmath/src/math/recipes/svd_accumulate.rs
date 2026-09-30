//! Accumulation of the right and left Householder transforms in SVD_Decompose.
use super::svd::SvdWork;
impl SvdWork<'_, '_, '_, '_, '_> {
    /// Builds V from the stored right reflections, last column first.
    pub(super) fn accumulate_right(&mut self) {
        for index in (1..=self.n).rev() {
            if index < self.n {
                self.right_column(index, self.rv1[index + 1]);
            }
            self.v[(index, index)] = 1.0;
        }
    }
    fn right_column(&mut self, index: i32, g: f64) {
        if g != 0.0 {
            for row in index + 1..=self.n {
                self.v[(row, index)] = (self.a[(index, row)] / self.a[(index, index + 1)]) / g;
            }
            self.right_projection(index);
        }
        for col in index + 1..=self.n {
            self.v[(index, col)] = 0.0;
            self.v[(col, index)] = 0.0;
        }
    }
    fn right_projection(&mut self, index: i32) {
        for col in index + 1..=self.n {
            let sum: f64 = self.right_dot(index, col);
            for row in index + 1..=self.n {
                self.v[(row, col)] += sum * self.v[(row, index)];
            }
        }
    }
    fn right_dot(&self, index: i32, col: i32) -> f64 {
        let mut sum: f64 = 0.0;
        for row in index + 1..=self.n {
            sum += self.a[(index, row)] * self.v[(row, col)];
        }
        sum
    }
    /// Turns `a` into U from the stored left reflections, last column first.
    pub(super) fn accumulate_left(&mut self) {
        for index in (1..=self.n).rev() {
            for col in index + 1..=self.n {
                self.a[(index, col)] = 0.0;
            }
            self.left_column(index);
            self.a[(index, index)] += 1.0;
        }
    }
    fn left_column(&mut self, index: i32) {
        let g: f64 = self.w[index];
        if g == 0.0 {
            for row in index..=self.m {
                self.a[(row, index)] = 0.0;
            }
            return;
        }
        let reciprocal: f64 = 1.0 / g;
        self.left_projection(index, reciprocal);
        for row in index..=self.m {
            self.a[(row, index)] *= reciprocal;
        }
    }
    fn left_projection(&mut self, index: i32, reciprocal: f64) {
        for col in index + 1..=self.n {
            let sum: f64 = self.column_dot(index, col, index + 1);
            let f: f64 = (sum / self.a[(index, index)]) * reciprocal;
            for row in index..=self.m {
                self.a[(row, col)] += f * self.a[(row, index)];
            }
        }
    }
}
