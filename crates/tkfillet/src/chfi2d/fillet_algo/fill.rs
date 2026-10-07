// Rust translation of ChFi2d_FilletAlgo::FillPoint, FillDiff and IsRadiusIntersected
// (ChFi2d_FilletAlgo.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fillet function at a point of the first curve: the center of the circle tangent
//! there, its projections on the second curve, and the derivative by a small shift.
use super::{ChFi2dFilletAlgo, FilletPoint};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkgeomalgo::geom2d_api::{InterCurveCurve, ProjectPointOnCurve};
use rustycad_tkmath::gp::{Direction2, Point2, Vector2};

/// The curve behind a handle.
pub(super) fn live(curve: &Handle<dyn Geom2dCurve>) -> Result<&dyn Geom2dCurve, Failure> {
    curve.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ChFi2d_FilletAlgo: null curve; expected a curve in the plane",
        )
    })
}

/// The point of the first curve where the circle center is searched, and the center.
struct Tangency {
    valid: bool,
    param: f64,
    point: Point2,
    center: Point2,
}

impl ChFi2dFilletAlgo {
    /// `FillPoint(thePoint, theLimit)`: from the parameter of the point towards
    /// `limit` by doubling steps, the first point of the first curve off the second
    /// curve (the intersection point); there the fillet center, and the values of the
    /// function for each projection of the center on the second curve.
    pub(super) fn fill_point(&self, point: &mut FilletPoint, limit: f64) -> Result<(), Failure> {
        let mut t = self.tangency(point.get_param(), limit)?;
        if t.valid {
            point.set_param(t.param);
            point.set_center(t.center);
            t.valid = !is_radius_intersected(
                &self.curve2,
                (self.start2, self.end2),
                (t.point, t.center),
                true,
            )?;
        }
        let proj = ProjectPointOnCurve::with_curve(t.center, &self.curve2)?;
        for a in (1..=proj.nb_points()).rev() {
            if t.point.square_distance(proj.point(a)?) < precision::confusion() {
                continue;
            }
            self.append_projection(point, &proj, a, &t)?;
        }
        Ok(())
    }

    /// The search of FillPoint: steps from `param` (doubled each time) until a point of
    /// the first curve is off the second curve or `limit` is passed.
    fn tangency(&self, param: f64, limit: f64) -> Result<Tangency, Failure> {
        let mut step = precision::confusion();
        if limit < param {
            step = -step;
        }
        let mut t = Tangency {
            valid: false,
            param,
            point: Point2::default(),
            center: Point2::default(),
        };
        loop {
            if (t.param - step - limit) * (t.param - limit) <= 0.0 {
                return Ok(t); // limit was exceeded
            }
            step *= 2.0;
            let (point, vec) = live(&self.curve1)?.d1(t.param)?;
            t.point = point;
            if vec.square_magnitude() >= precision::confusion() {
                t.center = point.translated(self.perpendicular(vec)?);
                if self.off_second_curve(point)? {
                    t.valid = true;
                    return Ok(t);
                }
            }
            t.param += step;
        }
    }

    /// The vector to the center: the tangent turned to the side of the fillet, at the
    /// radius.
    fn perpendicular(&self, vec: Vector2) -> Result<Vector2, Failure> {
        let (sx, sy) = if self.start_side {
            (-1.0, 1.0)
        } else {
            (1.0, -1.0)
        };
        let perp = Vector2::new(sx * vec.y(), sy * vec.x()).normalized()?;
        Ok(perp * self.radius)
    }

    /// Whether the point is not on the second curve (no projection, or the nearest one
    /// beyond Precision::Confusion).
    fn off_second_curve(&self, point: Point2) -> Result<bool, Failure> {
        let proj = ProjectPointOnCurve::with_range(point, &self.curve2, self.start2, self.end2)?;
        Ok(proj.nb_points() == 0 || point.distance(proj.nearest_point()?) > precision::confusion())
    }

