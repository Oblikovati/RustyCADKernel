// Rust translation of ModelingData/TKG3d/AdvApprox/AdvApprox_SimpleApprox.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AdvApprox_SimpleApprox`: one polynomial approximation of a function on one
//! interval, in the Jacobi basis, with Hermite constraints at both ends.
//!
//! The polynomial is `P(t) = R(t) + W(t) Q(t)` on `[-1, 1]`: `R` interpolates the
//! end values and derivatives, `Q` is the Gauss-quadrature projection of the rest.
use super::evaluator::AdvApproxEvaluatorFunction;
use crate::geom::errors;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::at;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::plib::{self, JacobiPolynomial};
use std::io;

/// The approximator of one span (`AdvApprox_SimpleApprox`); it keeps the evaluator
/// borrowed, as the native object keeps its pointer;
/// `AdvApproxSimpleApprox::new(3, 1, Shape::C1, 20, 25, &jacobi, &mut f)?`.
pub struct AdvApproxSimpleApprox<'e> {
    total_num_ss: i32,
    total_dimension: i32,
    nb_gauss_points: i32,
    work_degree: i32,
    niv_constr: i32,
    jac_pol: JacobiPolynomial,
    tab_points: Array1<'static, f64>,
    tab_weights: Array2<'static, f64>,
    evaluator: &'e mut dyn AdvApproxEvaluatorFunction,
    degree: i32,
    coeff: Array1<'static, f64>,
    first_constr: Array2<'static, f64>,
    last_constr: Array2<'static, f64>,
    som_tab: Array1<'static, f64>,
    dif_tab: Array1<'static, f64>,
    max_error: Option<Array1<'static, f64>>,
    average_error: Option<Array1<'static, f64>>,
    done: bool,
}

// The constraint order of a continuity; Standard_ConstructionError beyond C2.
fn constraint_level(continuity: Shape) -> Result<i32, Failure> {
    match continuity {
        Shape::C0 => Ok(0),
        Shape::C1 => Ok(1),
        Shape::C2 => Ok(2),
        other => Err(errors::construction(&format!(
            "Invalid Continuity: {other:?}; expected C0, C1 or C2"
        ))),
    }
}

// A zero-filled array over `lower..=upper` with the native bound error.
fn array(lower: i32, upper: i32) -> Result<Array1<'static, f64>, Failure> {
    Array1::with_bounds(lower, upper).map_err(errors::array_bounds)
}

// A zero-filled grid over the given bounds with the native bound error.
fn grid(rows: (i32, i32), cols: (i32, i32)) -> Result<Array2<'static, f64>, Failure> {
    Array2::with_bounds(rows.0, rows.1, cols.0, cols.1).map_err(errors::array_bounds)
}

impl<'e> AdvApproxSimpleApprox<'e> {
    /// Prepares the Gauss points and weights of `jacobi_base`; Standard_ConstructionError
    /// for a continuity beyond C2 or an untabulated point count
    /// (`AdvApprox_SimpleApprox(TotalDimension, TotalNumSS, Continuity, WorkDegree,
    /// NbGaussPoints, JacobiBase, Func)`);
    /// `AdvApproxSimpleApprox::new(3, 1, Shape::C1, 20, 25, &jacobi, &mut f)?`.
    pub fn new(
        total_dimension: i32,
        total_num_ss: i32,
        continuity: Shape,
        work_degree: i32,
        nb_gauss_points: i32,
        jacobi_base: &JacobiPolynomial,
        func: &'e mut dyn AdvApproxEvaluatorFunction,
    ) -> Result<Self, Failure> {
        let niv_constr: i32 = constraint_level(continuity)?;
        let degree_q: i32 = work_degree - 2 * (niv_constr + 1);
        let half: i32 = nb_gauss_points / 2;
        let mut tab_points = array(0, half)?;
        jacobi_base.points(nb_gauss_points, &mut tab_points)?;
        let mut tab_weights = grid((0, half), (0, degree_q))?;
        jacobi_base.weights(nb_gauss_points, &mut tab_weights)?;
        let tables: i32 = (half + 1) * total_dimension - 1;
        Ok(Self {
            total_num_ss,
            total_dimension,
            nb_gauss_points,
            work_degree,
            niv_constr,
            jac_pol: *jacobi_base,
            tab_points,
            tab_weights,
            evaluator: func,
            degree: 0,
            coeff: array(0, (work_degree + 1) * total_dimension - 1)?,
            first_constr: grid((1, total_dimension), (0, niv_constr))?,
            last_constr: grid((1, total_dimension), (0, niv_constr))?,
            som_tab: array(0, tables)?,
            dif_tab: array(0, tables)?,
            max_error: None,
            average_error: None,
            done: false,
        })
    }

