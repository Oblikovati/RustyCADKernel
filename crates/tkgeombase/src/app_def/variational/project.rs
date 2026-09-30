// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_Variational.cxx (Project, ACR)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parameter optimizations of the points: the orthogonal projection on the
//! curve (Newton) and the arc-length reparametrization (ACR).
use super::Variational;
use crate::fem_tool::Curve;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// The errors `Project` returns: the maximum (and its point, which stays at the
/// caller's value, shifted, when every distance is 0), the quadratic and the average.
#[derive(Clone, Copy, Default)]
pub(super) struct ProjectErrors {
    pub(super) num_points: i32,
    pub(super) max_err: f64,
    pub(super) qua_err: f64,
    pub(super) ave_err: f64,
}

impl ProjectErrors {
    // The distance of point `ipnt`: a new maximum (and its point) when larger, and
    // its terms of the quadratic and the average error.
    fn add_distance(&mut self, ipnt: i32, dist: f64) {
        if dist > self.max_err {
            self.max_err = dist;
            self.num_points = ipnt;
        }
        self.qua_err += dist * dist;
        self.ave_err += dist;
    }
}

fn at(array: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    array
        .value(index)
        .copied()
        .map_err(|error| array_failure(error, "AppDef_Variational"))
}

fn put(array: &mut Array1<'_, f64>, index: i32, value: f64) -> Result<(), Failure> {
    array
        .set_value(index, value)
        .map_err(|error| array_failure(error, "AppDef_Variational"))
}

/// The evaluation buffers of the projection.
struct Evaluation {
    val: Array1<'static, f64>,
    first: Array1<'static, f64>,
    second: Array1<'static, f64>,
}

/// The running Newton projection of one point: the parameter, its distance, the
/// count of converged steps and the iteration count.
struct NewtonRun {
    t: f64,
    dist: f64,
    n_it_cv: i32,
    iter: i32,
}

impl Variational {
    // |C(t) - P| for the point at offset `i0` of myTabPoints.
    fn distance_at(&self, val: &Array1<'_, f64>, i0: i32) -> Result<f64, Failure> {
        let mut dist: f64 = 0.;
        for i in 1..=self.dimension {
            let aux: f64 = at(val, i)? - self.tab_point(i0 + i)?;
            dist += aux * aux;
        }
        Ok(dist.sqrt())
    }

    // Newton's method for solving (C'(t), C(t) - P) = 0 from `t_new`, clamped to
    // [0, 1]: stops on a null derivative, a distance increase above 1e-9 (keeping
    // the previous step), two converged steps or `nb_iterations` steps.
    fn newton(
        &self,
        c: &Curve,
        (t_new, i0): (f64, i32),
        ev: &mut Evaluation,
        nb_iterations: i32,
    ) -> Result<(f64, f64), Failure> {
        c.d0(t_new, &mut ev.val)?;
        let dist: f64 = self.distance_at(&ev.val, i0)?;
        let mut run = NewtonRun {
            t: t_new,
            dist,
            n_it_cv: 0,
            iter: 0,
        };
        loop {
            if let Some(result) = self.newton_iteration(c, i0, ev, (&mut run, nb_iterations))? {
                return Ok(result);
            }
        }
    }

    // One Newton iteration from `run`: the `(t, distance)` result when it stops (a
    // null derivative, no improvement, convergence or the last iteration).
    fn newton_iteration(
        &self,
        c: &Curve,
        i0: i32,
        ev: &mut Evaluation,
        (run, nb_iterations): (&mut NewtonRun, i32),
    ) -> Result<Option<(f64, f64)>, Failure> {
        let seuil: f64 = 1.0e-9;
        run.iter += 1;
        let (t0, dist0) = (run.t, run.dist);
        let Some(t_next) = self.newton_step(c, (run.t, i0), ev)? else {
            return Ok(Some((run.t, run.dist)));
        };
        run.t = t_next;
        run.dist = self.distance_at(&ev.val, i0)?;
        let ecart: f64 = dist0 - run.dist;
        if ecart <= -seuil {
            // No improvement, stop here
            return Ok(Some((t0, dist0)));
        }
        run.n_it_cv = if ecart <= seuil { run.n_it_cv + 1 } else { 0 };
        if run.n_it_cv >= 2 || run.iter >= nb_iterations {
            return Ok(Some((run.t, run.dist)));
        }
        Ok(None)
    }

