// Rust translation of ModelingData/TKGeomBase/Geom2dConvert/Geom2dConvert_ApproxArcsSegments.cxx
// (makeApproximation, calculateBiArcs, calculateLines) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The recursive approximation of a non-linear part by segments, single arcs and
//! biarcs.
use super::ApproxArcsSegments;
use super::curve_tests::{check_continuity, get_parameter};
use crate::geom2d_convert::ppoint::PPoint;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkmath::gp::{self, Direction2, Vector2, Xy};

impl ApproxArcsSegments<'_> {
    /// `makeApproximation`: a checked segment when the points are closer than the
    /// tolerance, else a biarc, else the same on the two parts split at 55% of the
    /// parameter range; false (nothing done) for parameters within
    /// Precision::PConfusion(). A chord below Precision::Confusion() whose segment
    /// is refused still appends the null segment, as natively.
    pub(super) fn make_approximation(
        &mut self,
        first_param: &mut PPoint,
        last_param: &mut PPoint,
    ) -> Result<bool, Failure> {
        // If the difference between the parameters is less than
        // Precision::PConfusion(), no approximation is made.
        if !(*last_param != *first_param) {
            return Ok(false);
        }
        let distance: f64 = (first_param.point() - last_param.point()).modulus();
        if distance < self.my_tolerance {
            let curve: Handle<dyn Geom2dCurve> =
                self.make_line(first_param, last_param, true)?.cast();
            let mut is_done: bool = !curve.is_null();
            if is_done && self.my_seq_curves.length() != 0 {
                let previous: Handle<dyn Geom2dCurve> = self.last_curve()?;
                is_done = check_continuity(&previous, &curve, self.my_angle_tolerance)?;
            }
            if is_done || distance < precision::confusion() {
                self.my_seq_curves.append(curve);
                return Ok(is_done);
            }
        }
        // Calculate the biarc.
        if self.calculate_bi_arcs(first_param, last_param)? {
            return Ok(true);
        }
        // If the biarc was not calculated, the calculation is repeated on the halves.
        let split: f64 =
            first_param.parameter() + (last_param.parameter() - first_param.parameter()) * 0.55;
        let mut para_m = PPoint::from_adaptor(split, self.my_curve)?;
        if !self.make_approximation(first_param, &mut para_m)? {
            return Ok(false);
        }
        self.make_approximation(&mut para_m, last_param)
    }

    fn last_curve(&self) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        Ok(self
            .my_seq_curves
            .value(self.my_seq_curves.length() as usize)?
            .clone())
    }

    /// `calculateBiArcs`: a single arc when the tangents make equal angles with the
    /// chord (within 10 Precision::Angular() for the whole curve, a tenth of the
    /// angle tolerance otherwise), else the biarc of Nasri et al. ("A Recursive
    /// Subdivision Algorithm for Piecewise Circular Spline", Computer Graphics Forum,
    /// 2001) through the curve point nearest its junction; each arc checked against
    /// the curve.
    pub(super) fn calculate_bi_arcs(
        &mut self,
        first_param: &mut PPoint,
        last_param: &mut PPoint,
    ) -> Result<bool, Failure> {
        let resolution: f64 = gp::resolution();
        if first_param.d1().square_modulus() < resolution
            || last_param.d1().square_modulus() < resolution
        {
            // Bad initial data.
            return Ok(false);
        }
        let pnt: [Xy; 2] = [first_param.point(), last_param.point()];
        let mut dir: [Direction2; 2] = [
            Direction2::from_xy(first_param.d1())?,
            Direction2::from_xy(last_param.d1())?,
        ];
        // Try to approximate the curve by a single arc. The criterion is more rigid
        // when the curve is the entire input curve (possible problem connecting with
        // other boundaries).
        let delta = Vector2::from(pnt[1] - pnt[0]);
        let angle1: f64 = delta.angle(Vector2::from(dir[0]))?.abs();
        let angle2: f64 = delta.angle(Vector2::from(dir[1]))?.abs();
        let mut angle_threshold: f64 = precision::angular() * 10.0;
        if *first_param != self.my_ext[0] || *last_param != self.my_ext[1] {
            angle_threshold = self.my_angle_tolerance * 0.1;
        }
        if (angle1 - angle2).abs() < angle_threshold {
            // Protect last_param from the modification of D1 when the arc is rejected.
            let mut last_copy: PPoint = *last_param;
            let Some(curve) = self.make_arc(first_param, &mut last_copy, true)? else {
                return Ok(false);
            };
            if self.check_curve(
                &curve.cast(),
                first_param.parameter(),
                last_copy.parameter(),
            )? {
                *last_param = last_copy;
                self.my_seq_curves.append(curve.cast());
                return Ok(true);
            }
        }
        // The point of intersection of the two tangents at the ends.
        dir[1].reverse();
        let Some(junction) = BiarcJunction::new(pnt, dir, delta.magnitude())? else {
            return Ok(false);
        };
        self.build_biarc(first_param, last_param, junction)
    }

    // The two arcs of the biarc, each checked against the curve.
    fn build_biarc(
        &mut self,
        first_param: &PPoint,
        last_param: &PPoint,
        junction: BiarcJunction,
    ) -> Result<bool, Failure> {
        let mut param_mid_arc: PPoint = get_parameter(
            junction.xy_mid_arc,
            first_param.parameter(),
            last_param.parameter(),
            self.my_curve,
        )?;
        // Building the first arc of the biarc.
        let Some(curve1) = self.make_arc(first_param, &mut param_mid_arc, true)? else {
            return Ok(false);
        };
        if !self.check_curve(
            &curve1.cast(),
            first_param.parameter(),
            param_mid_arc.parameter(),
        )? {
            return Ok(false);
        }
        // Building the second arc of the biarc.
        let curve2: Option<Handle<Geom2dTrimmedCurve>> =
            self.make_arc(last_param, &mut param_mid_arc, false)?;
        let Some(curve2) = curve2 else {
            return Ok(false);
        };
        if !self.check_curve(
            &curve2.cast(),
            param_mid_arc.parameter(),
            last_param.parameter(),
        )? {
            return Ok(false);
        }
        self.my_seq_curves.append(curve1.cast());
        self.my_seq_curves.append(curve2.cast());
        Ok(true)
    }

    /// `calculateLines`: segments between the scan parameters of
    /// `[first_param, last_param]` longer than the tolerance. The native class
    /// declares and defines it but never calls it.
    #[allow(dead_code)]
    pub(super) fn calculate_lines(
        &mut self,
        first_param: &mut PPoint,
        last_param: &mut PPoint,
    ) -> Result<bool, Failure> {
        // None designates first_param, Some(i) the scan parameter i (1-based).
        let mut prev: Option<usize> = None;
        for i in 1..=self.my_seq_params.len() {
            let mut cur_param: PPoint = self.my_seq_params[i - 1];
            let mut prev_param: PPoint =
                prev.map_or(*first_param, |index| self.my_seq_params[index - 1]);
            if cur_param.parameter() < prev_param.parameter() {
                continue;
            }
            if cur_param.parameter() > last_param.parameter() {
                break;
            }
            // Build a line segment.
            // The negations keep the native branches for NaN parameters and distances.
            #[allow(clippy::neg_cmp_op_on_partial_ord)]
            if !(cur_param != prev_param)
                || !((cur_param.point() - prev_param.point()).modulus() > self.my_tolerance)
            {
                continue;
            }
            let curve: Handle<Geom2dTrimmedCurve> =
                self.make_line(&mut prev_param, &mut cur_param, false)?;
            match prev {
                None => *first_param = prev_param,
                Some(index) => self.my_seq_params[index - 1] = prev_param,
            }
            self.my_seq_params[i - 1] = cur_param;
            if curve.is_null() {
                return Ok(false);
            }
            self.my_seq_curves.append(curve.cast());
            prev = Some(i);
        }
        Ok(true)
    }
}

