// Rust translation of Extrema_ExtElC2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_ExtElC2d`: the extrema between two elementary 2D curves in closed form
//! (a line and a line or conic, a circle and a circle or conic).
use super::ext_p_el_c2d::ExtPElC2d;
use super::pair_result::CurvePairExtrema;
use super::pon::POnCurv2d;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib::{self, ElementaryCurve};
use rustycad_tkmath::gp::{
    Circle2, Direction2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2, Vector2,
};
use std::f64::consts::PI;
use std::ops::Deref;

/// `Extrema_ExtElC2d`: one constructor per curve pair; the accessors are those of
/// `CurvePairExtrema`, reached through `Deref`;
/// `ExtElC2d::lines(line1, line2, 1e-10).nb_ext()? == 1`.
#[derive(Clone, Debug)]
pub struct ExtElC2d {
    result: CurvePairExtrema<Point2, 8>,
}

impl Default for ExtElC2d {
    fn default() -> Self {
        Self {
            result: CurvePairExtrema::new("Extrema_ExtElC2d"),
        }
    }
}

impl Deref for ExtElC2d {
    type Target = CurvePairExtrema<Point2, 8>;
    fn deref(&self) -> &Self::Target {
        &self.result
    }
}

impl ExtElC2d {
    /// `Extrema_ExtElC2d()`: not done; `ExtElC2d::new().is_done() == false`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_ExtElC2d(gp_Lin2d, gp_Lin2d, AngTol)`: parallel lines (within
    /// Precision::Angular(); the tolerance argument is unused) give their square
    /// distance without points, other lines their intersection (Cramer's rule) at
    /// square distance 0; `ExtElC2d::lines(line1, line2, 1e-10)?`.
    pub fn lines(first: Line2, second: Line2, _ang_tol: f64) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        let d1: Vector2 = Vector2::from(first.direction());
        let d2: Vector2 = Vector2::from(second.direction());
        if d1.is_parallel(d2, precision::angular())? {
            extrema
                .result
                .set_parallel(second.square_distance(first.location()));
            return Ok(extrema);
        }
        // Vector from P1 to P2 (P2 - P1); no division by zero, the lines are not parallel.
        let p1p2: Vector2 = Vector2::between(first.location(), second.location());
        let delim: f64 = 1. / d1.crossed(d2);
        let param1: f64 = p1p2.crossed(d2) * delim;
        let param2: f64 = -(d1.crossed(p1p2)) * delim; // -1.0 coefficient before t2.
        let p1: Point2 = elclib::value(param1, &first);
        let p2: Point2 = elclib::value(param2, &second);
        extrema
            .result
            .push(0.0, POnCurv2d::new(param1, p1), POnCurv2d::new(param2, p2));
        extrema.result.done = true;
        Ok(extrema)
    }

    // Stores the extremum of the conic parameter `u2`: `u1 = O1P2.D`.
    fn push_conic_parameter<C>(&mut self, line: &Line2, conic: &C, u2: f64)
    where
        C: ElementaryCurve<Point = Point2>,
    {
        let p2: Point2 = conic.value(u2);
        let u1: f64 = Vector2::between(line.location(), p2).dot(Vector2::from(line.direction()));
        let p1: Point2 = elclib::value(u1, line);
        let sq_dist: f64 = p1.square_distance(p2);
        self.result
            .push(sq_dist, POnCurv2d::new(u1, p1), POnCurv2d::new(u2, p2));
    }

    // The two parameters `teta` and `teta + PI` of the line-circle and line-ellipse
    // cases, the first brought into `[0, 2 PI)`.
    fn push_opposite_pair<C>(&mut self, line: &Line2, conic: &C, teta: f64)
    where
        C: ElementaryCurve<Point = Point2>,
    {
        let second: f64 = teta + PI;
        let first: f64 = if teta < 0.0 { teta + 2.0 * PI } else { teta };
        self.push_conic_parameter(line, conic, first);
        self.push_conic_parameter(line, conic, second);
        self.result.done = true;
    }

    /// `Extrema_ExtElC2d(gp_Lin2d, gp_Circ2d, Tol)`: the two points whose tangent is
    /// parallel to the line (`atan(-Dx/Dy)`, PI/2 for a line along X within
    /// RealEpsilon()); the tolerance is unused;
    /// `ExtElC2d::line_circle(line, circle, 1e-10)?`.
    pub fn line_circle(line: Line2, circle: Circle2, _tol: f64) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        let d: Direction2 = line.direction();
        let dx: f64 = d.dot(circle.x_axis().direction());
        let dy: f64 = d.dot(circle.y_axis().direction());
        let teta: f64 = if dy.abs() <= f64::EPSILON {
            PI / 2.0
        } else {
            (-dx / dy).atan()
        };
        extrema.push_opposite_pair(&line, &circle, teta);
        Ok(extrema)
    }

    /// `Extrema_ExtElC2d(gp_Lin2d, gp_Elips2d)`: `atan(-Dx r / (Dy R))` and half a
    /// turn further; `ExtElC2d::line_ellipse(line, ellipse)?`.
    pub fn line_ellipse(line: Line2, ellipse: Ellipse2) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        let d: Direction2 = line.direction();
        let dx: f64 = d.dot(ellipse.x_axis().direction());
        let dy: f64 = d.dot(ellipse.y_axis().direction());
        let (r1, r2) = (ellipse.major_radius(), ellipse.minor_radius());
        let teta: f64 = if dy.abs() <= f64::EPSILON {
            PI / 2.0
        } else {
            (-dx * r2 / (dy * r1)).atan()
        };
        extrema.push_opposite_pair(&line, &ellipse, teta);
        Ok(extrema)
    }

    /// `Extrema_ExtElC2d(gp_Lin2d, gp_Hypr2d)`: the point whose tangent is parallel
    /// to the line, `ln(sqrt(v))` with `v = (R + r Dx/Dy) / (R - r Dx/Dy)` (parameter 0
    /// when `v <= 0`); not done for a line along X or an asymptotic direction;
    /// `ExtElC2d::line_hyperbola(line, hyperbola)?`.
    pub fn line_hyperbola(line: Line2, hyperbola: Hyperbola2) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        let d: Direction2 = line.direction();
        let dx: f64 = d.dot(hyperbola.x_axis().direction());
        let dy: f64 = d.dot(hyperbola.y_axis().direction());
        let (r_major, r_minor) = (hyperbola.major_radius(), hyperbola.minor_radius());
        if dy.abs() < f64::EPSILON || (r_major - r_minor * dx / dy).abs() < f64::EPSILON {
            return Ok(extrema);
        }
        let v2: f64 = (r_major + r_minor * dx / dy) / (r_major - r_minor * dx / dy);
        let u2: f64 = if v2 > 0.0 { v2.sqrt().ln() } else { 0.0 };
        extrema.push_conic_parameter(&line, &hyperbola, u2);
        extrema.result.done = true;
        Ok(extrema)
    }

    /// `Extrema_ExtElC2d(gp_Lin2d, gp_Parab2d)`: the point at `Dx P / Dy` in the
    /// mirror-axis frame; not done for a line along the axis;
    /// `ExtElC2d::line_parabola(line, parabola)?`.
    pub fn line_parabola(line: Line2, parabola: Parabola2) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        let d: Direction2 = line.direction();
        let dx: f64 = d.dot(parabola.axis_of_symmetry().direction());
        let dy: f64 = d.dot(parabola.axis().y_axis().direction());
        if dy.abs() < f64::EPSILON {
            return Ok(extrema);
        }
        extrema.push_conic_parameter(&line, &parabola, dx * parabola.parameter() / dy);
        extrema.result.done = true;
        Ok(extrema)
    }
}