    /// Approximates the function on `[first, last]`, reducing each subspace to at most
    /// `max_degree` within its tolerance. An evaluator error leaves `is_done()` false
    /// (`Perform`); `approx.perform(&dims, &tolerances, 0.0, 1.0, 13)?`.
    pub fn perform(
        &mut self,
        local_dimension: &Array1<'_, i32>,
        local_tolerances: &Array1<'_, f64>,
        first: f64,
        last: f64,
        max_degree: i32,
    ) -> Result<(), Failure> {
        self.done = false;
        let first_last: [f64; 2] = [first, last];
        if !self.hermite_part(first_last)? || !self.gauss_sums(first_last)? {
            return Ok(());
        }
        self.q_coefficients()?;
        self.reduce(local_dimension, local_tolerances, max_degree)?;
        self.done = true;
        Ok(())
    }

    // Evaluates the function; false when the evaluator reports an error.
    fn evaluate(&mut self, first_last: [f64; 2], t: f64, derive: i32, out: &mut [f64]) -> bool {
        let dimension: i32 = self.total_dimension;
        self.evaluator
            .evaluate(dimension, first_last, t, derive, out)
            == 0
    }

    // The end constraints, scaled to [-1, 1], and their Hermite interpolant R(t).
    fn hermite_part(&mut self, first_last: [f64; 2]) -> Result<bool, Failure> {
        let mut result: Vec<f64> = vec![0.0; self.total_dimension as usize];
        for (param, at_last) in [(first_last[0], false), (first_last[1], true)] {
            if !self.end_constraints(first_last, param, at_last, &mut result)? {
                return Ok(false);
            }
        }
        plib::hermite_interpolate(
            self.total_dimension,
            -1.,
            1.,
            self.niv_constr,
            self.niv_constr,
            &self.first_constr,
            &self.last_constr,
            &mut self.coeff,
        )?;
        Ok(true)
    }

    fn end_constraints(
        &mut self,
        first_last: [f64; 2],
        param: f64,
        at_last: bool,
        result: &mut [f64],
    ) -> Result<bool, Failure> {
        let fact: f64 = (first_last[1] - first_last[0]) / 2.;
        for derive in (0..=self.niv_constr).rev() {
            if !self.evaluate(first_last, param, derive, result) {
                return Ok(false);
            }
            if derive >= 1 {
                result.iter_mut().for_each(|value| *value *= fact);
            }
            if derive == 2 {
                result.iter_mut().for_each(|value| *value *= fact);
            }
            let target = if at_last {
                &mut self.last_constr
            } else {
                &mut self.first_constr
            };
            for (idim, &value) in (1..).zip(result.iter()) {
                target
                    .set_value(idim, derive, value)
                    .map_err(errors::array_bounds)?;
            }
        }
        Ok(true)
    }