    // The Newton step from `t` (C evaluated there in `ev.val`), with C evaluated at
    // the new parameter; None on a null derivative.
    fn newton_step(
        &self,
        c: &Curve,
        (t, i0): (f64, i32),
        ev: &mut Evaluation,
    ) -> Result<Option<f64>, Failure> {
        let eps: f64 = 1.0e-12;
        c.d2(t, &mut ev.second)?;
        c.d1(t, &mut ev.first)?;
        let (mut f1, mut f2) = (0., 0.);
        for i in 1..=self.dimension {
            let aux: f64 = at(&ev.val, i)? - self.tab_point(i0 + i)?;
            let df: f64 = at(&ev.first, i)?;
            f1 += aux * df;
            f2 += df * df + aux * at(&ev.second, i)?;
        }
        if f2.abs() < eps {
            return Ok(None);
        }
        // Formula of Newton x(k+1) = x(k) - F(x(k))/F'(x(k))
        let t_new: f64 = clamp_unit(t - f1 / f2);
        c.d0(t_new, &mut ev.val)?;
        Ok(Some(t_new))
    }

    /// `Project(C, Ti, ProjTi, Distance, NumPoints, MaxErr, QuaErr, AveErr,
    /// NbIterations)`: the projection parameters of the points (`ProjTi`), their
    /// distances (`Distance` from its lower bound) and the errors; `num_points` is
    /// the caller's value, replaced by the point of the largest distance and shifted
    /// by FirstPoint - 1 in any case.
    pub(super) fn project(
        &self,
        c: &Curve,
        (ti, proj_ti): (&Array1<'_, f64>, &mut Array1<'_, f64>),
        distance: &mut Array1<'_, f64>,
        num_points: i32,
        nb_iterations: i32,
    ) -> Result<ProjectErrors, Failure> {
        let mut errors = ProjectErrors {
            num_points,
            ..ProjectErrors::default()
        };
        let mut ev: Evaluation = self.evaluation()?;
        let d0: i32 = distance.lower() - 1;
        let mut i0: i32 = -self.dimension;
        for ipnt in 1..=proj_ti.length() {
            i0 += self.dimension;
            let (t_new, dist) = self.newton(c, (at(ti, ipnt)?, i0), &mut ev, nb_iterations)?;
            put(proj_ti, ipnt, t_new)?;
            put(distance, d0 + ipnt, dist)?;
            errors.add_distance(ipnt, dist);
        }
        // Setting NumPoints to interval [myFirstPoint, myLastPoint]
        errors.num_points = errors.num_points + self.first_point - 1;
        Ok(errors)
    }

    // The three evaluation buffers `(1, Dimension)`.
    fn evaluation(&self) -> Result<Evaluation, Failure> {
        Ok(Evaluation {
            val: self.evaluation_buffer()?,
            first: self.evaluation_buffer()?,
            second: self.evaluation_buffer()?,
        })
    }

    fn evaluation_buffer(&self) -> Result<Array1<'static, f64>, Failure> {
        Array1::with_bounds(1, self.dimension)
            .map_err(|error| array_failure(error, "AppDef_Variational::Project"))
    }
}

// The Newton step clamped to [0, 1] by the native two tests (a NaN passes both).
fn clamp_unit(t: f64) -> f64 {
    t.clamp(0., 1.)
}

/// The running state of ACR: the last updated point and knot, and the next
/// constrained point.
struct Acr {
    u_old: f64,
    t_old: f64,
    p_old: i32,
    i_elm: i32,
    i_old: i32,
    p_cnt: i32,
    i_cnt: i32,
}

