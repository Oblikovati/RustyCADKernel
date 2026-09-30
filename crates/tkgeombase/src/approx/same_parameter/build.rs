// Rust translation of ModelingData/TKGeomBase/Approx/Approx_SameParameter.cxx (Build)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_SameParameter::Build`: the samples, the same-parameter check, then the
//! loop over the number of poles of the change of parameter.
use super::fit::{CheckInput, SameParameterEvaluator, check, compute_tol_reached};
use super::{MAX_ARRAY_SIZE, NB_SAMPLES, SameParameter, SameParameterData};
use crate::geom_lib::MakeCurvefromApprox;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_to_int;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg2d::geom2d_adaptor::{Geom2dAdaptorCurve, make_curve};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface};
use rustycad_tkg3d::adv_approx::AdvApproxApproxAFunction;
use rustycad_tkmath::geom_abs::Shape;

/// `aPercentOfBadProj`: the share of samples the projections may lose.
const BAD_PROJECTION_SHARE: f64 = 0.3;
/// `aMult`: the tolerance factor accepting the approximation.
const ACCEPTANCE_FACTOR: f64 = 250.0;
/// `aMaxDeg`: the degree bound of the approximations.
const MAX_DEGREE: i32 = 11;
/// `aMaxSeg` in the loop, and the segments of the last approximation.
const LOOP_SEGMENTS: i32 = 1000;
const FALLBACK_SEGMENTS: i32 = 40;

/// What one approximation in the loop decides.
enum Outcome {
    /// Within 250 times the tolerance: the result.
    Accepted,
    /// Too far: more poles are tried (the curve on surface is restored).
    Rejected,
    /// Too far with the arrays full: the loop stops.
    Stop,
    /// AdvApprox gave nothing: the loop stops with `interpolok` still true.
    NoResult,
}

/// The approximated pcurve of one change of parameter.
struct Approximated {
    curve: Handle<Geom2dBSplineCurve>,
    adaptor: Handle<dyn Adaptor2dCurve2d>,
}

impl SameParameter {
    /// `Build(Tolerance)`.
    pub(super) fn build(&mut self, tolerance: f64) -> Result<(), Failure> {
        let mut data: SameParameterData = self.initial_data(tolerance)?;
        if !self.build_initial_distribution(&mut data)? {
            (self.same_parameter, self.done) = (false, false);
            return Ok(());
        }
        let nb_pnt: i32 = data.nb_pnt - real_to_int(BAD_PROJECTION_SHARE * f64::from(data.nb_pnt));
        let (same, _max_sq_deviation) = self.check_same_parameter(&mut data)?;
        self.same_parameter = same;
        if same {
            self.tol_reached = self.tol_reached_on(&data)?;
            self.done = true;
            return Ok(());
        }
        if data.nb_pnt < nb_pnt {
            self.tol_reached = self.tol_reached_on(&data)?;
            self.curve_2d = make_curve(self.hcurve_2d()?)?;
            self.done = false;
            return Ok(());
        }
        let Some(tangents) = self.compute_tangents(&data)? else {
            (self.same_parameter, self.done) = (false, false);
            self.tol_reached = self.tol_reached_on(&data)?;
            return Ok(());
        };
        let continuity: Shape = self.hcurve_2d()?.continuity()?.min(Shape::C1);
        let mut best_sq_tol: f64 = data.tol * data.tol;
        if !self.pole_loop(&mut data, tangents, continuity, &mut best_sq_tol)? {
            return Ok(());
        }
        if !self.done && !self.fallback(&mut data, tangents, continuity, best_sq_tol)? {
            return Ok(());
        }
        let copy: Handle<dyn Adaptor3dCurve> = data.cons.shallow_copy()?;
        self.curve_on_surface = copy.cast();
        Ok(())
    }

    fn initial_data(&self, tolerance: f64) -> Result<SameParameterData, Failure> {
        let cons = Adaptor3dCurveOnSurface::with_curve(&self.hcurve_2d, &self.surf)?;
        let c3d = self.c3d()?;
        let size: usize = MAX_ARRAY_SIZE as usize;
        Ok(SameParameterData {
            c2d_pf: cons.first_parameter()?,
            c2d_pl: cons.last_parameter()?,
            c3d_pf: c3d.first_parameter()?,
            c3d_pl: c3d.last_parameter()?,
            cons,
            nb_pnt: 0,
            pc3d: vec![0.0; size],
            pc2d: vec![0.0; size],
            new_pc3d: vec![0.0; size],
            new_pc2d: vec![0.0; size],
            tol: tolerance,
        })
    }

    fn tol_reached_on(&self, data: &SameParameterData) -> Result<f64, Failure> {
        compute_tol_reached(self.c3d()?, &data.cons, 2 * NB_SAMPLES)
    }