    // F(t_i) + F(-t_i) - R(t_i) - R(-t_i) into SomTab and the differences into DifTab.
    fn gauss_sums(&mut self, first_last: [f64; 2]) -> Result<bool, Failure> {
        let td: usize = self.total_dimension as usize;
        let mut fti: Vec<f64> = vec![0.0; td];
        let (alin, blin) = (
            (first_last[1] - first_last[0]) / 2.,
            (first_last[1] + first_last[0]) / 2.,
        );
        let half: i32 = self.nb_gauss_points / 2;
        for i in 1..=half {
            let tip: f64 = alin * at(&self.tab_points, i)? + blin;
            if !self.evaluate(first_last, tip, 0, &mut fti) {
                return Ok(false);
            }
            let start: usize = i as usize * td;
            self.som_tab.as_mut_slice()[start..start + td].copy_from_slice(&fti);
            self.dif_tab.as_mut_slice()[start..start + td].copy_from_slice(&fti);
        }
        for i in 1..=half {
            if !self.negative_side(first_last, (alin, blin), i, &mut fti)? {
                return Ok(false);
            }
        }
        if self.nb_gauss_points % 2 == 1 {
            return self.center_value(first_last, blin, &mut fti);
        }
        Ok(true)
    }

    // R(t) at the Gauss abscissa `ti`, degree 2 * NivConstr + 1.
    fn hermite_value(&self, ti: f64) -> Result<Vec<f64>, Failure> {
        let mut values: Vec<f64> = vec![0.0; self.total_dimension as usize];
        let degree_r: i32 = 2 * self.niv_constr + 1;
        let coefficients: &[f64] = self.coeff.as_slice();
        plib::eval_polynomial(
            ti,
            0,
            degree_r,
            self.total_dimension,
            coefficients,
            &mut values,
        )?;
        Ok(values)
    }

    fn negative_side(
        &mut self,
        first_last: [f64; 2],
        (alin, blin): (f64, f64),
        i: i32,
        fti: &mut [f64],
    ) -> Result<bool, Failure> {
        let ti: f64 = at(&self.tab_points, i)?;
        let tin: f64 = -alin * ti + blin;
        if !self.evaluate(first_last, tin, 0, fti) {
            return Ok(false);
        }
        let (rpti, rmti) = (self.hermite_value(ti)?, self.hermite_value(-ti)?);
        let start: usize = i as usize * fti.len();
        let som: &mut [f64] = self.som_tab.as_mut_slice();
        let dif: &mut [f64] = self.dif_tab.as_mut_slice();
        for idim in 0..fti.len() {
            som[start + idim] = som[start + idim] + fti[idim] - rpti[idim] - rmti[idim];
            dif[start + idim] = dif[start + idim] - fti[idim] - rpti[idim] + rmti[idim];
        }
        Ok(true)
    }

    // For an odd point count, F(0) - R(0) at the head of both tables.
    fn center_value(
        &mut self,
        first_last: [f64; 2],
        blin: f64,
        fti: &mut [f64],
    ) -> Result<bool, Failure> {
        let ti: f64 = at(&self.tab_points, 0)?;
        if !self.evaluate(first_last, blin, 0, fti) {
            return Ok(false);
        }
        let rpti: Vec<f64> = self.hermite_value(ti)?;
        for idim in 0..fti.len() {
            self.som_tab.as_mut_slice()[idim] = fti[idim] - rpti[idim];
            self.dif_tab.as_mut_slice()[idim] = fti[idim] - rpti[idim];
        }
        Ok(true)
    }

    // The Gauss projections of Q(t). Quirk kept: in the odd-count branch the native
    // running sum is neither reset per dimension nor per degree, and it replaces the
    // even coefficients instead of adding the central term to them.
    fn q_coefficients(&mut self) -> Result<(), Failure> {
        let degree_r: i32 = 2 * self.niv_constr + 1;
        let degree_q: i32 = self.work_degree - 2 * (self.niv_constr + 1);
        let mut sum: f64 = 0.;
        for parity in [0, 1] {
            for k in (parity..=degree_q).step_by(2) {
                for idim in 1..=self.total_dimension {
                    sum = self.projection(k, idim, parity == 1)?;
                    self.set_coefficient(k + degree_r + 1, idim, sum)?;
                }
            }
        }
        if self.nb_gauss_points % 2 == 0 {
            return Ok(());
        }
        for idim in 1..=self.total_dimension {
            for k in (0..=degree_q).step_by(2) {
                let weight: f64 = *self.tab_weights.value(0, k).map_err(errors::array_bounds)?;
                sum += weight * at(&self.som_tab, idim - 1)?;
                self.set_coefficient(k + degree_r + 1, idim, sum)?;
            }
        }
        Ok(())
    }