impl Acr {
    // The parameter `t` mapped linearly from the last kept parameter: `UOld + (t -
    // TOld) * DtInv * DU`, as `ratio = (t - TOld) * DtInv` then `UOld + ratio * DU`.
    fn remapped(&self, t: f64, (dt_inv, du): (f64, f64)) -> f64 {
        let ratio: f64 = (t - self.t_old) * dt_inv;
        self.u_old + ratio * du
    }

    // The element holding `t_para` from the current one on (at most the last).
    fn advance_element(&mut self, curve: &Curve, t_para: f64) -> Result<(), Failure> {
        let k_last: i32 = curve.knots().upper();
        while knot(curve, self.i_elm + 1)? < t_para && self.i_elm < k_last - 1 {
            self.i_elm += 1;
        }
        Ok(())
    }

    // Point `ipnt` at `t_para` is kept with the arc-length parameter `u_new`.
    fn keep(&mut self, u_new: f64, t_para: f64, ipnt: i32) {
        self.u_old = u_new;
        self.i_old = self.i_elm;
        self.t_old = t_para;
        self.p_old = ipnt;
    }
}

// Knots(index) of the curve (a short lock: the curve reads its knots in Length).
fn knot(curve: &Curve, index: i32) -> Result<f64, Failure> {
    at(&curve.knots(), index)
}

fn set_knot(curve: &Curve, index: i32, value: f64) -> Result<(), Failure> {
    put(&mut curve.knots(), index, value)
}

impl Variational {
    /// `ACR(Curve, Ti, Decima)`: the arc-length parameters of the points kept every
    /// 1/Decima of the parameter range and at the constrained points, with the curve
    /// knots mapped the same way (the lengths are measured on the partly remapped
    /// knots, as natively), then the ends forced to 0 and 1.
    pub(super) fn acr(
        &self,
        curve: &Curve,
        ti: &mut Array1<'_, f64>,
        decima: i32,
    ) -> Result<(), Failure> {
        let (ti_first, ti_last) = (ti.lower(), ti.upper());
        // (1) Compute the curve length
        let cb_long: f64 = curve.length(at(ti, ti_first)?, at(ti, ti_last)?)?;
        if ti.length() >= 2 {
            self.acr_points(curve, ti, cb_long, decima)?;
        }
        // --- Adjust the extreme values
        put(ti, ti_first, 0.)?;
        put(ti, ti_last, 1.)?;
        let (k_first, k_last) = {
            let knots = curve.knots();
            (knots.lower(), knots.upper())
        };
        let mut ii: i32 = ti_last - 1;
        while at(ti, ii)? > knot(curve, k_last)? {
            put(ti, ii, 1.)?;
            ii -= 1;
        }
        set_knot(curve, k_first, 0.)?;
        set_knot(curve, k_last, 1.)
    }

    // (2) Place the arc-length parameterization in Ti.
    fn acr_points(
        &self,
        curve: &Curve,
        ti: &mut Array1<'_, f64>,
        cb_long: f64,
        decima: i32,
    ) -> Result<(), Failure> {
        let (ti_first, ti_last) = (ti.lower(), ti.upper());
        // (2.0) Initialisation
        let delta_t: f64 = (at(ti, ti_last)? - at(ti, ti_first)?) / f64::from(decima);
        let mut v_test: f64 = at(ti, ti_first)? + delta_t;
        let mut s: Acr = self.acr_start(curve, ti)?;
        put(ti, ti_first, 0.)?;
        let mut ipnt: i32 = ti_first + 1;
        while ipnt <= ti_last {
            self.next_constrained_point(&mut s, ipnt, ti_first)?;
            let mut t_para: f64 = at(ti, ipnt)?;
            if t_para >= v_test || s.p_cnt == ipnt {
                (ipnt, t_para) = snap_to_last(ti, (ipnt, t_para), delta_t)?;
                acr_update(curve, ti, (ipnt, t_para), (cb_long, &mut s))?;
            }
            // --> New parametric threshold for decimation
            v_test = next_threshold(t_para, v_test, delta_t);
            ipnt += 1;
        }
        Ok(())
    }

