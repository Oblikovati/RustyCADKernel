// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_Function.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The gradient of the Uzawa correction in the constrained `AppParCurves_Function::
//! Perform`: the derivative of the corrected poles along each parameter.
use super::Function;
use super::constrained::PoleColumns;
use crate::app_par_curves::constraint::Constraint;
use crate::app_par_curves::line_tool::LineTool;
use crate::app_par_curves::resol_constraint::{ResolConstraint, couples};
use crate::native_failure::array_failure;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Matrix, Vector};

/// The matrices of the correction: KK = tK CTCinv, K, TrA, TrDA and
/// RESTM = (tA A)^-1, and the running DERR.
struct Correction {
    k: Matrix<'static>,
    kk: Matrix<'static>,
    tr_a: Matrix<'static>,
    tr_da: Matrix<'static>,
    restm: Matrix<'static>,
    derr: Vector<'static>,
}

// `Col(ci)` of a pole matrix.
fn column(m: &Matrix<'_>, ci: i32) -> Result<Vector<'static>, Failure> {
    m.col(ci)
}

impl<L: LineTool + Clone> Function<L> {
    // DKPTC: DK times the corrected poles; the native loop resets DKPTC(i) for each
    // curve, so only the last curve's sum is kept.
    fn dk_times_poles(
        &self,
        dk: &Matrix<'_>,
        rows: i32,
        corrected: &PoleColumns,
    ) -> Result<Vector<'static>, Failure> {
        let npol: i32 = self.degre + 1;
        let mut dkptc: Vector<'static> = Vector::filled(1, rows, 0.)?;
        for i in 1..=rows {
            let mut inc: i32 = 0;
            for ci in 1..=self.nb_cu {
                dkptc[i] = 0.;
                for j in 1..=npol {
                    dkptc[i] += *dk.value(i, j + inc)? * corrected[0][(j, ci)]
                        + *dk.value(i, j + inc + npol)? * corrected[1][(j, ci)];
                    if self.is_3d(ci) {
                        dkptc[i] += *dk.value(i, j + inc + 2 * npol)? * corrected[2][(j, ci)];
                    }
                }
                inc += if self.is_3d(ci) { 3 * npol } else { 2 * npol };
            }
        }
        Ok(dkptc)
    }

    // The matrices of the correction and DERR = tDK Vardua - KK (DKPTC + K tDK Vardua).
    fn correction(
        &self,
        resol: &ResolConstraint,
        dk: &Matrix<'static>,
        corrected: &PoleColumns,
    ) -> Result<Correction, Failure> {
        let k: Matrix<'static> = resol.constraint_matrix().clone();
        let vardua: &Vector<'static> = resol.duale();
        let kk: Matrix<'static> = k.transposed().multiplied_matrix(resol.inverse_matrix())?;
        let dtk: Matrix<'static> = dk.transposed();
        let restm: Matrix<'static> = self.a.transposed().multiplied_matrix(&self.a)?.inverse()?;
        let dkptc: Vector<'static> = self.dk_times_poles(dk, k.row_number(), corrected)?;
        let inner: Vector<'static> =
            dkptc.added(&k.multiplied_matrix(&dtk)?.multiplied_vector(vardua)?)?;
        let derr: Vector<'static> = dtk
            .multiplied_vector(vardua)?
            .subtracted(&kk.multiplied_vector(&inner)?)?;
        Ok(Correction {
            k,
            kk,
            tr_a: self.a.transposed(),
            tr_da: self.da.transposed(),
            restm,
            derr,
        })
    }

    /// The correction terms of the gradient: the derivative of the fit's poles
    /// (DPTCO1) and of the Uzawa correction (DERR), point by point.
    pub(super) fn correction_gradient(
        &mut self,
        resol: &ResolConstraint,
        dk: &Matrix<'static>,
        [original, corrected]: [&PoleColumns; 2],
    ) -> Result<(), Failure> {
        let mut correction: Correction = self.correction(resol, dk, corrected)?;
        let columns: i32 = correction.k.col_number();
        let mut dptco1: Matrix<'static> = Matrix::filled(self.first_p, self.last_p, 1, columns, 0.)
            .map_err(|error| array_failure(error, "AppParCurves_Function"))?;
        let mut inc: i32 = 0;
        for ci in 1..=self.nb_cu {
            self.projection_terms(ci, inc, &correction, [original, corrected], &mut dptco1)?;
            inc += if self.is_3d(ci) {
                3 * self.npol()
            } else {
                2 * self.npol()
            };
        }
        let mut cons: Constraint = Constraint::NoConstraint;
        for pi in self.first_p..=self.last_p {
            if let Some(couple) = couples(&self.constraints)?
                .into_iter()
                .find(|couple| couple.index() == pi)
            {
                cons = couple.constraint();
            }
            if cons >= Constraint::PassPoint {
                self.pass_point_terms(pi, &mut correction, [original, corrected], &dptco1)?;
            }
        }
        Ok(())
    }

    fn npol(&self) -> i32 {
        self.degre + 1
    }

    // DPTCO1(Pi, :) = tDA(:, Pi) (A PTCO - PTL)(Pi) + tA(:, Pi) (DA PTCO)(Pi) for curve
    // Ci, and ValGrad_F(Pi) -= 2 sum(DPTCO1 Taa (PTC - PTCO)).
    fn projection_terms(
        &mut self,
        ci: i32,
        inc: i32,
        c: &Correction,
        [original, corrected]: [&PoleColumns; 2],
        dptco1: &mut Matrix<'_>,
    ) -> Result<(), Failure> {
        let (npol, width) = (self.npol(), if self.is_3d(ci) { 3 } else { 2 });
        let mut err: Vec<Vector<'static>> = Vec::new();
        let mut scal: Vec<Vector<'static>> = Vec::new();
        let mut erruza: Vec<Vector<'static>> = Vec::new();
        for coordinate in 0..3 {
            let ptco: Vector<'static> = column(&original[coordinate], ci)?;
            err.push(
                self.a
                    .multiplied_vector(&ptco)?
                    .subtracted(&column(&self.targets.ptl[coordinate], ci)?)?,
            );
            scal.push(self.da.multiplied_vector(&ptco)?);
            erruza.push(column(&corrected[coordinate], ci)?.subtracted(&ptco)?);
        }
        for pi in self.first_p..=self.last_p {
            let (tr_dapi, tr_api) = (c.tr_da.col(pi)?, c.tr_a.col(pi)?);
            let taa: f64 = tr_api.dot(&self.a.row(pi)?)?;
            let mut sum: f64 = 0.;
            for j in 1..=npol {
                let mut term = |coordinate: usize| -> Result<f64, Failure> {
                    let value: f64 =
                        tr_dapi[j] * err[coordinate][pi] + tr_api[j] * scal[coordinate][pi];
                    *dptco1.change_value(pi, j + inc + coordinate as i32 * npol)? = value;
                    Ok(value * taa * erruza[coordinate][j])
                };
                let (x, y) = (term(0)?, term(1)?);
                sum += x + y;
                if width == 3 {
                    sum += term(2)?;
                }
            }
            *self.val_grad_f.change_value(pi)? -= 2. * sum;
        }
        Ok(())
    }

    // For a constrained point Pi: DPTCO = -RESTM DPTCO1 (with the native j-for-k
    // index), DERR -= KK K DPTCO, and ValGrad_F(Pi) += 2 Scal per curve, where Scal
    // keeps the last j only (the native `=`) and uses the X error for Z.
    fn pass_point_terms(
        &mut self,
        pi: i32,
        c: &mut Correction,
        [original, corrected]: [&PoleColumns; 2],
        dptco1: &Matrix<'_>,
    ) -> Result<(), Failure> {
        let npol: i32 = self.npol();
        let mut dptco: Vector<'static> = Vector::filled(1, c.k.col_number(), 0.)?;
        let mut inc: i32 = 0;
        for ci in 1..=self.nb_cu {
            let width: i32 = if self.is_3d(ci) { 3 } else { 2 };
            for j in 1..=npol {
                for coordinate in 0..width {
                    let slot: i32 = j + inc + coordinate * npol;
                    dptco[slot] = 0.;
                    for k in 1..=npol {
                        dptco[slot] -= c.restm[(j, k)] * *dptco1.value(pi, slot)?;
                    }
                }
            }
            inc += width * npol;
        }
        c.derr = c
            .derr
            .subtracted(&c.kk.multiplied_matrix(&c.k)?.multiplied_vector(&dptco)?)?;
        let mut inc: i32 = 0;
        for ci in 1..=self.nb_cu {
            let three: bool = self.is_3d(ci);
            let ex: Vector<'static> =
                column(&corrected[0], ci)?.subtracted(&column(&original[0], ci)?)?;
            let ey: Vector<'static> =
                column(&corrected[1], ci)?.subtracted(&column(&original[1], ci)?)?;
            let mut scal: f64 = 0.;
            for j in 1..=npol {
                let a: f64 = self.a[(pi, j)];
                scal = (a * ex[j]) * (a * c.derr[j + inc])
                    + (a * ey[j]) * (a * c.derr[j + inc + npol]);
                if three {
                    scal += (a * ex[j]) * (a * c.derr[j + inc + 2 * npol]);
                }
            }
            *self.val_grad_f.change_value(pi)? += 2. * scal;
            inc += if three { 3 * npol } else { 2 * npol };
        }
        Ok(())
    }
}
