// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_Inter.cxx (SinglePerform, NeighbourPerform and TestBound; skv, OCC616) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersection of one part of each bisector: by `Geom2dInt_GInter`, by the
//! common guide of neighbouring curve-curve bisectors, and the ends of a curve on
//! a segment.
use super::{BisectorInter, InterTolerances, Part};
use crate::bisector::{BisecAna, BisecCC, BisecPC, BisectorCurve, BisectorFunctionInter};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, errors};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeomalgo::int_res2d::{Domain, IntersectionPoint, Transition};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Line2, Point2};
use rustycad_tkmath::math::BissecNewton;

// The curve of a part; a null one is a native null dereference.
fn curve_of(part: &Part) -> Result<&Handle<dyn Geom2dCurve>, Failure> {
    if part.curve.is_null() {
        return Err(errors::null_object(
            "Bisector_Inter::SinglePerform: the part is null (an interval before the domain was skipped); expected a curve",
        ));
    }
    Ok(&part.curve)
}

// The conic of an analytic bisector, the curve itself otherwise.
fn analytic_curve(curve: &Handle<dyn Geom2dCurve>) -> Handle<dyn Geom2dCurve> {
    match curve.cast::<BisecAna>().get() {
        Some(ana) => ana.geom2d_curve(),
        None => curve.clone(),
    }
}

impl BisectorInter {
    /// `SinglePerform(C1, D1, C2, D2, TolConf, Tol, ComunElement)`.
    pub(super) fn single_perform(
        &mut self,
        part1: &Part,
        part2: &Part,
        tol: InterTolerances,
    ) -> Result<(), Failure> {
        let (bis1, bis2) = (curve_of(part1)?, curve_of(part2)?);
        let (d1, d2) = (&part1.domain, &part2.domain);
        let ana = |c: &Handle<dyn Geom2dCurve>| c.cast::<BisecAna>().is_some();
        let pc = |c: &Handle<dyn Geom2dCurve>| c.cast::<BisecPC>().is_some();
        if ana(bis1) || ana(bis2) {
            let (c1, c2) = (analytic_curve(bis1), analytic_curve(bis2));
            self.test_segment_ends(&c1, d1, &c2, d2, tol.tol_conf)?;
            return self.intersect(&c1, d1, &c2, d2, tol);
        }
        if pc(bis1) || pc(bis2) {
            return self.intersect(bis1, d1, bis2, d2, tol);
        }
        let (cc1, cc2): (Handle<BisecCC>, Handle<BisecCC>) = (bis1.cast(), bis2.cast());
        if tol.comun_element && cc1.is_some() && cc2.is_some() {
            return self.neighbour_perform(&cc1, d1, &cc2, d2, tol.tol);
        }
        // One of the two bisectors is a segment: the ends of the other are tested
        // on it.
        self.test_segment_ends(bis1, d1, bis2, d2, tol.tol_conf)?;
        self.intersect(bis1, d1, bis2, d2, tol)
    }

    // The TestBound of the one curve that is a line when the other is not.
    fn test_segment_ends(
        &mut self,
        c1: &Handle<dyn Geom2dCurve>,
        d1: &Domain,
        c2: &Handle<dyn Geom2dCurve>,
        d2: &Domain,
        tol_conf: f64,
    ) -> Result<(), Failure> {
        let (l1, l2): (Handle<Geom2dLine>, Handle<Geom2dLine>) = (c1.cast(), c2.cast());
        match (l1.get(), l2.get()) {
            (Some(line), None) => self.test_bound(line.lin2d(), d1, c2, d2, tol_conf, false),
            (None, Some(line)) => self.test_bound(line.lin2d(), d2, c1, d1, tol_conf, true),
            _ => Ok(()),
        }
    }

    // `Geom2dInt_GInter` on the two parts, appended with their domain bounds.
    fn intersect(
        &mut self,
        c1: &Handle<dyn Geom2dCurve>,
        d1: &Domain,
        c2: &Handle<dyn Geom2dCurve>,
        d2: &Domain,
        tol: InterTolerances,
    ) -> Result<(), Failure> {
        let a1 = Geom2dAdaptorCurve::with_curve(c1)?;
        let a2 = Geom2dAdaptorCurve::with_curve(c2)?;
        let mut intersect = GInter::new();
        intersect.perform(&a1, d1, &a2, d2, tol.tol_conf, tol.tol)?;
        let (f1, l1) = (d1.first_parameter()?, d1.last_parameter()?);
        let (f2, l2) = (d2.first_parameter()?, d2.last_parameter()?);
        self.base.append_composite(&intersect, f1, l1, f2, l2)
    }

