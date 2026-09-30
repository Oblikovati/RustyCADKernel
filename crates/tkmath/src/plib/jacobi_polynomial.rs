// Rust translation of FoundationClasses/TKMath/PLib/PLib_JacobiPolynomial.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Jacobi polynomials orthogonal for the weight W(t) = (1-t^2)^(NivConstr+1).
use super::access::{array_cell, failure, grid_cell};
use super::constraint::niv_constr;
use super::jacobi_tables::{self, ChunkedTable};
use crate::geom_abs::Shape;
use crate::math::{Vector, gauss_points};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

mod basis;
pub(in crate::plib) use basis::BasisRows;
mod conversion;
mod errors;

// Largest degree of Q(t) the native tables cover.
const MAX_DEGREE: i32 = 30;
// Placeholder the native code writes where a table entry is undefined.
const INVALID_VALUE: f64 = -999.0;
// Gauss point counts the weight tables are tabulated for, in storage order.
const GAUSS_COUNTS: [i32; 9] = [8, 10, 15, 20, 25, 30, 40, 50, 61];

static WEIGHTS_DB: [&ChunkedTable; 3] = [
    &jacobi_tables::WEIGHTS_DB_C0,
    &jacobi_tables::WEIGHTS_DB_C1,
    &jacobi_tables::WEIGHTS_DB_C2,
];
static WEIGHTS_DB0: [&ChunkedTable; 3] = [
    &jacobi_tables::WEIGHTS_DB0_C0,
    &jacobi_tables::WEIGHTS_DB0_C1,
    &jacobi_tables::WEIGHTS_DB0_C2,
];
static MAX_VALUES_DB: [&ChunkedTable; 3] = [
    &jacobi_tables::MAX_VALUES_DB_C0,
    &jacobi_tables::MAX_VALUES_DB_C1,
    &jacobi_tables::MAX_VALUES_DB_C2,
];

/// Reads a native table entry, failing where the C++ pointer would leave the array;
/// `table_value(&TABLE, 3, "MaxValuesDB")?`.
fn table_value(table: &ChunkedTable, index: usize, name: &str) -> Result<f64, Failure> {
    table.get(index).ok_or_else(|| {
        Failure::out_of_range(&format!(
            "PLib_JacobiPolynomial: {name} index {index} beyond {} tabulated values",
            table.len()
        ))
    })
}

/// Jacobi basis of degree <= 30 used by approximation algorithms (`PLib_JacobiPolynomial`);
/// `JacobiPolynomial::new(10, Shape::C0)?.work_degree() == 10`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JacobiPolynomial {
    work_degree: i32,
    niv_constr: i32,
    degree: i32,
}

impl JacobiPolynomial {
    /// Validates WorkDegree - 2*(NivConstr+1) in 0..=30 for C0, C1 or C2;
    /// `JacobiPolynomial::new(10, Shape::C1)?`.
    pub fn new(work_degree: i32, constraint_order: Shape) -> Result<Self, Failure> {
        let niv_constr: i32 = niv_constr(constraint_order)?;
        let degree: i32 = work_degree - 2 * (niv_constr + 1);
        if degree < 0 {
            return Err(failure(
                "Standard_ConstructionError",
                &format!(
                    "WorkDegree too small for given ConstraintOrder (WorkDegree = {work_degree})"
                ),
            ));
        }
        if degree > MAX_DEGREE {
            return Err(failure(
                "Standard_ConstructionError",
                &format!("Invalid Degree ({degree}; expected <= 30)"),
            ));
        }
        Ok(Self {
            work_degree,
            niv_constr,
            degree,
        })
    }

    /// Returns WorkDegree; `jacobi.work_degree()`.
    pub fn work_degree(&self) -> i32 {
        self.work_degree
    }

    /// Returns NivConstr; `jacobi.niv_constr()`.
    pub fn niv_constr(&self) -> i32 {
        self.niv_constr
    }

    fn level(&self) -> usize {
        self.niv_constr as usize
    }

