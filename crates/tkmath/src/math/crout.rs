//! Symmetric lower-triangle solver translated from math_Crout at pinned OCCT 3d097a0.
use super::{Matrix, Vector};
use rustycad_tkernel::standard::Failure;
mod factor;
mod inverse;
/// Crout decomposition using only the supplied lower triangle; `Crout::new(&matrix)?`.
pub struct Crout {
    inverse: Matrix<'static>,
    determinant: f64,
    done: bool,
}
impl Crout {
    /// Uses the default native minimum pivot of 1e-20; `Crout::new(&matrix)?`.
    pub fn new(matrix: &Matrix<'_>) -> Result<Self, Failure> {
        Self::with_min_pivot(matrix, 1e-20)
    }
    /// Decomposes with the supplied absolute pivot threshold; `Crout::with_min_pivot(&matrix,1e-12)?`.
    pub fn with_min_pivot(matrix: &Matrix<'_>, minimum: f64) -> Result<Self, Failure> {
        let n = matrix.row_number();
        if n != matrix.col_number() {
            return Err(super::not_square(" "));
        }
        let mut result = Self {
            inverse: Matrix::with_bounds(1, n, 1, n).unwrap(),
            determinant: 1.0,
            done: false,
        };
        let mut lower = Matrix::with_bounds(1, n, 1, n).unwrap();
        let mut diagonal = Vector::with_bounds(1, n).unwrap();
        if !result.factor(matrix, &mut lower, &mut diagonal, minimum) {
            return Ok(result);
        }
        inverse::invert_lower(&mut lower);
        result.build_inverse(&lower, &diagonal);
        result.done = true;
        Ok(result)
    }
    /// Reports a successful factorization; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Returns the diagonal product or native not-done failure; `solver.determinant()?`.
    pub fn determinant(&self) -> Result<f64, Failure> {
        self.check_done()?;
        Ok(self.determinant)
    }
    /// Returns the inverse's defined lower triangle; `solver.inverse()?`.
    pub fn inverse(&self) -> Result<&Matrix<'static>, Failure> {
        self.check_done()?;
        Ok(&self.inverse)
    }
    /// Copies the stored inverse into matching destination dimensions; `solver.invert(&mut destination)?`.
    pub fn invert(&self, destination: &mut Matrix<'_>) -> Result<(), Failure> {
        self.check_done()?;
        destination.initialized(&self.inverse)
    }
    /// Multiplies RHS by the stored symmetric inverse; `solver.solve_into(&rhs,&mut solution)?`.
    pub fn solve_into(&self, rhs: &Vector<'_>, solution: &mut Vector<'_>) -> Result<(), Failure> {
        self.check_done()?;
        for row in 1..=self.inverse.row_number() {
            let value = self.solution_component(rhs, row)?;
            *solution.change_value(solution.lower() + row - 1)? = value;
        }
        Ok(())
    }
    /// Preserves native aliasing: each newly written result becomes part of the next RHS; `solver.solve_in_place(&mut rhs)?`.
    pub fn solve_in_place(&self, rhs: &mut Vector<'_>) -> Result<(), Failure> {
        self.check_done()?;
        for row in 1..=self.inverse.row_number() {
            self.solve_aliased_row(rhs, row)?;
        }
        Ok(())
    }
    fn solve_aliased_row(&self, rhs: &mut Vector<'_>, row: i32) -> Result<(), Failure> {
        let index = rhs.lower() + row - 1;
        *rhs.change_value(index)? = self.inverse[(row, 1)] * rhs.value(rhs.lower())?;
        for col in 2..=row {
            *rhs.change_value(index)? +=
                self.inverse[(row, col)] * rhs.value(rhs.lower() + col - 1)?;
        }
        for col in row + 1..=self.inverse.row_number() {
            *rhs.change_value(index)? +=
                self.inverse[(col, row)] * rhs.value(rhs.lower() + col - 1)?;
        }
        Ok(())
    }
    fn solution_component(&self, rhs: &Vector<'_>, row: i32) -> Result<f64, Failure> {
        let mut value = self.inverse[(row, 1)] * rhs.value(rhs.lower())?;
        for col in 2..=row {
            value += self.inverse[(row, col)] * rhs.value(rhs.lower() + col - 1)?;
        }
        for col in row + 1..=self.inverse.row_number() {
            value += self.inverse[(col, row)] * rhs.value(rhs.lower() + col - 1)?;
        }
        Ok(value)
    }
    fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_type("StdFail_NotDone", " ", ""));
        }
        Ok(())
    }
}
impl Crout {
    /// Writes the native `Dump` status text; `solver.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(stream, "math_Crout ")?;
        if self.done {
            writeln!(stream, " Status = Done ")
        } else {
            writeln!(stream, " Status = not Done ")
        }
    }
}
display_via_dump!(Crout);
