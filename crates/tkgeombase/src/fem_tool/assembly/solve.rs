// Rust translation of ModelingData/TKGeomBase/FEmTool/FEmTool_Assembly.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FEmTool_Assembly::Solve` and `Solution`: the Cholesky factor of H, and with
//! constraints the factor of M = G H^-1 G^t whose profile follows the blocks of H.
use super::{Assembly, ConstraintRow};
use crate::fem_tool::profile_matrix::ProfileMatrix;
use crate::fem_tool::sparse_matrix::SparseMatrix;
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::math::Vector;

// H1(k, l) != 0 for any variables k of row `gi` and l of row `gj`.
fn rows_coupled(
    h1: &Array2<'_, i32>,
    gi: &ConstraintRow,
    gj: &ConstraintRow,
) -> Result<bool, Failure> {
    for a in gi {
        for k in a.lower()..=a.upper() {
            for b in gj {
                for l in b.lower()..=b.upper() {
                    let coupled: i32 = *h1
                        .value(k, l)
                        .map_err(|error| array_failure(error, "FEmTool_Assembly::Solve"))?;
                    if coupled != 0 {
                        return Ok(true);
                    }
                }
            }
        }
    }
    Ok(false)
}

// The scalar product of a constraint row with `v`.
fn row_dot(row: &ConstraintRow, v: &Vector<'_>) -> Result<f64, Failure> {
    let mut m: f64 = 0.;
    for a in row {
        for j in a.lower()..=a.upper() {
            m += *v.value(j)? * a[j];
        }
    }
    Ok(m)
}

impl Assembly {
    /// `Solve()`: factorizes H, then (with constraints) G H^-1 G^t; false when
    /// either is not positive definite; `assembly.solve()?`.
    pub fn solve(&mut self) -> Result<bool, Failure> {
        self.is_solved = self.h.decompose()?;
        if self.g.is_empty() || !self.is_solved {
            return Ok(self.is_solved);
        }
        let rebuild: bool = self
            .ghgt
            .get()
            .is_none_or(|ghgt| ghgt.row_number() != self.g.len() as i32);
        if rebuild {
            let first: Array1<'static, i32> = self.schur_profile()?;
            self.ghgt = Handle::new(ProfileMatrix::new(&first)?);
        }
        self.fill_schur()?;
        self.is_solved = self.ghgt.decompose()?;
        Ok(self.is_solved)
    }

    /// `Solution(Solution)`: StdFail_NotDone before a successful Solve;
    /// `assembly.solution(&mut x)?`.
    pub fn solution(&self, solution: &mut Vector<'_>) -> Result<(), Failure> {
        if !self.is_solved {
            return Err(failure("StdFail_NotDone", "FEmTool_Assembly::Solution"));
        }
        if self.g.is_empty() {
            return self.h.solve(&self.b, solution);
        }
        let mut v1: Vector<'static> = Vector::with_bounds(self.b.lower(), self.b.upper())?;
        self.h.solve(&self.b, &mut v1)?;
        let length: i32 = self.g.len() as i32;
        let (mut l, mut v2) = (
            Vector::with_bounds(1, length)?,
            Vector::with_bounds(1, length)?,
        );
        for (row, i) in self.g.iter().zip(1..) {
            v2[i] = row_dot(row, &v1)? - self.c[(i - 1) as usize];
        }
        self.ghgt.solve(&v2, &mut l)?;
        v1.initialized(&self.b)?;
        for (row, i) in self.g.iter().zip(1..) {
            for a in row {
                for j in a.lower()..=a.upper() {
                    *v1.change_value(j)? -= l[i] * a[j];
                }
            }
        }
        self.h.solve(&v1, solution)
    }

    // The block pattern H1 of H: 1 except between the variables of distinct
    // diagonal blocks (a block ends before a row i that no later row links to i-1).
    fn block_pattern(&self) -> Result<Array2<'static, i32>, Failure> {
        let n: i32 = self.nb_glob_var();
        let mut h1: Array2<'static, i32> = Array2::with_bounds(1, n, 1, n)
            .map_err(|error| array_failure(error, "FEmTool_Assembly::Solve"))?;
        h1.init(1);
        let (mut block_beg, mut i): (i32, i32) = (1, 2);
        while i <= n {
            let block_end: i32 = i - 1;
            if !self.h.is_in_profile(i, block_end)? {
                match self.linked_after(i, block_end)? {
                    Some(j) => i = j,
                    None => {
                        for j in i..=n {
                            for k in block_beg..=block_end {
                                h1[(j, k)] = 0;
                                h1[(k, j)] = 0;
                            }
                        }
                        block_beg = block_end + 1;
                    }
                }
            }
            i += 1;
        }
        Ok(h1)
    }

    // The first row j > i of H storing column `block_end`.
    fn linked_after(&self, i: i32, block_end: i32) -> Result<Option<i32>, Failure> {
        for j in i + 1..=self.nb_glob_var() {
            if self.h.is_in_profile(j, block_end)? {
                return Ok(Some(j));
            }
        }
        Ok(None)
    }

    // FirstIndexes of G H^-1 G^t: for row i, the first row j <= i coupled with it
    // through H1. A row coupled with none (an empty constraint row) is left
    // uninitialized natively; it is 0 here.
    fn schur_profile(&self) -> Result<Array1<'static, i32>, Failure> {
        let h1: Array2<'static, i32> = self.block_pattern()?;
        let length: i32 = self.g.len() as i32;
        let mut first: Array1<'static, i32> = Array1::with_bounds(1, length)
            .map_err(|error| array_failure(error, "FEmTool_Assembly::Solve"))?;
        first.init(0);
        for (gi, i) in self.g.iter().zip(1..) {
            for (gj, j) in self.g.iter().zip(1..=i) {
                if rows_coupled(&h1, gi, gj)? {
                    first[i] = j;
                    break;
                }
            }
        }
        Ok(first)
    }

    // M(k, i) = G(k) . H^-1 G(i)^t for the stored k >= i.
    fn fill_schur(&self) -> Result<(), Failure> {
        let (lower, upper) = (self.b.lower(), self.b.upper());
        let (mut gi, mut qi) = (
            Vector::with_bounds(lower, upper)?,
            Vector::with_bounds(lower, upper)?,
        );
        self.ghgt.init(0.);
        for (row, i) in self.g.iter().zip(1..) {
            gi.init(0.);
            for a in row {
                for j in a.lower()..=a.upper() {
                    *gi.change_value(j)? = a[j];
                }
            }
            self.h.solve(&gi, &mut qi)?;
            for (row_k, k) in self.g.iter().zip(1..).skip((i - 1) as usize) {
                if self.ghgt.is_in_profile(k, i)? {
                    let m: f64 = row_dot(row_k, &qi)?;
                    *self.ghgt.change_value(k, i)? = m;
                }
            }
        }
        Ok(())
    }
}