    /// Writes the positive Legendre roots by increasing value into TabPoints(1..n/2)
    /// and 0 (odd n) or -999 (even n) into TabPoints(0) (`PLib_JacobiPolynomial::Points`);
    /// `jacobi.points(15, &mut points)?`.
    pub fn points(
        &self,
        nb_gauss_points: i32,
        tab_points: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        if !GAUSS_COUNTS.contains(&nb_gauss_points) || nb_gauss_points <= self.degree {
            return Err(failure(
                "Standard_ConstructionError",
                &format!(
                    "Invalid NbGaussPoints ({nb_gauss_points}; expected a tabulated count > {})",
                    self.degree
                ),
            ));
        }
        let mut decreasing: Vector<'static> = Vector::with_bounds(1, nb_gauss_points)?;
        gauss_points(nb_gauss_points, &mut decreasing);
        let half: i32 = nb_gauss_points / 2;
        for i in 1..=half {
            *array_cell(tab_points, i)? = decreasing[half - i + 1];
        }
        *array_cell(tab_points, 0)? = if nb_gauss_points % 2 == 1 {
            0.0
        } else {
            INVALID_VALUE
        };
        Ok(())
    }

    /// Loads the Gauss weights of the basis into TabWeights(0..n/2, 0..Degree)
    /// (`PLib_JacobiPolynomial::Weights`).
    ///
    /// The whole array is zeroed first; row 0 holds the odd-count central weights
    /// (even columns only) or -999. The count is not validated, as natively;
    /// `jacobi.weights(15, &mut weights)?`.
    pub fn weights(
        &self,
        nb_gauss_points: i32,
        tab_weights: &mut Array2<'_, f64>,
    ) -> Result<(), Failure> {
        tab_weights.init(0.0);
        let table: &ChunkedTable = WEIGHTS_DB[self.level()];
        let mut index: usize = self.weights_offset(nb_gauss_points);
        for j in 0..=self.degree {
            for i in 1..=nb_gauss_points / 2 {
                *grid_cell(tab_weights, i, j)? = table_value(table, index, "WeightsDB")?;
                index += 1;
            }
        }
        if nb_gauss_points % 2 == 1 {
            return self.central_weights(nb_gauss_points, tab_weights);
        }
        for j in 0..=self.degree {
            *grid_cell(tab_weights, 0, j)? = INVALID_VALUE;
        }
        Ok(())
    }

    // Skips the blocks of every smaller tabulated count, with the native integer formulas.
    fn weights_offset(&self, nb_gauss_points: i32) -> usize {
        let min_degree: i32 = 2 * (self.niv_constr + 1);
        let mut offset: i32 = 0;
        for &count in &GAUSS_COUNTS[..8] {
            if nb_gauss_points <= count {
                break;
            }
            offset += match count {
                15 | 25 => ((count - 1) / 2) * (count - min_degree),
                _ => count * (count - min_degree) / 2,
            };
        }
        offset as usize
    }

    fn central_weights(
        &self,
        nb_gauss_points: i32,
        tab_weights: &mut Array2<'_, f64>,
    ) -> Result<(), Failure> {
        let min_degree: i32 = 2 * (self.niv_constr + 1);
        let mut index: i32 = 0;
        for count in [15, 25] {
            if nb_gauss_points > count {
                index += (count - 1 - min_degree) / 2 + 1;
            }
        }
        let table: &ChunkedTable = WEIGHTS_DB0[self.level()];
        for j in (0..=self.degree).step_by(2) {
            *grid_cell(tab_weights, 0, j)? = table_value(table, index as usize, "WeightsDB0")?;
            index += 1;
        }
        Ok(())
    }

    /// Loads max |W(t) J_k(t)| on [-1,1] for consecutive k into TabMax
    /// (`PLib_JacobiPolynomial::MaxValue`); `jacobi.max_value(&mut maxima)?`.
    pub fn max_value(&self, tab_max: &mut Array1<'_, f64>) -> Result<(), Failure> {
        let table: &ChunkedTable = MAX_VALUES_DB[self.level()];
        for (index, i) in (tab_max.lower()..=tab_max.upper()).enumerate() {
            *array_cell(tab_max, i)? = table_value(table, index, "MaxValuesDB")?;
        }
        Ok(())
    }
}
