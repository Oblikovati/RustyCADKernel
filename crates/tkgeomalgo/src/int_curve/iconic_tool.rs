// Rust translation of IntCurve_IConicTool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurve_IConicTool`: a gp conic as the implicit curve of IntImpParGen.
use crate::int_imp_par_gen::ImpTool;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{
    self, Axis22d, Circle2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2, Transform2, Vector2,
};
use std::f64::consts::PI;

/// `IntCurve_IConicTool`: a conic by its frame, up to three parameters and the
/// transformation from the absolute frame to the conic's one, with its signed
/// distance: the line equation, the distance to the circle, the radial distance in
/// the ellipse scaled to a circle (`AN_ELIPS` is 0), `Y² - 4fX` for a parabola and
/// `(X/a)² - (Y/b)² - 1` (with `-X²` on the negative branch) for a hyperbola.
///
/// A default tool has no curve (GeomAbs_OtherCurve): its members print an error
/// natively and return the origin, a null vector, 0 or leave the outputs; the port
/// does not print.
///
/// ```
/// use rustycad_tkgeomalgo::int_curve::IConicTool;
/// use rustycad_tkmath::gp::{Axis22d, Circle2, Point2};
/// let tool = IConicTool::from_circle(Circle2::new(Axis22d::default(), 2.0).unwrap());
/// assert_eq!(tool.distance(Point2::new(3.0, 0.0)), 1.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IConicTool {
    prm1: f64,
    prm2: f64,
    prm3: f64,
    axis: Axis22d,
    kind: CurveType,
    abs_to_object: Transform2,
}

impl Default for IConicTool {
    /// `IntCurve_IConicTool()`: no curve. `IConicTool::default()`.
    fn default() -> Self {
        Self {
            prm1: 0.0,
            prm2: 0.0,
            prm3: 0.0,
            axis: Axis22d::default(),
            kind: CurveType::OtherCurve,
            abs_to_object: Transform2::default(),
        }
    }
}

impl IConicTool {
    /// `IntCurve_IConicTool()`. `IConicTool::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    // A tool of `kind` in `axis`, transformed from the absolute frame to it.
    fn in_frame(kind: CurveType, axis: Axis22d, prm: [f64; 3]) -> Self {
        let mut abs_to_object: Transform2 = Transform2::default();
        abs_to_object.set_transformation_between(gp::ox2d(), axis.x_axis());
        Self {
            prm1: prm[0],
            prm2: prm[1],
            prm3: prm[2],
            axis,
            kind,
            abs_to_object,
        }
    }
    /// `IntCurve_IConicTool(gp_Lin2d)`: the line equation `aX + bY + c`; the frame
    /// is built from the line's axis (a gp error that a unit direction cannot
    /// raise is returned). `IConicTool::from_line(line)?`.
    pub fn from_line(line: Line2) -> Result<Self, Failure> {
        let (a, b, c) = line.coefficients();
        Ok(Self {
            prm1: a,
            prm2: b,
            prm3: c,
            axis: Axis22d::from_axis(line.position(), true)?,
            kind: CurveType::Line,
            abs_to_object: Transform2::default(),
        })
    }
    /// `IntCurve_IConicTool(gp_Elips2d)`. `IConicTool::from_ellipse(ellipse)`.
    pub fn from_ellipse(elips: Ellipse2) -> Self {
        let (a, b) = (elips.major_radius(), elips.minor_radius());
        let c: f64 = (a * a - b * b).sqrt();
        Self::in_frame(CurveType::Ellipse, elips.axis(), [a, b, c])
    }
    /// `IntCurve_IConicTool(gp_Circ2d)`. `IConicTool::from_circle(circle)`.
    pub fn from_circle(circle: Circle2) -> Self {
        let axis: Axis22d = circle.axis();
        let center: Point2 = axis.location();
        Self::in_frame(
            CurveType::Circle,
            axis,
            [circle.radius(), center.x(), center.y()],
        )
    }
    /// `IntCurve_IConicTool(gp_Parab2d)`: the focal length and `2p = 4f`.
    /// `IConicTool::from_parabola(parabola)`.
    pub fn from_parabola(parab: Parabola2) -> Self {
        let focal: f64 = parab.focal();
        Self::in_frame(CurveType::Parabola, parab.axis(), [focal, 4.0 * focal, 0.0])
    }
    /// `IntCurve_IConicTool(gp_Hypr2d)`. `IConicTool::from_hyperbola(hyperbola)`.
    pub fn from_hyperbola(hypr: Hyperbola2) -> Self {
        Self::in_frame(
            CurveType::Hyperbola,
            hypr.axis(),
            [hypr.major_radius(), hypr.minor_radius(), 0.0],
        )
    }

