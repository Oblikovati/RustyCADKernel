// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecPC.cxx (Perform, ComputeIntervals, SearchBound and Parameter) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The construction of a curve-point bisector: the interval of the curve where the
//! bisector point is closer than `DistMax`, then the extensions of a concave curve.
use super::state::{PcState, PcValues};
use crate::bisector::is_convex;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dGeometry, errors};
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};

// A private copy of the curve, as `occ::down_cast<Geom2d_Curve>(Cu->Copy())`.
pub(crate) fn copy_curve(cu: &Handle<dyn Geom2dCurve>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let copy: Handle<dyn Geom2dGeometry> = cu.copy()?;
    Ok(copy.cast())
}

// The lowest-distance parameter of the projection of `p` on `curve`, if any.
fn project(p: Point2, curve: &Handle<dyn Geom2dCurve>) -> Result<Option<f64>, Failure> {
    let (first, last) = (curve.first_parameter(), curve.last_parameter());
    let projection = ProjectPointOnCurve::with_range(p, curve, first, last)?;
    if projection.nb_points() > 0 {
        return Ok(Some(projection.lower_distance_parameter()?));
    }
    Ok(None)
}

impl PcState {
    /// The point of parameter `u`, as `Value(U)` evaluates it (`EvalD0`).
    pub(crate) fn value(&self, u: f64) -> Result<Point2, Failure> {
        let start = PcValues {
            p: self.point,
            ..PcValues::default()
        };
        Ok(self.values(u, 0, start)?.p)
    }
    /// `Perform(Cu, P, Side, DistMax)` on the current fields: the intervals are
    /// appended to the existing ones, as natively.
    pub(crate) fn perform(
        &mut self,
        cu: &Handle<dyn Geom2dCurve>,
        p: Point2,
        side: f64,
        dist_max: f64,
    ) -> Result<(), Failure> {
        self.curve = copy_curve(cu)?;
        self.point = p;
        self.dist_max = dist_max;
        self.sign = side;
        self.is_convex = is_convex(&self.curve, self.sign)?;
        self.compute_intervals()?;
        if self.is_empty {
            return Ok(());
        }
        self.bis_interval = 1;
        self.extension_start = false;
        self.extension_end = false;
        self.point_start_bis = self.value(self.start_intervals.first()?)?;
        self.point_end_bis = self.value(self.end_intervals.last()?)?;
        if !self.is_convex {
            self.add_extension(p)?;
        }
        Ok(())
    }
    // A concave curve whose end is the point is extended by a half line there.
    fn add_extension(&mut self, p: Point2) -> Result<(), Failure> {
        let curve_start: Point2 = self.curve.value(self.curve.first_parameter())?;
        if self.point.is_equal(curve_start, precision::confusion()) {
            self.extension_start = true;
            let u_first: f64 = self.start_intervals.first()? - p.distance(self.point_start_bis);
            self.start_intervals.prepend(u_first);
            self.end_intervals.prepend(self.start_intervals.value(2)?);
            self.bis_interval = 2;
            return Ok(());
        }
        let curve_end: Point2 = self.curve.value(self.curve.last_parameter())?;
        if self.point.is_equal(curve_end, precision::confusion()) {
            self.extension_end = true;
            let u_last: f64 = self.end_intervals.last()? + p.distance(self.point_end_bis);
            self.start_intervals.append(self.end_intervals.last()?);
            self.end_intervals.append(u_last);
            self.bis_interval = 1;
        }
        Ok(())
    }
    /// `SearchBound(U1, U2)`: the dichotomy between a parameter closer than
    /// `DistMax` and one farther.
    pub(crate) fn search_bound(&self, u1: f64, u2: f64) -> Result<f64, Failure> {
        let tol: f64 = precision::p_confusion();
        let dist_max2: f64 = self.dist_max * self.dist_max;
        let (mut u11, mut u22) = (u1, u2);
        let mut dist1: f64 = self.distance(u11)?;
        let mut umid: f64 = 0.;
        while (u22 - u11) > tol {
            umid = 0.5 * (u22 + u11);
            let dist_mid: f64 = self.distance(umid)?;
            if (dist1 > dist_max2) == (dist_mid > dist_max2) {
                u11 = umid;
                dist1 = dist_mid;
            } else {
                u22 = umid;
            }
        }
        Ok(umid)
    }
    /// `ComputeIntervals()`: the part of the curve whose bisector points are closer
    /// than `DistMax`, or an empty bisector.
    pub(crate) fn compute_intervals(&mut self) -> Result<(), Failure> {
        self.is_empty = false;
        self.shift_parameter = 0.;
        let dist_max2: f64 = self.dist_max * self.dist_max;
        let (u1, u2) = (self.curve.first_parameter(), self.curve.last_parameter());
        let dist1: f64 = self.distance(u1)?;
        let dist2: f64 = self.distance(u2)?;
        let projection: Option<f64> = project(self.point, &self.curve)?;
        let (u_proj, dist_proj) = match projection {
            Some(u) => (u, self.distance(u)?),
            None => (0., precision::infinite()),
        };
        if dist1 < dist_max2 && dist2 < dist_max2 {
            if dist_proj > dist_max2 && projection.is_some() {
                self.is_empty = true;
            } else {
                self.start_intervals.append(u1);
                self.end_intervals.append(u2);
            }
            return Ok(());
        }
        let Some((u_start, u_end)) =
            self.bounds(u1, u2, u_proj, [dist1, dist2, dist_proj], dist_max2)?
        else {
            self.is_empty = true;
            return Ok(());
        };
        self.start_intervals.append(u_start);
        self.end_intervals.append(u_end);
        self.shift_periodic()
    }
    // The bounds of the interval when an end of the curve is too far; `None` when
    // both ends and the projection are.
    fn bounds(
        &self,
        u1: f64,
        u2: f64,
        u_proj: f64,
        dists: [f64; 3],
        dist_max2: f64,
    ) -> Result<Option<(f64, f64)>, Failure> {
        let [dist1, dist2, dist_proj] = dists;
        if dist1 > dist_max2 && dist2 > dist_max2 {
            if dist_proj < dist_max2 {
                let u_start: f64 = self.search_bound(u1, u_proj)?;
                return Ok(Some((u_start, self.search_bound(u_proj, u2)?)));
            }
            return Ok(None);
        }
        if dist1 < dist_max2 {
            return Ok(Some((u1, self.search_bound(u1, u2)?)));
        }
        if dist2 < dist_max2 {
            return Ok(Some((self.search_bound(u1, u2)?, u2)));
        }
        // Natively both bounds stay 0 when a distance equals DistMax^2 exactly.
        Ok(Some((0., 0.)))
    }
    // On a periodic curve whose several intervals cover its origin, the first and
    // the last are merged and the parameters shifted; the shift reads the period of
    // the bisector itself, which is not periodic, so it raises
    // Standard_NoSuchObject (reached only when Perform runs twice on one object).
    fn shift_periodic(&mut self) -> Result<(), Failure> {
        if !self.curve.is_periodic() || self.start_intervals.len() <= 1 {
            return Ok(());
        }
        if self.end_intervals.last()? != self.curve.last_parameter()
            || self.start_intervals.first()? != self.curve.first_parameter()
        {
            return Ok(());
        }
        self.start_intervals.remove(1)?;
        self.end_intervals.remove(self.end_intervals.len())?;
        Err(errors::no_such_object(
            "Geom2d_Curve::Period: Bisector_BisecPC is not periodic; expected IsPeriodic()",
        ))
    }
    /// `Parameter(P)`: the bounds, then the extensions, then the projection on the
    /// curve.
    pub(crate) fn parameter(&self, p: Point2) -> Result<f64, Failure> {
        let tol: f64 = precision::confusion();
        if p.is_equal(self.point_start_bis, tol) {
            return self.start_intervals.value(self.bis_interval);
        }
        if p.is_equal(self.point_end_bis, tol) {
            return self.end_intervals.value(self.bis_interval);
        }
        if self.extension_start {
            let dir = Direction2::new(
                self.point_start_bis.x() - p.x(),
                self.point_start_bis.y() - p.y(),
            )?;
            if let Some(u) = on_extension(self.point_start_bis, dir, p, |u| u < 0.) {
                return Ok(u + self.start_intervals.value(self.bis_interval)?);
            }
        }
        if self.extension_end {
            let dir = Direction2::new(
                p.x() - self.point_end_bis.x(),
                p.y() - self.point_end_bis.y(),
            )?;
            if let Some(u) = on_extension(self.point_end_bis, dir, p, |u| u > 0.) {
                return Ok(u + self.end_intervals.value(self.bis_interval)?);
            }
        }
        let u_on_curve: f64 = project(p, &self.curve)?.unwrap_or(0.);
        Ok(self.link_curve_bis(u_on_curve))
    }
}

// The parameter of `p` on the half line from `origin` along `dir` when `p` is on it
// and the parameter passes `side`.
fn on_extension(
    origin: Point2,
    dir: Direction2,
    p: Point2,
    side: impl Fn(f64) -> bool,
) -> Option<f64> {
    let axe = Axis2d::new(origin, dir);
    let u: f64 = elclib::line_parameter_2d(axe, p);
    let proj: Point2 = elclib::line_value_2d(u, axe);
    (proj.is_equal(p, precision::confusion()) && side(u)).then_some(u)
}
