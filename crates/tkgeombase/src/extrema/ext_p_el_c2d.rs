// Rust translation of Extrema_ExtPElC2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_ExtPElC2d`: the extrema of the distance between a 2D point and a segment
//! of an elementary 2D curve, in closed form. Its code differs from the 3D class:
//! the circle marks both extrema as minima, the ellipse, hyperbola and parabola none,
//! and a line outside its bounds is done with no extremum.
use super::curve_tools::Curve2dTool;
use super::elementary_point_curve::{ElementaryPointCurve, PointCurveExtrema};
use super::ext_p_el_c::{adjust_pair, polynomial_roots};
use super::pon::POnCurv2d;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{
    Circle2, Direction2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2, Vector2, Xy,
};
use rustycad_tkmath::math::{DirectPolynomialRoots, TrigonometricRoots};
use std::f64::consts::PI;
use std::ops::Deref;

/// `Extrema_ExtPElC2d`. The result accessors are those of `PointCurveExtrema`,
/// reached through `Deref`;
/// `ExtPElC2d::with_line(point, line, 1e-7, -1.0, 1.0).nb_ext()? == 1`.
#[derive(Clone, Debug)]
pub struct ExtPElC2d {
    result: PointCurveExtrema<Point2>,
}

impl Default for ExtPElC2d {
    fn default() -> Self {
        Self {
            result: PointCurveExtrema::new("Extrema_ExtPElC2d"),
        }
    }
}

impl Deref for ExtPElC2d {
    type Target = PointCurveExtrema<Point2>;
    fn deref(&self) -> &Self::Target {
        &self.result
    }
}

impl ExtPElC2d {
    /// `Extrema_ExtPElC2d()`: not done; `ExtPElC2d::new().is_done() == false`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_ExtPElC2d(P, gp_Lin2d, Tol, Uinf, Usup)`;
    /// `ExtPElC2d::with_line(point, line, 1e-7, -1.0, 1.0)`.
    pub fn with_line(point: Point2, line: Line2, tol: f64, uinf: f64, usup: f64) -> Self {
        let mut extrema: Self = Self::new();
        extrema.perform_line(point, &line, tol, uinf, usup);
        extrema
    }
    /// `Perform(P, gp_Lin2d, Tol, Uinf, Usup)`: always done; one extremum when the
    /// projection parameter lies in `[Uinf - Tol, Usup + Tol]`;
    /// `extrema.perform_line(point, &line, 1e-7, -1.0, 1.0)`.
    pub fn perform_line(&mut self, point: Point2, line: &Line2, tol: f64, uinf: f64, usup: f64) {
        self.result.done = true;
        self.result.nb_ext = 0;
        let v1: Vector2 = Vector2::from(line.direction());
        let origin: Point2 = line.location();
        let distance: f64 = v1.dot(Vector2::between(origin, point));
        if (distance >= uinf - tol) && (distance <= usup + tol) {
            let projection: Point2 = origin.translated(v1 * distance);
            let sq_dist: f64 = point.square_distance(projection);
            self.result
                .push(sq_dist, true, POnCurv2d::new(distance, projection));
        }
    }
    /// `Extrema_ExtPElC2d(P, gp_Circ2d, Tol, Uinf, Usup)`;
    /// `ExtPElC2d::with_circle(point, circle, 1e-7, 0.0, 2.0 * PI)?`.
    pub fn with_circle(
        point: Point2,
        circle: Circle2,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.perform_circle(point, &circle, tol, uinf, usup)?;
        Ok(extrema)
    }
    /// `Perform(P, gp_Circ2d, Tol, Uinf, Usup)`: the two points on the line through
    /// the centre, both flagged as minima; not done when the point is within
    /// Precision::Confusion() of the centre;
    /// `extrema.perform_circle(point, &circle, 1e-7, 0.0, 2.0 * PI)?`.
    pub fn perform_circle(
        &mut self,
        point: Point2,
        circle: &Circle2,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        let center: Point2 = circle.location();
        self.result.nb_ext = 0;
        if center.is_equal(point, precision::confusion()) {
            self.result.done = false;
            return Ok(());
        }
        self.result.done = true;
        let direction: Vector2 =
            Vector2::from(Direction2::from_vector(Vector2::between(point, center))?);
        let radius: f64 = circle.radius();
        let near: Point2 = center.translated(direction * radius);
        let far: Point2 = center.translated(direction * -radius);
        let u1: f64 = elclib::parameter(circle, near)?;
        let [u1, u2] = adjust_pair(uinf, precision::p_confusion(), [u1, u1 + PI]);
        let (u1, near) = snap_to_start(circle, (u1, near), uinf, tol);
        let (u2, far) = snap_to_start(circle, (u2, far), uinf, tol);
        for (u, p) in [(u1, near), (u2, far)] {
            if ((uinf - u) < tol) && ((u - usup) < tol) {
                self.result
                    .push(point.square_distance(p), true, POnCurv2d::new(u, p));
            }
        }
        Ok(())
    }
}