    // The loop over the number of poles; false when an interpolation fails (Build
    // then returns at once).
    fn pole_loop(
        &mut self,
        data: &mut SameParameterData,
        tangents: [f64; 2],
        continuity: Shape,
        best_sq_tol: &mut f64,
    ) -> Result<bool, Failure> {
        let mut tolsov: f64 = precision::infinite();
        loop {
            let (mut poles, mut flat_knots) = change_arrays(data.nb_pnt)?;
            if !Self::interpolate(data, tangents, &mut poles, &mut flat_knots)? {
                (self.same_parameter, self.done) = (false, false);
                self.tol_reached = self.tol_reached_on(data)?;
                return Ok(false);
            }
            let mut algtol: f64 = best_sq_tol.sqrt();
            let input: CheckInput<'_> = CheckInput {
                flat_knots: &flat_knots,
                poles: &poles,
                nbp: data.nb_pnt + 1,
                pc3d: &data.pc3d,
                c3d: self.c3d()?,
                cons: &data.cons,
            };
            let mut interpolok: bool = check(&input, &mut algtol, tolsov)?;
            tolsov = algtol;
            if interpolok {
                match self.try_approximation(
                    data,
                    [&poles, &flat_knots],
                    continuity,
                    *best_sq_tol,
                )? {
                    Outcome::Accepted | Outcome::Stop => return Ok(true),
                    Outcome::Rejected => interpolok = false,
                    Outcome::NoResult => {}
                }
            }
            if interpolok || !self.increase_nb_poles(&poles, &flat_knots, data, best_sq_tol)? {
                return Ok(true);
            }
        }
    }

    // AdvApprox of the pcurve composed with the change of parameter.
    fn approximate(
        &self,
        data: &SameParameterData,
        [poles, flat_knots]: [&Array1<'static, f64>; 2],
        continuity: Shape,
        best_sq_tol: f64,
        max_segments: i32,
    ) -> Result<AdvApproxApproxAFunction, Failure> {
        let besttol: f64 = best_sq_tol.sqrt();
        let surf = self.surf()?;
        let mut tol1d: Array1<'static, f64> =
            Array1::with_bounds(1, 2).map_err(|e| array_failure(e, "Approx_SameParameter"))?;
        (tol1d[1], tol1d[2]) = (surf.u_resolution(besttol)?, surf.v_resolution(besttol)?);
        let mut evaluator = SameParameterEvaluator::new(flat_knots, poles, self.hcurve_2d.clone());
        let approx = AdvApproxApproxAFunction::new(
            2,
            0,
            0,
            Some(&tol1d),
            None,
            None,
            data.c3d_pf,
            data.c3d_pl,
            continuity,
            MAX_DEGREE,
            max_segments,
            &mut evaluator,
        );
        evaluator.take_failure()?;
        approx
    }

    fn try_approximation(
        &mut self,
        data: &mut SameParameterData,
        arrays: [&Array1<'static, f64>; 2],
        continuity: Shape,
        best_sq_tol: f64,
    ) -> Result<Outcome, Failure> {
        let approx = self.approximate(data, arrays, continuity, best_sq_tol, LOOP_SEGMENTS)?;
        if !approx.is_done() && !approx.has_result() {
            return Ok(Outcome::NoResult);
        }
        let saved: Handle<dyn Adaptor2dCurve2d> = data.cons.get_curve();
        let approximated: Approximated = approximated_pcurve(&approx)?;
        data.cons.load_curve(&approximated.adaptor)?;
        self.tol_reached = self.tol_reached_on(data)?;
        if self.tol_reached < ACCEPTANCE_FACTOR * best_sq_tol.sqrt() {
            self.curve_2d = approximated.curve.cast();
            self.hcurve_2d = approximated.adaptor;
            self.done = true;
            return Ok(Outcome::Accepted);
        }
        if data.nb_pnt < MAX_ARRAY_SIZE - 1 {
            data.cons.load_curve(&saved)?;
            return Ok(Outcome::Rejected);
        }
        Ok(Outcome::Stop)
    }

    // "Loop is finished unsuccessfully": the original pcurve with its deviation, or
    // the last approximation when it deviates less; false when AdvApprox gives
    // nothing (Build then returns without a curve on surface).
    fn fallback(
        &mut self,
        data: &mut SameParameterData,
        tangents: [f64; 2],
        continuity: Shape,
        best_sq_tol: f64,
    ) -> Result<bool, Failure> {
        data.cons.load_curve(&self.hcurve_2d)?;
        self.tol_reached = self.tol_reached_on(data)?;
        self.curve_2d = make_curve(self.hcurve_2d()?)?;
        let (mut poles, mut flat_knots) = change_arrays(data.nb_pnt)?;
        Self::interpolate(data, tangents, &mut poles, &mut flat_knots)?;
        let arrays = [&poles, &flat_knots];
        let approx = self.approximate(data, arrays, continuity, best_sq_tol, FALLBACK_SEGMENTS)?;
        if !approx.is_done() && !approx.has_result() {
            self.done = false;
            return Ok(false);
        }
        let approximated: Approximated = approximated_pcurve(&approx)?;
        data.cons.load_curve(&approximated.adaptor)?;
        let approx_tol: f64 = self.tol_reached_on(data)?;
        if approx_tol < self.tol_reached {
            self.tol_reached = approx_tol;
            self.curve_2d = approximated.curve.cast();
            self.hcurve_2d = approximated.adaptor;
        }
        self.done = true;
        Ok(true)
    }
}

/// The pcurve of the two 1D subspaces (`GeomLib_MakeCurvefromApprox::Curve2dFromTwo1d`)
/// and its adaptor.
fn approximated_pcurve(approx: &AdvApproxApproxAFunction) -> Result<Approximated, Failure> {
    let curve: Handle<Geom2dBSplineCurve> =
        MakeCurvefromApprox::new(approx).curve2d_from_two_1d(1, 2)?;
    let adaptor = Geom2dAdaptorCurve::with_curve(&curve.clone().cast())?;
    Ok(Approximated {
        curve,
        adaptor: Handle::new(adaptor).cast(),
    })
}

/// `Poles(1, nbpnt + 3)` and `FlatKnots(1, nbpnt + 7)`.
fn change_arrays(nb_pnt: i32) -> Result<(Array1<'static, f64>, Array1<'static, f64>), Failure> {
    let failure = |e| array_failure(e, "Approx_SameParameter::Build");
    Ok((
        Array1::with_bounds(1, nb_pnt + 3).map_err(failure)?,
        Array1::with_bounds(1, nb_pnt + 7).map_err(failure)?,
    ))
}