    /// One value of the function: the square distance to the projection `a` minus the
    /// square radius, valid inside the range of the second curve when the radius to it
    /// crosses no curve; the parameter of the projection when it is at the radius.
    fn append_projection(
        &self,
        point: &mut FilletPoint,
        proj: &ProjectPointOnCurve,
        a: i32,
        t: &Tangency,
    ) -> Result<(), Failure> {
        let mut valid2 = t.valid;
        if valid2 {
            let ends = (t.center, proj.point(a)?);
            valid2 = !is_radius_intersected(&self.curve1, (self.start1, self.end1), ends, false)?;
        }
        // checking the right parameter
        let mut param_proj = proj.parameter(a)?;
        let curve2 = live(&self.curve2)?;
        while curve2.is_periodic() && param_proj < self.start2 {
            param_proj += curve2.period()?;
        }
        let d = proj.distance(a)?;
        let inside = param_proj >= self.start2 && param_proj <= self.end2;
        point.append_value(d * d - self.radius * self.radius, inside && valid2);
        if (d - self.radius).abs() < precision::confusion() {
            point.set_param2(param_proj);
        }
        Ok(())
    }

    /// `FillDiff(thePoint, theDiffStep, theFront)`: the derivative from the point shifted
    /// by the step (forward or backward), or from the other side when the values do not
    /// match. The shifted point is filled up to `delta * 999` (not shifted), as natively.
    pub(super) fn fill_diff(
        &self,
        point: &mut FilletPoint,
        diff_step: f64,
        front: bool,
    ) -> Result<(), Failure> {
        let delta = if front { diff_step } else { -diff_step };
        let mut diff = FilletPoint::new(point.get_param() + delta);
        self.fill_point(&mut diff, delta * 999.0)?;
        if !point.calculate_diff(&diff)? {
            diff.set_param(point.get_param() - delta);
            self.fill_point(&mut diff, -delta * 999.0)?;
            point.calculate_diff(&diff)?;
        }
        Ok(())
    }
}

/// `IsRadiusIntersected(theCurve, theCurveMin, theCurveMax, theStart, theEnd,
/// theStartConnected)`: whether the segment from the start to the end of a radius of
/// the fillet crosses the curve (between its bounds); a crossing at the start counts
/// unless the start is connected to the curve.
fn is_radius_intersected(
    curve: &Handle<dyn Geom2dCurve>,
    (min, max): (f64, f64),
    (start, end): (Point2, Point2),
    start_connected: bool,
) -> Result<bool, Failure> {
    if start.square_distance(end) < precision::square_confusion() {
        return Ok(false);
    }
    let dir = Direction2::from_vector(Vector2::between(start, end))?;
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(start, dir)).cast();
    let inter = InterCurveCurve::between(curve, &line, precision::confusion())?;
    for a in (1..=inter.nb_points()).rev() {
        let point = inter.point(a)?;
        let proj = ProjectPointOnCurve::with_range(point, curve, min, max)?;
        if proj.nb_points() < 1 || proj.lower_distance_parameter()? > precision::confusion() {
            continue; // point is not on edge
        }
        if crosses(point, (start, end), start_connected)? {
            return Ok(true);
        }
    }
    let c = live(curve)?;
    for _ in (1..=inter.nb_segments()).rev() {
        // anInter.Segment(a, aCurve); //not implemented (bug in OCC): the curve ends
        for u in [c.first_parameter(), c.last_parameter()] {
            let point = c.value(u)?;
            let proj = ProjectPointOnCurve::with_range(point, curve, min, max)?;
            let on_edge =
                proj.nb_points() != 0 && proj.lower_distance_parameter()? < precision::confusion();
            if on_edge && crosses(point, (start, end), start_connected)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Whether a point of the curve stops the radius: at the start (unless connected), at
/// the end, or between them.
fn crosses(
    point: Point2,
    (start, end): (Point2, Point2),
    start_connected: bool,
) -> Result<bool, Failure> {
    if point.distance(start) < precision::confusion() && !start_connected {
        return Ok(true);
    }
    if point.distance(end) < precision::confusion() {
        return Ok(true);
    }
    Vector2::between(point, start).is_opposite(Vector2::between(point, end), precision::angular())
}