// A solution one period away from `uinf` (within Tol) moves to `uinf`, with its
// point recomputed as `O + R (cos U X + sin U Y)`.
fn snap_to_start(circle: &Circle2, (u, p): (f64, Point2), uinf: f64, tol: f64) -> (f64, Point2) {
    let shift: f64 = u - 2.0 * PI - uinf;
    if !(shift < tol && shift > -tol) {
        return (u, p);
    }
    let x: Xy = circle.x_axis().direction().xy() * uinf.cos();
    let y: Xy = circle.y_axis().direction().xy() * uinf.sin();
    (
        uinf,
        Point2::from(circle.location().xy() + (x + y) * circle.radius()),
    )
}

impl ExtPElC2d {
    /// `Extrema_ExtPElC2d(P, gp_Elips2d, Tol, Uinf, Usup)`;
    /// `ExtPElC2d::with_ellipse(point, ellipse, 1e-7, 0.0, 2.0 * PI)?`.
    pub fn with_ellipse(
        point: Point2,
        ellipse: Ellipse2,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.perform_ellipse(point, &ellipse, tol, uinf, usup)?;
        Ok(extrema)
    }
    /// `Perform(P, gp_Elips2d, Tol, Uinf, Usup)`: the roots of the trigonometric
    /// equation of the 3D class, none flagged as a minimum (`NoSol == 0` never holds);
    /// not done at the centre of a circular ellipse or when the solver fails;
    /// `extrema.perform_ellipse(point, &ellipse, 1e-7, 0.0, 2.0 * PI)?`.
    pub fn perform_ellipse(
        &mut self,
        point: Point2,
        ellipse: &Ellipse2,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        self.result.done = false;
        self.result.nb_ext = 0;
        let origin: Point2 = ellipse.location();
        let (a, b) = (ellipse.major_radius(), ellipse.minor_radius());
        let v: Vector2 = Vector2::between(origin, point);
        if origin.is_equal(point, precision::confusion()) && ((a - b).abs() <= tol) {
            return Ok(());
        }
        let x: f64 = v.dot(Vector2::from(ellipse.x_axis().direction()));
        let y: f64 = v.dot(Vector2::from(ellipse.y_axis().direction()));
        let roots =
            TrigonometricRoots::new(0., (b * b - a * a) / 2., -b * y, a * x, 0., uinf, usup);
        if !roots.is_done() {
            return Ok(());
        }
        for index in 1..=roots.solutions()? {
            let us: f64 = roots.value(index)?;
            let cu: Point2 = elclib::value(us, ellipse);
            self.result
                .push(cu.square_distance(point), false, POnCurv2d::new(us, cu));
        }
        self.result.done = true;
        Ok(())
    }
    /// `Extrema_ExtPElC2d(P, gp_Hypr2d, Tol, Uinf, Usup)`;
    /// `ExtPElC2d::with_hyperbola(point, hyperbola, 1e-7, -2.0, 2.0)?`.
    pub fn with_hyperbola(
        point: Point2,
        hyperbola: Hyperbola2,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.perform_hyperbola(point, &hyperbola, tol, uinf, usup)?;
        Ok(extrema)
    }
}