    fn projection(&self, k: i32, idim: i32, odd: bool) -> Result<f64, Failure> {
        let table: &Array1<'static, f64> = if odd { &self.dif_tab } else { &self.som_tab };
        let mut sum: f64 = 0.;
        for i in 1..=self.nb_gauss_points / 2 {
            let weight: f64 = *self.tab_weights.value(i, k).map_err(errors::array_bounds)?;
            sum += weight * at(table, i * self.total_dimension + idim - 1)?;
        }
        Ok(sum)
    }

    fn set_coefficient(&mut self, degree: i32, idim: i32, value: f64) -> Result<(), Failure> {
        let index: i32 = degree * self.total_dimension + idim - 1;
        self.coeff
            .set_value(index, value)
            .map_err(errors::array_bounds)
    }

    // Regroups the coefficients per subspace, reduces each one, then bounds the
    // errors at the largest reduced degree.
    fn reduce(
        &mut self,
        local_dimension: &Array1<'_, i32>,
        local_tolerances: &Array1<'_, f64>,
        max_degree: i32,
    ) -> Result<(), Failure> {
        let jac_coeff: Vec<f64> = self.per_subspace(local_dimension)?;
        self.max_error = Some(array(1, self.total_num_ss)?);
        self.average_error = Some(array(1, self.total_num_ss)?);
        let mut new_degree_max: i32 = 0;
        let mut rang: usize = 0;
        for numss in 1..=self.total_num_ss {
            let dim: i32 = at(local_dimension, numss)?;
            let tolerance: f64 = at(local_tolerances, numss)?;
            let (new_degree, _) =
                self.jac_pol
                    .reduce_degree(dim, max_degree, tolerance, &jac_coeff[rang..])?;
            new_degree_max = new_degree_max.max(new_degree);
            rang += ((self.work_degree + 1) * dim) as usize;
        }
        self.store_errors(local_dimension, &jac_coeff, new_degree_max)?;
        self.degree = new_degree_max;
        Ok(())
    }

    fn per_subspace(&self, local_dimension: &Array1<'_, i32>) -> Result<Vec<f64>, Failure> {
        let td: i32 = self.total_dimension;
        let mut jac_coeff: Vec<f64> = vec![0.0; (td * (self.work_degree + 1)) as usize];
        let (mut rang_ss, mut rang_jac) = (0, 0);
        for numss in 1..=self.total_num_ss {
            let dim: i32 = at(local_dimension, numss)?;
            let (mut rang_coeff, mut rang_dim) = (0, 0);
            for _ in 0..=self.work_degree {
                for idim in 1..=dim {
                    let value: f64 = at(&self.coeff, rang_coeff + rang_ss + idim - 1)?;
                    let slot = jac_coeff.get_mut((rang_jac + rang_dim + idim - 1) as usize);
                    *slot.ok_or_else(|| overflow("JacCoeff", rang_jac + rang_dim + idim - 1))? =
                        value;
                }
                rang_dim += dim;
                rang_coeff += td;
            }
            rang_ss += dim;
            rang_jac += (self.work_degree + 1) * dim;
        }
        Ok(jac_coeff)
    }

