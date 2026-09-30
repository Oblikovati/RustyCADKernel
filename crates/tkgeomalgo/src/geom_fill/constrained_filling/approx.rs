// Rust translation of Build, PerformApprox, Eval and the evaluator class
// GeomFill_ConstrainedFilling_Eval of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_ConstrainedFilling.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The approximation of two opposite boundaries (and their tangent fields) by one
//! B-spline function (`AdvApprox_ApproxAFunction`).
use super::{GeomFillConstrainedFilling, field, live};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::{one_based, put};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::adv_approx::AdvApproxApproxAFunction;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Vector3};

/// `Coord(res[k], res[k + 1], res[k + 2])` into the evaluator's result.
fn store(result: &mut [f64], k: usize, x: f64, y: f64, z: f64) -> Result<(), Failure> {
    let length: usize = result.len();
    let slot = result.get_mut(k..k + 3).ok_or_else(|| {
        errors::out_of_range(&format!(
            "GeomFill_ConstrainedFilling::Eval: result of {length} values; expected at least {}",
            k + 3
        ))
    })?;
    slot.copy_from_slice(&[x, y, z]);
    Ok(())
}

fn store_point(result: &mut [f64], k: usize, p: Point3) -> Result<(), Failure> {
    store(result, k, p.x(), p.y(), p.z())
}

fn store_vector(result: &mut [f64], k: usize, v: Vector3) -> Result<(), Failure> {
    store(result, k, v.x(), v.y(), v.z())
}

