// Rust translation of ModelingData/TKGeomBase/Geom2dConvert/Geom2dConvert_ApproxArcsSegments.cxx
// (makeFreeform, getLinearParts, findInflection) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The free-form approximation: the scan for straight parts and inflections, then
//! segments on the straight parts and biarcs between them.
use super::curve_tests::{check_continuity, is_inflection_point, is_inflection_point_from};
use super::{ApproxArcsSegments, ApproxArcsStatus, MAXPOINTS, seq_at};
use crate::gcpnts::QuasiUniformDeflection;
use crate::geom2d_convert::ppoint::PPoint;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Xy;

/// The parameter point the native `aPrevParam` pointer designates: the first
/// extremity or an element of the local parameter sequence (1-based).
#[derive(Clone, Copy)]
enum PrevParam {
    FirstExtremity,
    Sequence(usize),
}

impl ApproxArcsSegments<'_> {
    /// `makeFreeform`: segments on the straight parts found by `getLinearParts`,
    /// biarc approximations between them, each checked for tangent continuity with
    /// the element before; true when something was built without error.
    pub(super) fn make_freeform(&mut self) -> Result<bool, Failure> {
        let mut seq_param_points: Vec<PPoint> = Vec::new();
        // Calculation of the inflection points.
        self.get_linear_parts(&mut seq_param_points)?;
        let is_no_inf_points: bool = seq_param_points.is_empty();
        let seq_linear_parts: Vec<Handle<dyn Geom2dCurve>> =
            self.linear_parts(&mut seq_param_points)?;
        let mut is_done: bool = true;
        let mut prev: PrevParam = PrevParam::FirstExtremity;
        let mut i: usize = 1;
        while i < seq_param_points.len() {
            is_done =
                self.approximate_to_linear_part(&mut seq_param_points, prev, i, &seq_linear_parts)?;
            if !is_done {
                self.my_status = ApproxArcsStatus::Error;
                break;
            }
            prev = PrevParam::Sequence(i + 1);
            i += 2;
        }
        // Approximation for the non-linear part following the last linear part.
        let prev_point: PPoint = self.prev_point(&seq_param_points, prev);
        if is_done && prev_point != self.my_ext[1] {
            is_done = self.approximate_tail(&mut seq_param_points, prev, is_no_inf_points)?;
            if !is_done {
                self.my_status = ApproxArcsStatus::Error;
            }
        }
        Ok(self.my_seq_curves.length() != 0 && self.my_status != ApproxArcsStatus::Error)
    }

    // The segment of each linear part (between odd and even entries), or null.
    fn linear_parts(&self, points: &mut [PPoint]) -> Result<Vec<Handle<dyn Geom2dCurve>>, Failure> {
        let mut parts: Vec<Handle<dyn Geom2dCurve>> = Vec::new();
        let mut i: usize = 1;
        while i < points.len() {
            let mut line_curve: Handle<dyn Geom2dCurve> = Handle::default();
            let (mut param0, mut param1) = (points[i - 1], points[i]);
            if param0 != param1 {
                line_curve = self.make_line(&mut param0, &mut param1, false)?.cast();
                (points[i - 1], points[i]) = (param0, param1);
            }
            parts.push(line_curve);
            i += 2;
        }
        Ok(parts)
    }

    // One step of the main loop: the approximation of the non-linear part before
    // linear part i (from `prev`), then the linear part itself (its segment, or an
    // approximation where no segment was built).
    fn approximate_to_linear_part(
        &mut self,
        points: &mut [PPoint],
        prev: PrevParam,
        i: usize,
        linear_parts: &[Handle<dyn Geom2dCurve>],
    ) -> Result<bool, Failure> {
        if points[i - 1] != self.prev_point(points, prev) {
            let mut first: PPoint = self.prev_point(points, prev);
            let mut last: PPoint = points[i - 1];
            let is_done: bool = self.approximate_checked(&mut first, &mut last)?;
            self.set_prev_point(points, prev, first);
            points[i - 1] = last;
            if !is_done {
                return Ok(false);
            }
        }
        let curve: &Handle<dyn Geom2dCurve> = &linear_parts[i.div_ceil(2) - 1];
        if !curve.is_null() {
            self.my_seq_curves.append(curve.clone());
            return Ok(true);
        }
        let (mut param0, mut param1) = (points[i - 1], points[i]);
        let is_done: bool = self.approximate_checked(&mut param0, &mut param1)?;
        (points[i - 1], points[i]) = (param0, param1);
        Ok(is_done)
    }

    // makeApproximation, then the continuity test of the first new element against
    // the element before it.
    fn approximate_checked(
        &mut self,
        first: &mut PPoint,
        last: &mut PPoint,
    ) -> Result<bool, Failure> {
        let last_ind: i32 = self.my_seq_curves.length();
        let mut is_done: bool = self.make_approximation(first, last)?;
        if is_done && last_ind != 0 && self.my_seq_curves.length() > last_ind {
            let before: Handle<dyn Geom2dCurve> =
                self.my_seq_curves.value(last_ind as usize)?.clone();
            let after: Handle<dyn Geom2dCurve> =
                self.my_seq_curves.value(last_ind as usize + 1)?.clone();
            is_done = check_continuity(&before, &after, self.my_angle_tolerance)?;
        }
        Ok(is_done)
    }

    // The part after the last linear part. A closed curve without inflection (like
    // an ellipse) is approximated in two halves.
    fn approximate_tail(
        &mut self,
        points: &mut [PPoint],
        prev: PrevParam,
        is_no_inf_points: bool,
    ) -> Result<bool, Failure> {
        let (ext0, ext1): (Xy, Xy) = (self.my_ext[0].point(), self.my_ext[1].point());
        if is_no_inf_points && (ext0 - ext1).modulus() < self.my_tolerance {
            let middle: f64 = 0.5 * (self.my_ext[0].parameter() + self.my_ext[1].parameter());
            let mut pp = PPoint::from_adaptor(middle, self.my_curve)?;
            let mut first: PPoint = self.my_ext[0];
            let is_done: bool = self.make_approximation(&mut first, &mut pp)?;
            self.my_ext[0] = first;
            if !is_done {
                return Ok(false);
            }
            let mut last: PPoint = self.my_ext[1];
            let is_done: bool = self.make_approximation(&mut pp, &mut last)?;
            self.my_ext[1] = last;
            return Ok(is_done);
        }
        let (mut first, mut last): (PPoint, PPoint) =
            (self.prev_point(points, prev), self.my_ext[1]);
        let is_done: bool = self.make_approximation(&mut first, &mut last)?;
        self.set_prev_point(points, prev, first);
        self.my_ext[1] = last;
        Ok(is_done)
    }

    fn prev_point(&self, points: &[PPoint], prev: PrevParam) -> PPoint {
        match prev {
            PrevParam::FirstExtremity => self.my_ext[0],
            PrevParam::Sequence(index) => points[index - 1],
        }
    }

    fn set_prev_point(&mut self, points: &mut [PPoint], prev: PrevParam, value: PPoint) {
        match prev {
            PrevParam::FirstExtremity => self.my_ext[0] = value,
            PrevParam::Sequence(index) => points[index - 1] = value,
        }
    }

    /// `getLinearParts`: samples the curve (GCPnts_QuasiUniformDeflection at half the
    /// tolerance, else MAXPOINTS steps) into the stored scan parameters, then appends
    /// the bounds of each straight part (in pairs) to `seq_par`: the whole curve when
    /// every sample is within half the tolerance of the chord, else the runs of
    /// inflection points longer than 10 times the tolerance.
    pub(super) fn get_linear_parts(&mut self, seq_par: &mut Vec<PPoint>) -> Result<(), Failure> {
        self.sample_curve()?;
        if self.is_linear()? {
            seq_par.push(self.my_ext[0]);
            seq_par.push(self.my_ext[1]);
            return Ok(());
        }
        let mut scan = InflectionScan {
            ind_start_linear: 0,
            last_infl_param: self.my_ext[0],
            first_infl_param: self.my_ext[0],
        };
        for i in 1..=self.my_seq_params.len() as i32 {
            self.scan_step(&mut scan, i, seq_par)?;
        }
        let dist: f64 = (scan.first_infl_param.point() - self.my_ext[1].point()).modulus();
        if scan.ind_start_linear != 0 && dist > 10.0 * self.my_tolerance {
            seq_par.push(scan.first_infl_param);
            seq_par.push(self.my_ext[1]);
        }
        Ok(())
    }

    // Fill the sequence with values along the curve.
    fn sample_curve(&mut self) -> Result<(), Failure> {
        self.my_seq_params.clear();
        let curve: &dyn Adaptor2dCurve2d = self.my_curve;
        let qu_def_algo =
            QuasiUniformDeflection::with_curve(curve, self.my_tolerance * 0.5, Shape::C1)?;
        if qu_def_algo.is_done() {
            for i in 1..=qu_def_algo.nb_points()? {
                self.my_seq_params
                    .push(PPoint::from_adaptor(qu_def_algo.parameter(i)?, curve)?);
            }
            return Ok(());
        }
        let param_step: f64 =
            (self.my_ext[1].parameter() - self.my_ext[0].parameter()) / f64::from(MAXPOINTS);
        for i in 1..=MAXPOINTS {
            let param: f64 = self.my_ext[0].parameter() + param_step * f64::from(i);
            self.my_seq_params.push(PPoint::from_adaptor(param, curve)?);
        }
        Ok(())
    }

    // Whether the curve may be linearised: every sample within half the tolerance of
    // the chord (the chord's square length is compared with Precision::Confusion()).
    fn is_linear(&self) -> Result<bool, Failure> {
        let mut dir: Xy = self.my_ext[1].point() - self.my_ext[0].point();
        let mod2: f64 = dir.square_modulus();
        // The negation keeps the native fall-through for a NaN chord.
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !(mod2 > precision::confusion()) {
            return Ok(false);
        }
        dir /= mod2.sqrt();
        for i in 1..=self.my_seq_params.len() as i32 {
            // Distance from point i to the segment between the two extremities.
            let dist: f64 = dir ^ (seq_at(&self.my_seq_params, i).point() - self.my_ext[0].point());
            if dist > self.my_tolerance * 0.5 || dist < -self.my_tolerance * 0.5 {
                return Ok(false);
            }
        }
        Ok(true)
    }

    // One sample of the inflection scan (the loop body of getLinearParts).
    fn scan_step(
        &self,
        scan: &mut InflectionScan,
        i: i32,
        seq_par: &mut Vec<PPoint>,
    ) -> Result<(), Failure> {
        let cur_param: PPoint = seq_at(&self.my_seq_params, i);
        if scan.ind_start_linear == 0 {
            if is_inflection_point(cur_param.parameter(), self.my_curve)? {
                scan.last_infl_param = cur_param;
                if i > 1 {
                    scan.first_infl_param =
                        self.find_inflection(&cur_param, &seq_at(&self.my_seq_params, i - 1))?;
                }
                scan.ind_start_linear = i;
            }
            return Ok(());
        }
        let tol: f64 = self.my_angle_tolerance;
        let still_first: bool = is_inflection_point_from(
            scan.first_infl_param.parameter(),
            &cur_param,
            self.my_curve,
            tol,
        )?;
        if is_inflection_point_from(
            cur_param.parameter(),
            &scan.first_infl_param,
            self.my_curve,
            tol,
        )? {
            scan.last_infl_param = cur_param;
            return self.advance_first_inflection(scan, &cur_param, i, still_first);
        }
        self.close_linear_run(scan, &cur_param, seq_par)
    }

    // Moves the start of the run forward until it is still an inflection point seen
    // from the current sample, or restarts the scan when it reaches the sample.
    fn advance_first_inflection(
        &self,
        scan: &mut InflectionScan,
        cur_param: &PPoint,
        i: i32,
        mut still_first: bool,
    ) -> Result<(), Failure> {
        while !still_first {
            scan.ind_start_linear += 1;
            if scan.ind_start_linear >= i {
                scan.ind_start_linear = 0;
                break;
            }
            scan.first_infl_param = seq_at(&self.my_seq_params, scan.ind_start_linear);
            let tol: f64 = self.my_angle_tolerance;
            still_first = is_inflection_point_from(
                scan.first_infl_param.parameter(),
                cur_param,
                self.my_curve,
                tol,
            )?;
        }
        Ok(())
    }

    // The run ends before the current sample: refine both of its bounds and add the
    // interval when it is more than 10 times the tolerance long.
    fn close_linear_run(
        &self,
        scan: &mut InflectionScan,
        cur_param: &PPoint,
        seq_par: &mut Vec<PPoint>,
    ) -> Result<(), Failure> {
        scan.last_infl_param = self.find_inflection(&scan.last_infl_param, cur_param)?;
        let tol: f64 = self.my_angle_tolerance;
        if !is_inflection_point_from(
            scan.first_infl_param.parameter(),
            &scan.last_infl_param,
            self.my_curve,
            tol,
        )? {
            scan.first_infl_param =
                self.find_inflection(&scan.last_infl_param, &scan.first_infl_param)?;
        }
        let dist: f64 = (scan.first_infl_param.point() - scan.last_infl_param.point()).modulus();
        if scan.first_infl_param.parameter() < scan.last_infl_param.parameter()
            && dist > 10.0 * self.my_tolerance
        {
            seq_par.push(scan.first_infl_param);
            seq_par.push(scan.last_infl_param);
        }
        scan.ind_start_linear = 0;
        Ok(())
    }

    /// `findInflection`: three bisection steps between a point of the straight run
    /// and one outside it; the point at the last midpoint.
    pub(super) fn find_inflection(
        &self,
        param_is_infl: &PPoint,
        param_no_infl: &PPoint,
    ) -> Result<PPoint, Failure> {
        let (mut lower, mut upper): (f64, f64) =
            (param_is_infl.parameter(), param_no_infl.parameter());
        let mut test: f64 = 0.0;
        for _ in 0..3 {
            test = (lower + upper) * 0.5;
            if is_inflection_point_from(
                test,
                param_is_infl,
                self.my_curve,
                self.my_angle_tolerance,
            )? {
                lower = test;
            } else {
                upper = test;
            }
        }
        PPoint::from_adaptor(test, self.my_curve)
    }
}

// The state of the inflection scan of getLinearParts.
struct InflectionScan {
    ind_start_linear: i32,
    last_infl_param: PPoint,
    first_infl_param: PPoint,
}