    /// `NeighbourPerform(C1, D1, C2, D2, Tol)`: `Bis1` separates A and B and is
    /// parameterized by B, `Bis2` separates B and C; `Bis2` is parameterized by B
    /// (`ChangeGuide`) and the parameter where the distances to B are equal is
    /// found by math_BissecNewton on Bisector_FunctionInter.
    fn neighbour_perform(
        &mut self,
        bis1: &Handle<BisecCC>,
        d1: &Domain,
        bis2: &Handle<BisecCC>,
        d2: &Domain,
        tol: f64,
    ) -> Result<(), Failure> {
        let eps: f64 = precision::p_confusion();
        let bis_temp: Handle<BisecCC> = bis2.change_guide()?;
        let guide: Handle<dyn Geom2dCurve> = bis2.curve(2)?;
        // The points are not used; the parameters on the guide are (0 where the
        // native method does not write them).
        let u_max: f64 = bis2.value_and_dist(d2.first_parameter()?)?.u2;
        let u_min: f64 = bis2.value_and_dist(d2.last_parameter()?)?.u2;
        let u_min: f64 = std_max(d1.first_parameter()?, u_min);
        let u_max: f64 = std_min(d1.last_parameter()?, u_max);
        self.base.done = true;
        if u_min - eps > u_max + eps {
            return Ok(());
        }
        let bis1_curve: Handle<dyn BisectorCurve> = bis1.cast();
        let temp_curve: Handle<dyn BisectorCurve> = bis_temp.cast();
        let mut fint = BisectorFunctionInter::with_curves(&guide, &bis1_curve, &temp_curve);
        let mut solution = BissecNewton::new(tol);
        solution.perform(&mut fint, u_min, u_max, 20);
        fint.raised()?;
        if !solution.is_done() {
            return Ok(());
        }
        let u_sol: f64 = solution.root()?;
        let psol = bis_temp.value_and_dist(u_sol)?;
        let point = IntersectionPoint::new(
            psol.point,
            u_sol,
            psol.u2,
            Transition::new(),
            Transition::new(),
            false,
        );
        self.base.append_point(point);
        Ok(())
    }

    /// `TestBound(Bis1, D1, Bis2, D2, TolConf, Reverse)`: the ends of `bis2` that are
    /// on the segment `l1` (within its domain tolerances) become intersection
    /// points; on a curve-curve bisector the point is projected on the segment.
    #[allow(clippy::neg_cmp_op_on_partial_ord)] // The negated native test keeps its NaN answer.
    fn test_bound(
        &mut self,
        l1: Line2,
        d1: &Domain,
        bis2: &Handle<dyn Geom2dCurve>,
        d2: &Domain,
        tol_conf: f64,
        reverse: bool,
    ) -> Result<(), Failure> {
        let (uf, ul) = (d2.first_parameter()?, d2.last_parameter()?);
        let ends: [(f64, Point2); 2] = [(uf, bis2.value(uf)?), (ul, bis2.value(ul)?)];
        // skv, OCC616: the tolerance is TolConf for every kind of bisector.
        let bisec_algo: bool = bis2.cast::<BisecCC>().is_some();
        for (u2, p) in ends {
            if !(l1.distance(p) < tol_conf) {
                continue;
            }
            let u1: f64 = elclib::parameter(&l1, p)?;
            if d1.first_parameter()? - d1.first_tolerance()? < u1
                && d1.last_parameter()? + d1.last_tolerance()? > u1
            {
                let point: Point2 = if bisec_algo {
                    elclib::value(u1, &l1)
                } else {
                    p
                };
                self.base.append_point(IntersectionPoint::new(
                    point,
                    u1,
                    u2,
                    Transition::new(),
                    Transition::new(),
                    reverse,
                ));
            }
        }
        Ok(())
    }
}
