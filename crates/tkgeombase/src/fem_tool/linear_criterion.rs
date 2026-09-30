// Rust translation of ModelingData/TKGeomBase/FEmTool/FEmTool_LinearFlexion.cxx, FEmTool_LinearJerk.cxx, FEmTool_LinearTension.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The shared implementation of `FEmTool_LinearFlexion`, `FEmTool_LinearJerk` and
//! `FEmTool_LinearTension`: the three native files are one algorithm with a
//! different derivative order, scale power and three small divergences, which
//! [`LinearKind`] names.
use super::elementary_criterion::{CriterionBase, ElementaryCriterion, coefficients};
use super::reference_matrix::reference_matrix;
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::{Array2, HArray2};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::math::{Matrix, Vector};
use rustycad_tkmath::plib;
use std::marker::PhantomData;

/// Where a criterion clamps a negative J to 0 in `Value()`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NegativeClamp {
    /// Once, after the whole sum (LinearFlexion).
    AfterSum,
    /// After every row term (LinearJerk): a negative partial sum restarts at 0.
    EachRow,
    /// Never (LinearTension).
    Never,
}

/// The compile-time differences between the three linear criteria.
pub trait LinearKind: Send + Sync + 'static {
    /// The native class name, used in exception messages.
    const NAME: &'static str;
    /// The basis derivative order integrated into the reference matrix.
    const DER_ORDER: i32;
    /// Where `Value()` clamps a negative J.
    const CLAMP: NegativeClamp;
    /// LinearTension's DependenceTable sets `(i, i)` for `i = 1..=RowLength` instead
    /// of `LowerCol..=UpperCol`.
    const ONE_BASED_TABLE: bool;
    /// LinearJerk and LinearTension write the Hessian from `H.LowerRow()` and
    /// `H.LowerCol()`; LinearFlexion writes from index 0.
    const HESSIAN_FROM_LOWER: bool;
    /// `cteh3`, the scale of J for the half element length `coeff`.
    fn scale(coeff: f64) -> f64;
    /// The process-wide reference cache of the native function-local statics.
    fn cache() -> &'static std::sync::Mutex<super::reference_matrix::ReferenceCache>;
}

/// A linear FEmTool criterion; see [`super::LinearFlexion`], [`super::LinearJerk`] and
/// [`super::LinearTension`].
pub struct LinearCriterion<K: LinearKind> {
    pub(super) parent: StandardTransient,
    base: CriterionBase,
    ref_matrix: Matrix<'static>,
    order: i32,
    kind: PhantomData<K>,
}

impl<K: LinearKind> LinearCriterion<K> {
    /// The native constructor `(WorkDegree, ConstraintOrder)`: Standard_ConstructionError
    /// for a constraint order other than C0, C1 or C2, and for `WorkDegree > 14` when
    /// the process-wide reference matrix must be recomputed (a cached one is
    /// reused without the check); `LinearFlexion::new(10, Shape::C1)?`.
    pub fn new(work_degree: i32, constraint_order: Shape) -> Result<Self, Failure> {
        let mut ref_matrix: Matrix<'static> = Matrix::with_bounds(0, work_degree, 0, work_degree)
            .map_err(|error| array_failure(error, K::NAME))?;
        let order: i32 = plib::niv_constr(constraint_order)?;
        reference_matrix::<K>(order, constraint_order, work_degree, &mut ref_matrix)?;
        Ok(Self {
            parent: StandardTransient::default(),
            base: CriterionBase::new(),
            ref_matrix,
            order,
            kind: PhantomData,
        })
    }

    // J of one dimension row of NewCoeff: sum over i of (R(i,i)/2 c_i + sum_{j<i} R(i,j) c_j) c_i.
    fn dimension_sum(&self, new_coeff: &Array2<'_, f64>, dim: i32, mut j: f64) -> f64 {
        let deg: i32 = new_coeff.upper_col();
        for i in 0..=deg {
            let mut line: f64 = 0.5 * self.ref_matrix[(i, i)] * new_coeff[(dim, i)];
            for jj in 0..i {
                line += self.ref_matrix[(i, jj)] * new_coeff[(dim, jj)];
            }
            j += line * new_coeff[(dim, i)];
            if K::CLAMP == NegativeClamp::EachRow && j < 0.0 {
                j = 0.0;
            }
        }
        j
    }