impl ExtElC2d {
    /// `Extrema_ExtElC2d(gp_Circ2d, gp_Circ2d)`: always done; concentric circles
    /// (centres within Precision::Confusion()) are parallel with the square radius
    /// difference, other pairs give the four points on the line of centres;
    /// `ExtElC2d::circles(circle1, circle2)?`.
    pub fn circles(first: Circle2, second: Circle2) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.result.done = true;
        let (o1, o2) = (first.location(), second.location());
        let d_o1o2: Vector2 = Vector2::between(o1, o2);
        let sq_d_centers: f64 = d_o1o2.square_magnitude();
        if sq_d_centers < precision::square_confusion() {
            let dr: f64 = first.radius() - second.radius();
            extrema.result.set_parallel(dr * dr);
            return Ok(extrema);
        }
        let o1o2: Vector2 = d_o1o2 / sq_d_centers.sqrt();
        let (r1, r2) = (first.radius(), second.radius());
        let p1: [Point2; 2] = [o1.translated(o1o2 * r1), o1.translated(o1o2 * -r1)];
        let u1: [f64; 2] = [
            elclib::parameter(&first, p1[0])?,
            elclib::parameter(&first, p1[1])?,
        ];
        let p2: [Point2; 2] = [o2.translated(o1o2 * r2), o2.translated(o1o2 * -r2)];
        let u2: [f64; 2] = [
            elclib::parameter(&second, p2[0])?,
            elclib::parameter(&second, p2[1])?,
        ];
        for no_sol in 0..2 {
            for kk in 0..2 {
                let sq_dist: f64 = p2[kk].square_distance(p1[no_sol]);
                let on1 = POnCurv2d::new(u1[no_sol], p1[no_sol]);
                extrema
                    .result
                    .push(sq_dist, on1, POnCurv2d::new(u2[kk], p2[kk]));
            }
        }
        Ok(extrema)
    }

    // The circle-conic cases: the extrema from the circle centre to the conic, then
    // from each such conic point to the circle; done once a conic point is visited.
    fn circle_conic(&mut self, circle: &Circle2, conic_extrema: &ExtPElC2d) -> Result<(), Failure> {
        if !conic_extrema.is_done() {
            return Ok(());
        }
        for i in 1..=conic_extrema.nb_ext()? {
            let on_conic = conic_extrema.point(i)?;
            let tol: f64 = precision::confusion();
            let on_circle = ExtPElC2d::with_circle(*on_conic.value(), *circle, tol, 0.0, 2.0 * PI)?;
            if on_circle.is_done() {
                for j in 1..=on_circle.nb_ext()? {
                    let sq_dist: f64 = on_circle.square_distance(j)?;
                    self.result.push(sq_dist, on_circle.point(j)?, on_conic);
                }
            }
            self.result.done = true;
        }
        Ok(())
    }

    /// `Extrema_ExtElC2d(gp_Circ2d, gp_Elips2d)`: through Extrema_ExtPElC2d on
    /// `[0, 2 PI]`; `ExtElC2d::circle_ellipse(circle, ellipse)?`.
    pub fn circle_ellipse(circle: Circle2, ellipse: Ellipse2) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        let tol: f64 = precision::confusion();
        let conic = ExtPElC2d::with_ellipse(circle.location(), ellipse, tol, 0.0, 2.0 * PI)?;
        extrema.circle_conic(&circle, &conic)?;
        Ok(extrema)
    }

    /// `Extrema_ExtElC2d(gp_Circ2d, gp_Hypr2d)`: through Extrema_ExtPElC2d on
    /// `[RealFirst(), RealLast()]`; `ExtElC2d::circle_hyperbola(circle, hyperbola)?`.
    pub fn circle_hyperbola(circle: Circle2, hyperbola: Hyperbola2) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        let tol: f64 = precision::confusion();
        let conic =
            ExtPElC2d::with_hyperbola(circle.location(), hyperbola, tol, f64::MIN, f64::MAX)?;
        extrema.circle_conic(&circle, &conic)?;
        Ok(extrema)
    }

    /// `Extrema_ExtElC2d(gp_Circ2d, gp_Parab2d)`: through Extrema_ExtPElC2d on
    /// `[RealFirst(), RealLast()]`; `ExtElC2d::circle_parabola(circle, parabola)?`.
    pub fn circle_parabola(circle: Circle2, parabola: Parabola2) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        let tol: f64 = precision::confusion();
        let conic = ExtPElC2d::with_parabola(circle.location(), parabola, tol, f64::MIN, f64::MAX)?;
        extrema.circle_conic(&circle, &conic)?;
        Ok(extrema)
    }
}