    /// `Value(X)`: the point of parameter `x` (the origin without a curve).
    /// `tool.value(0.5)`.
    pub fn value(&self, x: f64) -> Point2 {
        let axis: Axis22d = self.axis;
        match self.kind {
            CurveType::Line => elclib::line_value_2d(x, axis.x_axis()),
            CurveType::Ellipse => elclib::ellipse_value_2d(x, axis, self.prm1, self.prm2),
            CurveType::Circle => elclib::circle_value_2d(x, axis, self.prm1),
            CurveType::Parabola => elclib::parabola_value_2d(x, axis, self.prm1),
            CurveType::Hyperbola => elclib::hyperbola_value_2d(x, axis, self.prm1, self.prm2),
            _ => Point2::new(0.0, 0.0),
        }
    }
    /// `D1(U, P, T)`: the point and tangent (left unchanged without a curve).
    /// `tool.d1(0.5, &mut p, &mut t)`.
    pub fn d1(&self, x: f64, pt: &mut Point2, tan: &mut Vector2) {
        let axis: Axis22d = self.axis;
        let (p, t) = match self.kind {
            CurveType::Line => elclib::line_d1_2d(x, axis.x_axis()),
            CurveType::Ellipse => elclib::ellipse_d1_2d(x, axis, self.prm1, self.prm2),
            CurveType::Circle => elclib::circle_d1_2d(x, axis, self.prm1),
            CurveType::Parabola => elclib::parabola_d1_2d(x, axis, self.prm1),
            CurveType::Hyperbola => elclib::hyperbola_d1_2d(x, axis, self.prm1, self.prm2),
            _ => return,
        };
        (*pt, *tan) = (p, t);
    }
    /// `D2(U, P, T, N)`: the point and the first two derivatives, a null second one
    /// for a line (left unchanged without a curve). `tool.d2(0.5, &mut p, &mut t, &mut n)`.
    pub fn d2(&self, x: f64, pt: &mut Point2, tan: &mut Vector2, norm: &mut Vector2) {
        let axis: Axis22d = self.axis;
        let (p, t, n) = match self.kind {
            CurveType::Line => {
                let (p, t) = elclib::line_d1_2d(x, axis.x_axis());
                (p, t, Vector2::new(0.0, 0.0))
            }
            CurveType::Ellipse => elclib::ellipse_d2_2d(x, axis, self.prm1, self.prm2),
            CurveType::Circle => elclib::circle_d2_2d(x, axis, self.prm1),
            CurveType::Parabola => elclib::parabola_d2_2d(x, axis, self.prm1),
            CurveType::Hyperbola => elclib::hyperbola_d2_2d(x, axis, self.prm1, self.prm2),
            _ => return,
        };
        (*pt, *tan, *norm) = (p, t, n);
    }

    /// `Distance(P)`: the signed distance between the point and the curve (0
    /// without a curve). `tool.distance(point)`.
    pub fn distance(&self, point: Point2) -> f64 {
        match self.kind {
            CurveType::Line => self.prm1 * point.x() + self.prm2 * point.y() + self.prm3,
            CurveType::Ellipse => {
                let p: Point2 = point.transformed(self.abs_to_object);
                let x: f64 = p.x();
                let y: f64 = p.y() * (self.prm1 / self.prm2);
                (x * x + y * y).sqrt() - self.prm1
            }
            CurveType::Circle => {
                let dx: f64 = self.prm2 - point.x();
                let dy: f64 = self.prm3 - point.y();
                (dx * dx + dy * dy).sqrt() - self.prm1
            }
            CurveType::Parabola => {
                let p: Point2 = point.transformed(self.abs_to_object);
                p.y() * p.y() - self.prm2 * p.x()
            }
            CurveType::Hyperbola => self.hyperbola_distance(point),
            _ => 0.0,
        }
    }
    // `(X/a)² - (Y/b)² - 1` for X > 0, `(-X²)/a² - (Y/b)² - 1` otherwise.
    fn hyperbola_distance(&self, point: Point2) -> f64 {
        let p: Point2 = point.transformed(self.abs_to_object);
        let (aa, bb) = (self.prm1 * self.prm1, self.prm2 * self.prm2);
        if p.x() > 0.0 {
            return (p.x() * p.x()) / aa - (p.y() * p.y()) / bb - 1.0;
        }
        (-p.x() * p.x()) / aa - (p.y() * p.y()) / bb - 1.0
    }