    // NewCoeff(dim, i): the element coefficients with the Hermite rows scaled by
    // coeff^k1 (k1 the derivative order of the Hermite function).
    fn scaled_coefficients(
        &self,
        coeff: &Array2<'_, f64>,
        half: f64,
    ) -> Result<Array2<'static, f64>, Failure> {
        let deg: i32 = (coeff.nb_rows() as i32 - 1).min(self.ref_matrix.upper_row());
        let (j0, nb_dim) = (coeff.lower_row(), coeff.nb_columns() as i32);
        let deg_h: i32 = (2 * self.order + 1).min(deg);
        let mut scaled: Array2<'static, f64> = Array2::with_bounds(1, nb_dim, 0, deg)
            .map_err(|error| array_failure(error, K::NAME))?;
        for i in 0..=deg {
            let factor: Option<f64> = (i <= deg_h).then(|| half.powf(self.hermite_order(i) as f64));
            for dim in 1..=nb_dim {
                let value: f64 = *coeff
                    .value(j0 + i, dim)
                    .map_err(|error| array_failure(error, K::NAME))?;
                scaled[(dim, i)] = factor.map_or(value, |factor| value * factor);
            }
        }
        Ok(scaled)
    }

    // k1: the derivative order of Hermite function i (i for the left end, i - order - 1
    // for the right end).
    fn hermite_order(&self, i: i32) -> i32 {
        if i <= self.order {
            i
        } else {
            i - self.order - 1
        }
    }

    // `(myLast - myFirst) / 2`.
    fn half_length(&self) -> f64 {
        let fields = self.base.fields();
        (fields.last - fields.first) / 2.
    }

    // Standard_DomainError before Set(Coeff); a unit diagonal over the coefficient columns.
    fn table(&self) -> Result<Handle<HArray2<i32>>, Failure> {
        let fields = self.base.fields();
        let Some(coeff) = fields.coeff.get() else {
            return Err(failure(
                "Standard_DomainError",
                &format!("{}::DependenceTable", K::NAME),
            ));
        };
        let coeff = coeff.array2();
        let (lower, upper) = (coeff.lower_col(), coeff.upper_col());
        let mut table: Array2<'static, i32> = Array2::with_bounds(lower, upper, lower, upper)
            .map_err(|error| array_failure(error, K::NAME))?;
        table.init(0);
        let diagonal = if K::ONE_BASED_TABLE {
            1..=coeff.nb_columns() as i32
        } else {
            lower..=upper
        };
        for i in diagonal {
            table
                .set_value(i, i, 1)
                .map_err(|error| array_failure(error, K::NAME))?;
        }
        Ok(Handle::new(HArray2::from_owned(table)))
    }

    // The Standard_OutOfRange / Standard_DomainError checks of Hessian.
    fn check_dimensions(&self, dimension1: i32, dimension2: i32) -> Result<(), Failure> {
        let table: Handle<HArray2<i32>> = self.table()?;
        let table = table.array2();
        let rows = table.lower_row()..=table.upper_row();
        let cols = table.lower_col()..=table.upper_col();
        if !rows.contains(&dimension1) || !cols.contains(&dimension2) {
            return Err(Failure::out_of_range(&format!(
                "{}::Hessian: dimensions ({dimension1}, {dimension2}); expected within {rows:?} x {cols:?}",
                K::NAME
            )));
        }
        if table[(dimension1, dimension2)] == 0 {
            return Err(failure(
                "Standard_DomainError",
                &format!(
                    "{}::Hessian: dimensions ({dimension1}, {dimension2}) are independent",
                    K::NAME
                ),
            ));
        }
        Ok(())
    }
}

// H(row, col) = value, with the native checked access.
fn put(h: &mut Matrix<'_>, row: i32, col: i32, value: f64) -> Result<(), Failure> {
    *h.change_value(row, col)? = value;
    Ok(())
}