impl ExtPElC2d {
    /// `Perform(P, gp_Hypr2d, Tol, Uinf, Usup)`: the quartic of the 3D class; points
    /// closer than Tol count once, none is flagged as a minimum;
    /// `extrema.perform_hyperbola(point, &hyperbola, 1e-7, -2.0, 2.0)?`.
    pub fn perform_hyperbola(
        &mut self,
        point: Point2,
        hyperbola: &Hyperbola2,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        self.result.done = false;
        self.result.nb_ext = 0;
        let (r_major, r_minor) = (hyperbola.major_radius(), hyperbola.minor_radius());
        let opp: Vector2 = Vector2::between(hyperbola.location(), point);
        let x: f64 = opp.dot(Vector2::from(hyperbola.x_axis().direction()));
        let y: f64 = opp.dot(Vector2::from(hyperbola.y_axis().direction()));
        let c1: f64 = (r_major * r_major + r_minor * r_minor) / 4.;
        let b: f64 = -(x * r_major + y * r_minor) / 2.;
        let d: f64 = (x * r_major - y * r_minor) / 2.;
        let roots = DirectPolynomialRoots::quartic(c1, b, 0., d, -c1);
        if !roots.is_done() {
            return Ok(());
        }
        let parameters: Vec<f64> = polynomial_roots(&roots)?
            .into_iter()
            .filter(|&v| v > 0.)
            .map(f64::ln)
            .collect();
        let value = |u: f64| elclib::value(u, hyperbola);
        let never_min = |_: f64, _: f64| false;
        let bounds: (f64, f64) = (uinf, usup);
        self.result
            .push_distinct(point, &parameters, bounds, tol * tol, value, never_min);
        self.result.done = true;
        Ok(())
    }
    /// `Extrema_ExtPElC2d(P, gp_Parab2d, Tol, Uinf, Usup)`;
    /// `ExtPElC2d::with_parabola(point, parabola, 1e-7, -2.0, 2.0)?`.
    pub fn with_parabola(
        point: Point2,
        parabola: Parabola2,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.perform_parabola(point, &parabola, tol, uinf, usup)?;
        Ok(extrema)
    }
    /// `Perform(P, gp_Parab2d, Tol, Uinf, Usup)`: the cubic of the 3D class in the
    /// mirror-axis frame; points closer than Tol count once, none is a minimum;
    /// `extrema.perform_parabola(point, &parabola, 1e-7, -2.0, 2.0)?`.
    pub fn perform_parabola(
        &mut self,
        point: Point2,
        parabola: &Parabola2,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        self.result.done = false;
        self.result.nb_ext = 0;
        let focal: f64 = parabola.focal();
        let opp: Vector2 = Vector2::between(parabola.location(), point);
        let x: f64 = opp.dot(Vector2::from(parabola.axis_of_symmetry().direction()));
        let y: f64 = opp.dot(Vector2::from(parabola.axis().y_axis().direction()));
        let roots =
            DirectPolynomialRoots::cubic(1. / (4. * focal), 0., 2. * focal - x, -2. * focal * y);
        if !roots.is_done() {
            return Ok(());
        }
        let parameters: Vec<f64> = polynomial_roots(&roots)?;
        let value = |u: f64| elclib::value(u, parabola);
        let never_min = |_: f64, _: f64| false;
        let bounds: (f64, f64) = (uinf, usup);
        self.result
            .push_distinct(point, &parameters, bounds, tol * tol, value, never_min);
        self.result.done = true;
        Ok(())
    }
}

impl ElementaryPointCurve<Curve2dTool> for ExtPElC2d {
    fn perform_line(
        &mut self,
        point: Point2,
        line: &Line2,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        ExtPElC2d::perform_line(self, point, line, tol, uinf, usup);
        Ok(())
    }
    fn perform_circle(
        &mut self,
        point: Point2,
        circle: &Circle2,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        ExtPElC2d::perform_circle(self, point, circle, tol, uinf, usup)
    }
    fn perform_ellipse(
        &mut self,
        point: Point2,
        ellipse: &Ellipse2,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        ExtPElC2d::perform_ellipse(self, point, ellipse, tol, uinf, usup)
    }
    fn perform_hyperbola(
        &mut self,
        point: Point2,
        hyperbola: &Hyperbola2,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        ExtPElC2d::perform_hyperbola(self, point, hyperbola, tol, uinf, usup)
    }
    fn perform_parabola(
        &mut self,
        point: Point2,
        parabola: &Parabola2,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        ExtPElC2d::perform_parabola(self, point, parabola, tol, uinf, usup)
    }
    fn point_extrema(&self) -> &PointCurveExtrema<Point2> {
        &self.result
    }
}