impl GeomFillConstrainedFilling {
    /// `Build()`: each pair of opposite boundaries (0 and 2, then 1 and 3) that is not
    /// two degenerated ones is approximated, then the knots are matched and the poles
    /// built.
    pub(super) fn build(&mut self) -> Result<(), Failure> {
        for count in 0..2 {
            self.ibound = [count, count + 2];
            (self.ctr, self.nbd3) = ([0, 0], 0);
            for ii in 0..2 {
                let bound = self.patch()?.bound(self.ibound[ii])?;
                let bound = live(&bound)?;
                if bound.has_normals() {
                    self.ctr[ii] = 2;
                } else if !bound.is_degenerated() {
                    self.ctr[ii] = 1;
                }
                self.nbd3 += self.ctr[ii];
            }
            if self.nbd3 != 0 {
                self.perform_approx()?;
            }
        }
        self.appdone = true;
        self.match_knots()?;
        self.perform_s0()?;
        self.perform_s1()?;
        self.perform_surface()
    }
    /// The 3d tolerances of the approximated functions: `Tol3d` for a boundary and
    /// `0.5 * mig * Tolang` for its tangent field.
    fn approx_tolerances(&self) -> Result<Array1<'static, f64>, Failure> {
        let mut tol3d: Array1<'static, f64> = one_based(self.nbd3)?;
        let mut i3d: i32 = 0;
        for ii in 0..2 {
            let bound = self.patch()?.bound(self.ibound[ii])?;
            let bound = live(&bound)?;
            if self.ctr[ii] != 0 {
                i3d += 1;
                put(&mut tol3d, i3d, bound.tol3d())?;
            }
            if self.ctr[ii] == 2 {
                i3d += 1;
                let tolang: f64 = 0.5 * self.mig[self.ibound[ii] as usize] * bound.tolang();
                put(&mut tol3d, i3d, tolang)?;
            }
        }
        Ok(tol3d)
    }
    /// `PerformApprox()`: one C1 approximation of the boundaries (and tangent
    /// fields) of the current pair on the range of the first; a failure of the
    /// evaluator is kept and raised when the approximation returns, where the native
    /// exception would unwind through it.
    fn perform_approx(&mut self) -> Result<(), Failure> {
        let tol3d: Array1<'static, f64> = self.approx_tolerances()?;
        let first = self.patch()?.bound(self.ibound[0])?;
        let (f, l) = live(&first)?.bounds()?;
        let mut capture = FailureCapture::default();
        let approximation = {
            let this: &Self = self;
            let mut ev =
                |_dim: i32, _range: [f64; 2], w: f64, ord: i32, result: &mut [f64]| -> i32 {
                    capture.keep(this.eval(w, ord, result)).unwrap_or(1)
                };
            AdvApproxApproxAFunction::new(
                0,
                0,
                this.nbd3,
                None,
                None,
                Some(&tol3d),
                f,
                l,
                Shape::C1,
                this.degmax,
                this.segmax,
                &mut ev,
            )
        };
        capture.take()?;
        let app = approximation?;
        if app.is_done() || app.has_result() {
            self.store_approximation(&app, 0.5 * (f + l))?;
        }
        Ok(())
    }
    /// The degree, knots and poles of a successful approximation; a boundary that is
    /// not approximated (degenerated) gets its middle point as every pole.
    fn store_approximation(
        &mut self,
        app: &AdvApproxApproxAFunction,
        middle: f64,
    ) -> Result<(), Failure> {
        let imk: usize = self.ibound[0].min(self.ibound[1]) as usize;
        let nbpol: i32 = app.nb_poles()?;
        self.degree[imk] = app.degree();
        self.mults[imk] = app.multiplicities().cloned();
        self.knots[imk] = app.knots().cloned();
        let mut i3d: i32 = 0;
        for ii in 0..2 {
            let side: usize = self.ibound[ii] as usize;
            let mut cp: Array1<'static, Point3> = one_based(nbpol)?;
            if self.ctr[ii] != 0 {
                i3d += 1;
                app.poles_of(i3d, &mut cp)?;
            } else {
                let bound = self.patch()?.bound(self.ibound[ii])?;
                cp.init(live(&bound)?.value(middle)?);
            }
            self.curvpol[side] = Some(cp);
            if self.ctr[ii] == 2 {
                let mut tp: Array1<'static, Point3> = one_based(nbpol)?;
                i3d += 1;
                app.poles_of(i3d, &mut tp)?;
                self.tgtepol[side] = Some(tp);
            }
        }
        Ok(())
    }
    /// `Eval(W, Ord, Result)`: the boundaries (and tangent fields) of the current
    /// pair at `w` (`ord` 0) or their derivatives (`ord` 1) into `result`, three
    /// values each; returns the error code 0. `result` must hold `3 * nbd3` values
    /// (Standard_OutOfRange otherwise; a write past it natively);
    /// `f.eval(0.5, 0, &mut values)?`.
    pub fn eval(&self, w: f64, ord: i32, result: &mut [f64]) -> Result<i32, Failure> {
        let jmp: usize = 3 * self.ctr[0] as usize;
        for (ii, offset) in [(0, 0), (1, jmp)] {
            match ord {
                0 => self.eval_value(ii, offset, w, result)?,
                1 => self.eval_derivative(ii, offset, w, result)?,
                _ => {}
            }
        }
        Ok(0)
    }
    fn eval_value(
        &self,
        ii: usize,
        offset: usize,
        w: f64,
        result: &mut [f64],
    ) -> Result<(), Failure> {
        let side: i32 = self.ibound[ii];
        if self.ctr[ii] != 0 {
            let bound = self.patch()?.bound(side)?;
            store_point(result, offset, live(&bound)?.value(w)?)?;
        }
        if self.ctr[ii] == 2 {
            store_vector(
                result,
                offset + 3,
                field(&self.tgalg[side as usize])?.value(w)?,
            )?;
        }
        Ok(())
    }
    fn eval_derivative(
        &self,
        ii: usize,
        offset: usize,
        w: f64,
        result: &mut [f64],
    ) -> Result<(), Failure> {
        let side: i32 = self.ibound[ii];
        if self.ctr[ii] != 0 {
            let bound = self.patch()?.bound(side)?;
            store_vector(result, offset, live(&bound)?.d1(w)?.1)?;
        }
        if self.ctr[ii] == 2 {
            store_vector(
                result,
                offset + 3,
                field(&self.tgalg[side as usize])?.d1(w)?,
            )?;
        }
        Ok(())
    }
}