    // The state of ACR before the first point: nothing updated yet, the first
    // constrained point (past the last point without constraints).
    fn acr_start(&self, curve: &Curve, ti: &Array1<'_, f64>) -> Result<Acr, Failure> {
        let (ti_first, ti_last) = (ti.lower(), ti.upper());
        let (p_cnt, i_cnt) = if self.tables.nb_constr() > 0 {
            (self.tables.typ(1)? - self.first_point + ti_first, 1)
        } else {
            (ti_last + 1, 0)
        };
        let k_first: i32 = curve.knots().lower();
        Ok(Acr {
            u_old: 0.,
            t_old: at(ti, ti_first)?,
            p_old: ti_first,
            i_elm: k_first,
            i_old: k_first,
            p_cnt,
            i_cnt,
        })
    }

    // The constrained points before `ipnt` are passed.
    fn next_constrained_point(&self, s: &mut Acr, ipnt: i32, ti_first: i32) -> Result<(), Failure> {
        let nb_cntr: i32 = self.tables.nb_constr();
        while s.i_cnt <= nb_cntr && s.p_cnt < ipnt {
            s.i_cnt += 1;
            s.p_cnt = self.tables.typ(2 * s.i_cnt - 1)? - self.first_point + ti_first;
        }
        Ok(())
    }
}

// A kept point within a hundredth of the step of the last parameter is replaced by
// the last point.
fn snap_to_last(
    ti: &Array1<'_, f64>,
    (ipnt, t_para): (i32, f64),
    delta_t: f64,
) -> Result<(i32, f64), Failure> {
    let ti_last: i32 = ti.upper();
    if at(ti, ti_last)? - t_para <= 1.0e-2 * delta_t {
        return Ok((ti_last, at(ti, ti_last)?));
    }
    Ok((ipnt, t_para))
}

// The decimation threshold past `t_para` (whole steps on, at most 1), unchanged
// while `t_para` is below it.
fn next_threshold(t_para: f64, mut v_test: f64, delta_t: f64) -> f64 {
    let eps: f64 = 1.0e-8;
    if t_para >= v_test {
        v_test += ((t_para - v_test + eps) / delta_t).ceil() * delta_t;
        if v_test > 1. - eps {
            v_test = 1.;
        }
    }
    v_test
}

// (2.2)-(2.5): the arc length at `t_para`, the knots of the elements crossed since the
// last kept point and the parameters of the skipped points mapped linearly.
fn acr_update(
    curve: &Curve,
    ti: &mut Array1<'_, f64>,
    (ipnt, t_para): (i32, f64),
    (cb_long, s): (f64, &mut Acr),
) -> Result<(), Failure> {
    let ti_first: i32 = ti.lower();
    // (2.2), (2.3) Compute the curve length
    let u_new: f64 = curve.length(at(ti, ti_first)?, t_para)? / cb_long;
    s.advance_element(curve, t_para)?;
    // (2.4) Update the splitting parameters
    let dt_inv: f64 = 1. / (t_para - s.t_old);
    let du: f64 = u_new - s.u_old;
    for ii in s.i_old + 1..=s.i_elm {
        set_knot(curve, ii, s.remapped(knot(curve, ii)?, (dt_inv, du)))?;
    }
    // (2.5) Update the point parameters ("very strange loop, because it never works").
    for ii in s.p_old + 1..=ipnt - 1 {
        put(ti, ii, s.remapped(at(ti, ii)?, (dt_inv, du)))?;
    }
    put(ti, ipnt, u_new)?;
    s.keep(u_new, t_para, ipnt);
    Ok(())
}