// The junction point of the biarc from the intersection of the end tangents.
struct BiarcJunction {
    xy_mid_arc: Xy,
}

impl BiarcJunction {
    // None when the intersection is behind an end, or when the junction ratio is
    // within 1e-4 of 1/2.
    fn new(pnt: [Xy; 2], dir: [Direction2; 2], ad12: f64) -> Result<Option<Self>, Failure> {
        // The direct calculation of the intersection point (replaces a class call).
        let prod: [f64; 3] = [
            pnt[0] ^ dir[0].xy(),
            pnt[1] ^ dir[1].xy(),
            dir[1].crossed(Direction2::from_xy(dir[0].xy())?),
        ];
        let int_point = Xy::new(
            (prod[0] * dir[1].x() - prod[1] * dir[0].x()) / prod[2],
            (prod[0] * dir[1].y() - prod[1] * dir[0].y()) / prod[2],
        );
        let diff: [Xy; 2] = [int_point - pnt[0], int_point - pnt[1]];
        if diff[0] * dir[0].xy() < 0.0 || diff[1] * dir[1].xy() < 0.0 {
            return Ok(None);
        }
        // The middle point of the biarc.
        let (ad1, ad2): (f64, f64) = (diff[0].modulus(), diff[1].modulus());
        let b1: f64 = ad1 / (ad1 + ad2);
        if (b1 - 0.5).abs() < 0.0001 {
            return Ok(None);
        }
        let xy: [Xy; 2] = [
            pnt[0] + dir[0].xy() * ad12 * ad1 / (ad12 + ad1 + ad2),
            pnt[1] + dir[1].xy() * ad12 * ad2 / (ad12 + ad1 + ad2),
        ];
        Ok(Some(Self {
            xy_mid_arc: xy[0] + b1 * (xy[1] - xy[0]),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkg2d::geom2d::Geom2dLine;
    use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;

    // calculateLines has no native caller, so only this test reaches it.
    #[test]
    fn calculate_lines_joins_the_scan_parameters_farther_apart_than_the_tolerance() {
        let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(gp::ox2d())).cast();
        let adaptor = Geom2dAdaptorCurve::with_range(&line, 0.0, 3.0).unwrap();
        let mut approx = ApproxArcsSegments::new(&adaptor, 0.4, 0.1).unwrap();
        approx.my_seq_params = (0..=6)
            .map(|k| PPoint::from_adaptor(f64::from(k) * 0.5, &adaptor).unwrap())
            .collect();
        let before: i32 = approx.my_seq_curves.length();
        let (mut first, mut last) = (approx.my_ext[0], approx.my_ext[1]);
        assert!(approx.calculate_lines(&mut first, &mut last).unwrap());
        // The first scan parameter equals `first`; each next one starts a segment.
        assert_eq!(approx.my_seq_curves.length() - before, 6);
    }
}