    /// `GradDistance(P)`: the gradient of the signed distance at the point (the
    /// hyperbola's negative branch pulls towards positive X; a null vector without
    /// a curve). `tool.grad_distance(point)`.
    pub fn grad_distance(&self, point: Point2) -> Vector2 {
        let p: Point2 = point.transformed(self.abs_to_object);
        let gradient: Vector2 = match self.kind {
            CurveType::Line => return Vector2::new(self.prm1, self.prm2),
            CurveType::Circle => radial_gradient(p.x(), p.y(), 1.0),
            CurveType::Ellipse => radial_gradient(
                p.x(),
                p.y() * (self.prm1 / self.prm2),
                self.prm1 / self.prm2,
            ),
            CurveType::Parabola => Vector2::new(-self.prm2, p.y() + p.y()),
            CurveType::Hyperbola => Vector2::new(
                2.0 * p.x().abs() / (self.prm1 * self.prm1),
                -2.0 * p.y() / (self.prm2 * self.prm2),
            ),
            _ => return Vector2::new(0.0, 0.0),
        };
        self.object_to_abs(gradient)
    }
    // `Gradient.Transform(Object_To_Abs)`: back to the absolute frame. The inverse
    // of a rigid transformation always exists; its failure (a null scale) is not
    // reachable and leaves the vector unchanged.
    fn object_to_abs(&self, gradient: Vector2) -> Vector2 {
        match self.abs_to_object.inverted() {
            Ok(object_to_abs) => gradient.transformed(object_to_abs),
            Err(_) => gradient,
        }
    }

    /// `FindParameter(P)`: the parameter of the point of the curve matching `p`,
    /// in `[0, 2PI)` for a circle or an ellipse (a gp exception at their centre; 0
    /// without a curve). `tool.find_parameter(point)?`.
    pub fn find_parameter(&self, p: Point2) -> Result<f64, Failure> {
        let axis: Axis22d = self.axis;
        let param: f64 = match self.kind {
            CurveType::Line => elclib::line_parameter_2d(axis.x_axis(), p),
            CurveType::Circle => positive_angle(elclib::circle_parameter_2d(axis, p)?),
            CurveType::Ellipse => {
                positive_angle(elclib::ellipse_parameter_2d(axis, self.prm1, self.prm2, p)?)
            }
            CurveType::Parabola => elclib::parabola_parameter_2d(axis, p),
            CurveType::Hyperbola => elclib::hyperbola_parameter_2d(axis, self.prm1, self.prm2, p),
            _ => 0.0,
        };
        Ok(param)
    }
}

// The circle gradient `(x, y) / |(x, y)|` with the Y component scaled by `scale`
// (the ellipse's a/b), null at the centre.
fn radial_gradient(x: f64, y: f64, scale: f64) -> Vector2 {
    let temp1: f64 = (y * y + x * x).sqrt();
    if temp1 == 0.0 {
        return Vector2::new(0.0, 0.0);
    }
    if scale == 1.0 {
        return Vector2::new(x / temp1, y / temp1);
    }
    Vector2::new(x / temp1, (y * scale) / temp1)
}

// An angle below 0 moved by `PI + PI` (the native guard; ElCLib already returns
// `[0, 2PI]`).
pub(crate) fn positive_angle(param: f64) -> f64 {
    if param < 0.0 {
        param + (PI + PI)
    } else {
        param
    }
}

impl ImpTool for IConicTool {
    fn value(&self, x: f64) -> Point2 {
        IConicTool::value(self, x)
    }
    fn d1(&self, u: f64, p: &mut Point2, t: &mut Vector2) {
        IConicTool::d1(self, u, p, t)
    }
    fn d2(&self, u: f64, p: &mut Point2, t: &mut Vector2, n: &mut Vector2) {
        IConicTool::d2(self, u, p, t, n)
    }
    fn distance(&self, p: Point2) -> f64 {
        IConicTool::distance(self, p)
    }
    fn grad_distance(&self, p: Point2) -> Vector2 {
        IConicTool::grad_distance(self, p)
    }
    fn find_parameter(&self, p: Point2) -> Result<f64, Failure> {
        IConicTool::find_parameter(self, p)
    }
}
