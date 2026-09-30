// Rust translation of GeomAPI_ExtremaCurveCurve.cxx (TotalNearestPoints,
// TotalLowerDistanceParameters, TotalLowerDistance, TotalPerform) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The minimal distance between the trimmed curves of GeomAPI_ExtremaCurveCurve:
//! the nearest extremum, the ends of the curves and the projections of each end on
//! the other curve.
use super::ExtremaCurveCurve;
use super::ProjectPointOnCurve;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::Point3;

/// The candidate of one end: the end point and parameter on the curve `own` (0 or
/// 1), projected on the other curve over its range.
struct EndProjection {
    point: Point3,
    parameter: f64,
    own: usize,
    other_range: (f64, f64),
}

impl ExtremaCurveCurve {
    /// `TotalNearestPoints(P1, P2)`: the points of the minimal distance between the
    /// trimmed curves; `None` when both curves are infinite and parallel.
    /// `let points = extrema.total_nearest_points()?;`
    pub fn total_nearest_points(&mut self) -> Result<Option<(Point3, Point3)>, Failure> {
        self.ensure_total()?;
        if self.my_is_infinite {
            return Ok(None);
        }
        Ok(Some((self.my_total_points[0], self.my_total_points[1])))
    }
    /// `TotalLowerDistanceParameters(U1, U2)`: their parameters; `None` when both
    /// curves are infinite and parallel.
    /// `let parameters = extrema.total_lower_distance_parameters()?;`
    pub fn total_lower_distance_parameters(&mut self) -> Result<Option<(f64, f64)>, Failure> {
        self.ensure_total()?;
        if self.my_is_infinite {
            return Ok(None);
        }
        Ok(Some((self.my_total_pars[0], self.my_total_pars[1])))
    }
    /// `TotalLowerDistance()`: the minimal distance between the trimmed curves.
    /// `extrema.total_lower_distance()?`.
    pub fn total_lower_distance(&mut self) -> Result<f64, Failure> {
        self.ensure_total()?;
        Ok(self.my_total_dist)
    }

    // `if (!myTotalExt) { TotalPerform(); myTotalExt = true; }`.
    fn ensure_total(&mut self) -> Result<(), Failure> {
        if !self.my_total_ext {
            self.total_perform()?;
            self.my_total_ext = true;
        }
        Ok(())
    }

    // `TotalPerform()` (private). The ranges are those of the adaptors, which hold the
    // whole curves even after the ranged Init.
    fn total_perform(&mut self) -> Result<(), Failure> {
        let (u11, u12) = (self.my_c1.first_parameter()?, self.my_c1.last_parameter()?);
        let (u21, u22) = (self.my_c2.first_parameter()?, self.my_c2.last_parameter()?);
        let infinite: bool = [u11, u12, u21, u22].into_iter().all(precision::is_infinite);
        self.my_is_infinite = false;
        if infinite && self.my_ext_cc.is_parallel.clone()? {
            self.my_is_infinite = true;
            // calculate distance between any suitable point on C1 and C2
            let pon_c1: Point3 = self.my_c1.value(0.)?;
            let proj = ProjectPointOnCurve::with_curve(pon_c1, &self.my_c2.curve())?;
            self.my_total_dist = proj.lower_distance()?;
            return Ok(());
        }
        self.my_total_dist = f64::MAX;
        if self.my_is_done && !self.my_ext_cc.is_parallel.clone()? {
            (self.my_total_points[0], self.my_total_points[1]) = self.points(self.my_index)?;
            (self.my_total_pars[0], self.my_total_pars[1]) = self.parameters(self.my_index)?;
            let sq: f64 = self.my_ext_cc.extrema.square_distance(self.my_index)?;
            self.my_total_dist = sq.sqrt();
            if self.my_total_dist <= precision::confusion() {
                return Ok(());
            }
        }
        if self.end_pairs([u11, u12, u21, u22]) {
            return Ok(());
        }
        self.end_projections([u11, u12, u21, u22])
    }

    // The four pairs of ends. The native code compares every pair with the square of
    // the distance found BEFORE the pairs (it never updates `aTotalDist2`), so a later
    // pair replaces an earlier one whenever it is below that first distance; true when
    // a pair within Precision::Confusion() ends the search.
    fn end_pairs(&mut self, [u11, u12, u21, u22]: [f64; 4]) -> bool {
        let (d, [p11, p12, p21, p22]) = self.my_ext_cc.trimmed;
        let a_total_dist2: f64 = self.my_total_dist * self.my_total_dist;
        let pairs = [
            (d[0], (p11, u11), (p21, u21)),
            (d[1], (p11, u11), (p22, u22)),
            (d[2], (p12, u12), (p21, u21)),
            (d[3], (p12, u12), (p22, u22)),
        ];
        for (square, (pa, ua), (pb, ub)) in pairs {
            if a_total_dist2 > square {
                self.my_total_dist = square.sqrt();
                self.my_total_points = [pa, pb];
                self.my_total_pars = [ua, ub];
                if self.my_total_dist <= precision::confusion() {
                    return true;
                }
            }
        }
        false
    }

    // The projections of each finite end on the other curve, in the native order;
    // the search ends at a distance within Precision::Confusion() (the last
    // projection does not test it).
    fn end_projections(&mut self, [u11, u12, u21, u22]: [f64; 4]) -> Result<(), Failure> {
        let (_, [p11, p12, p21, p22]) = self.my_ext_cc.trimmed;
        let ends = [
            (p11, u11, 0, (u21, u22)),
            (p12, u12, 0, (u21, u22)),
            (p21, u21, 1, (u11, u12)),
            (p22, u22, 1, (u11, u12)),
        ];
        for (point, parameter, own, other_range) in ends {
            if precision::is_infinite(parameter) {
                continue;
            }
            let end = EndProjection {
                point,
                parameter,
                own,
                other_range,
            };
            if self.project_end(&end)? && self.my_total_dist <= precision::confusion() {
                return Ok(());
            }
        }
        Ok(())
    }

    // One end projected on the other curve; true when it gives a smaller distance.
    fn project_end(&mut self, end: &EndProjection) -> Result<bool, Failure> {
        let other = if end.own == 0 {
            &self.my_c2
        } else {
            &self.my_c1
        };
        let (umin, umax) = end.other_range;
        let proj = ProjectPointOnCurve::with_range(end.point, &other.curve(), umin, umax)?;
        if proj.nb_points() <= 0 {
            return Ok(false);
        }
        let dmin: f64 = proj.lower_distance()?;
        // `myTotalDist > dmin` (false for a NaN distance, which is then kept).
        let smaller: bool = self.my_total_dist > dmin;
        if !smaller {
            return Ok(false);
        }
        self.my_total_dist = dmin;
        let (own, other_index) = (end.own, 1 - end.own);
        self.my_total_points[own] = end.point;
        self.my_total_pars[own] = end.parameter;
        self.my_total_points[other_index] = proj.nearest_point()?;
        self.my_total_pars[other_index] = proj.lower_distance_parameter()?;
        Ok(true)
    }
}