    fn store_errors(
        &mut self,
        local_dimension: &Array1<'_, i32>,
        jac_coeff: &[f64],
        new_degree_max: i32,
    ) -> Result<(), Failure> {
        let mut rang: usize = 0;
        for numss in 1..=self.total_num_ss {
            let dim: i32 = at(local_dimension, numss)?;
            let jac_ss: &[f64] = jac_coeff.get(rang..).unwrap_or(&[]);
            let max_err: f64 = self.jac_pol.max_error(dim, jac_ss, new_degree_max)?;
            let average: f64 = self.jac_pol.average_error(dim, jac_ss, new_degree_max)?;
            put_error(&mut self.max_error, numss, max_err)?;
            put_error(&mut self.average_error, numss, average)?;
            rang += ((self.work_degree + 1) * dim) as usize;
        }
        Ok(())
    }

    /// Whether the last `perform` completed (`IsDone`); `approx.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// The largest reduced degree of the last successful `perform` (`Degree`); `approx.degree()`.
    pub fn degree(&self) -> i32 {
        self.degree
    }
    /// The Jacobi coefficients, `(WorkDegree + 1) * TotalDimension` values degree by
    /// degree, from index 0 (`Coefficients`); `approx.coefficients()`.
    pub fn coefficients(&self) -> &Array1<'static, f64> {
        &self.coeff
    }
    /// The scaled constraints at the first parameter, `(1..TotalDimension, 0..NivConstr)`
    /// (`FirstConstr`); `approx.first_constr()`.
    pub fn first_constr(&self) -> &Array2<'static, f64> {
        &self.first_constr
    }
    /// The scaled constraints at the last parameter (`LastConstr`); `approx.last_constr()`.
    pub fn last_constr(&self) -> &Array2<'static, f64> {
        &self.last_constr
    }
    /// The Gauss sums table (`SomTab`); `approx.som_tab()`.
    pub fn som_tab(&self) -> &Array1<'static, f64> {
        &self.som_tab
    }
    /// The Gauss differences table (`DifTab`); `approx.dif_tab()`.
    pub fn dif_tab(&self) -> &Array1<'static, f64> {
        &self.dif_tab
    }
    /// Maximum error of subspace `index`; Standard_NullObject before a completed
    /// reduction, Standard_OutOfRange for a bad index (`MaxError`); `approx.max_error(1)?`.
    pub fn max_error(&self, index: i32) -> Result<f64, Failure> {
        read_error(&self.max_error, index, "MaxError")
    }
    /// Average error of subspace `index` (`AverageError`); `approx.average_error(1)?`.
    pub fn average_error(&self, index: i32) -> Result<f64, Failure> {
        read_error(&self.average_error, index, "AverageError")
    }
    /// Writes the native text: a title and one `Error   <max>` line per subspace (`Dump`);
    /// `approx.dump(&mut std::io::stdout())?`.
    pub fn dump(&self, output: &mut impl io::Write) -> io::Result<()> {
        writeln!(output, "Dump of SimpleApprox ")?;
        for ii in 1..=self.total_num_ss {
            let value: f64 = self.max_error(ii).map_err(io::Error::other)?;
            writeln!(output, "Error   {}", super::stream_real(value))?;
        }
        Ok(())
    }
}

fn overflow(name: &str, index: i32) -> Failure {
    errors::out_of_range(&format!(
        "AdvApprox_SimpleApprox: {name} index {index} out of range"
    ))
}

fn put_error(
    table: &mut Option<Array1<'static, f64>>,
    index: i32,
    value: f64,
) -> Result<(), Failure> {
    match table {
        Some(values) => values.set_value(index, value).map_err(errors::array_bounds),
        None => Err(overflow("error table", index)),
    }
}

fn read_error(
    table: &Option<Array1<'static, f64>>,
    index: i32,
    member: &str,
) -> Result<f64, Failure> {
    match table {
        Some(values) => at(values, index),
        None => Err(Failure::with_type(
            "Standard_NullObject",
            &format!(
                "AdvApprox_SimpleApprox::{member}: no error table; expected a performed approximation"
            ),
            "",
        )),
    }
}