impl<K: LinearKind> LinearCriterion<K> {
    // The Hermite rows of the Hessian: Hermite*Hermite then Hermite*Jacobi.
    fn hermite_rows(
        &self,
        h: &mut Matrix<'_>,
        [deg, deg_h]: [i32; 2],
        [i0, j0]: [i32; 2],
    ) -> Result<(), Failure> {
        let coeff: f64 = self.half_length();
        let cteh3: f64 = K::scale(coeff);
        for i in 0..=deg_h {
            let mfact: f64 = coeff.powf(self.hermite_order(i) as f64) * cteh3;
            for j in i..=deg_h {
                let value: f64 =
                    mfact * coeff.powf(self.hermite_order(j) as f64) * self.ref_matrix[(i, j)];
                put(h, i0 + i, j0 + j, value)?;
                if i != j {
                    put(h, j0 + j, i0 + i, value)?;
                }
            }
            // The write order matters when H's row and column origins differ: the
            // mirrored cells of different (i, j) can then coincide.
            for j in deg_h + 1..=deg {
                let value: f64 = mfact * self.ref_matrix[(i, j)];
                put(h, i0 + i, j0 + j, value)?;
                put(h, j0 + j, i0 + i, value)?;
            }
        }
        Ok(())
    }

    // The Jacobi*Jacobi block of the Hessian.
    fn jacobi_rows(
        &self,
        h: &mut Matrix<'_>,
        [deg, deg_h]: [i32; 2],
        [i0, j0]: [i32; 2],
    ) -> Result<(), Failure> {
        let cteh3: f64 = K::scale(self.half_length());
        for i in deg_h + 1..=deg {
            for j in i..=deg {
                let value: f64 = cteh3 * self.ref_matrix[(i, j)];
                put(h, i0 + i, j0 + j, value)?;
                if i != j {
                    put(h, j0 + j, i0 + i, value)?;
                }
            }
        }
        Ok(())
    }
}

impl<K: LinearKind> ElementaryCriterion for LinearCriterion<K>
where
    Self: rustycad_tkernel::standard::Transient,
{
    fn criterion(&self) -> &CriterionBase {
        &self.base
    }

    fn dependence_table(&self) -> Result<Handle<HArray2<i32>>, Failure> {
        self.table()
    }

    fn value(&self) -> Result<f64, Failure> {
        let coeff: Array2<'static, f64> = coefficients(&self.base.fields(), K::NAME)?;
        let half: f64 = self.half_length();
        let cteh3: f64 = K::scale(half);
        let new_coeff: Array2<'static, f64> = self.scaled_coefficients(&coeff, half)?;
        let mut j: f64 = 0.;
        for dim in 1..=new_coeff.upper_row() {
            j = self.dimension_sum(&new_coeff, dim, j);
        }
        if K::CLAMP == NegativeClamp::AfterSum && j < 0. {
            j = 0.;
        }
        Ok(cteh3 * j)
    }

    fn hessian(&self, dimension1: i32, dimension2: i32, h: &mut Matrix<'_>) -> Result<(), Failure> {
        self.check_dimensions(dimension1, dimension2)?;
        let deg: i32 = self.ref_matrix.upper_row().min(h.row_number() - 1);
        let deg_h: i32 = (2 * self.order + 1).min(deg);
        let origin: [i32; 2] = if K::HESSIAN_FROM_LOWER {
            [h.lower_row(), h.lower_col()]
        } else {
            [0, 0]
        };
        h.init(0.);
        self.hermite_rows(h, [deg, deg_h], origin)?;
        self.jacobi_rows(h, [deg, deg_h], origin)
    }

    fn gradient(&self, dimension: i32, g: &mut Vector<'_>) -> Result<(), Failure> {
        let coeff: Array2<'static, f64> = coefficients(&self.base.fields(), K::NAME)?;
        if dimension < coeff.lower_col() || dimension > coeff.upper_col() {
            return Err(Failure::out_of_range(&format!(
                "{}::Gradient: dimension {dimension}; expected {}..={}",
                K::NAME,
                coeff.lower_col(),
                coeff.upper_col()
            )));
        }
        let deg: i32 = (g.length() - 1).min(coeff.nb_rows() as i32 - 1);
        let mut x: Vector<'static> = Vector::with_bounds(0, deg)?;
        for i in 0..=deg {
            x[i] = *coeff
                .value(coeff.lower_row() + i, dimension)
                .map_err(|error| array_failure(error, K::NAME))?;
        }
        let mut h: Matrix<'static> =
            Matrix::with_bounds(0, deg, 0, deg).map_err(|error| array_failure(error, K::NAME))?;
        self.hessian(dimension, dimension, &mut h)?;
        g.set_matrix_vector(&h, &x)
    }
}
