// Rust translation of IntCurve_PConic.hxx/.lxx/.cxx and IntCurve_PConicTool.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurve_PConic` and `IntCurve_PConicTool`: a gp conic as the parametric curve of
//! IntImpParGen.
use crate::int_imp_par_gen::ParTool;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{
    Axis22d, Circle2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2, Vector2,
};

/// `IntCurve_PConic`: a conic by its frame and two parameters (radii or focal
/// length), with the root search tolerance `EpsX` (1e-8) and the number of samples
/// `Accuracy` (20, 50 for a hyperbola).
///
/// ```
/// use rustycad_tkgeomalgo::int_curve::PConic;
/// use rustycad_tkmath::gp::{Axis22d, Circle2};
/// let pconic = PConic::from_circle(Circle2::new(Axis22d::default(), 2.0).unwrap());
/// assert_eq!((pconic.param1(), pconic.accuracy()), (2.0, 20));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PConic {
    axe: Axis22d,
    prm1: f64,
    prm2: f64,
    the_eps_x: f64,
    the_accuracy: i32,
    kind: CurveType,
}

impl PConic {
    fn of(kind: CurveType, axe: Axis22d, (prm1, prm2): (f64, f64), accuracy: i32) -> Self {
        Self {
            axe,
            prm1,
            prm2,
            the_eps_x: 0.00000001,
            the_accuracy: accuracy,
            kind,
        }
    }
    /// `IntCurve_PConic(gp_Elips2d)`. `PConic::from_ellipse(ellipse)`.
    pub fn from_ellipse(e: Ellipse2) -> Self {
        Self::of(
            CurveType::Ellipse,
            e.axis(),
            (e.major_radius(), e.minor_radius()),
            20,
        )
    }
    /// `IntCurve_PConic(gp_Circ2d)`. `PConic::from_circle(circle)`.
    pub fn from_circle(c: Circle2) -> Self {
        Self::of(CurveType::Circle, c.axis(), (c.radius(), 0.0), 20)
    }
    /// `IntCurve_PConic(gp_Parab2d)`. `PConic::from_parabola(parabola)`.
    pub fn from_parabola(p: Parabola2) -> Self {
        Self::of(CurveType::Parabola, p.axis(), (p.focal(), 0.0), 20)
    }
    /// `IntCurve_PConic(gp_Hypr2d)`: 50 samples. `PConic::from_hyperbola(hyperbola)`.
    pub fn from_hyperbola(h: Hyperbola2) -> Self {
        Self::of(
            CurveType::Hyperbola,
            h.axis(),
            (h.major_radius(), h.minor_radius()),
            50,
        )
    }
    /// `IntCurve_PConic(gp_Lin2d)`: the frame built from the line's axis (a gp error
    /// that a unit direction cannot raise is returned). `PConic::from_line(line)?`.
    pub fn from_line(l: Line2) -> Result<Self, Failure> {
        let axe: Axis22d = Axis22d::from_axis(l.position(), true)?;
        Ok(Self::of(CurveType::Line, axe, (0.0, 0.0), 20))
    }
    /// `SetEpsX(EpsDist)`: the parametric tolerance of the root search.
    /// `pconic.set_eps_x(1e-10)`.
    pub fn set_eps_x(&mut self, epsx: f64) {
        self.the_eps_x = epsx;
    }
    /// `SetAccuracy(Nb)`: the number of samples of the root search.
    /// `pconic.set_accuracy(40)`.
    pub fn set_accuracy(&mut self, n: i32) {
        self.the_accuracy = n;
    }
    /// `Accuracy()`. `pconic.accuracy()`.
    pub fn accuracy(&self) -> i32 {
        self.the_accuracy
    }
    /// `EpsX()`. `pconic.eps_x()`.
    pub fn eps_x(&self) -> f64 {
        self.the_eps_x
    }
    /// `TypeCurve()`: the kind of conic. `pconic.type_curve()`.
    pub fn type_curve(&self) -> CurveType {
        self.kind
    }
    /// `Axis2()`: the frame of the conic. `pconic.axis2()`.
    pub fn axis2(&self) -> Axis22d {
        self.axe
    }
    /// `Param1()`: the radius, the major radius or the focal length.
    /// `pconic.param1()`.
    pub fn param1(&self) -> f64 {
        self.prm1
    }
    /// `Param2()`: the minor radius (0 otherwise). `pconic.param2()`.
    pub fn param2(&self) -> f64 {
        self.prm2
    }
}

/// `IntCurve_PConicTool`: the static evaluation of a [`PConic`] (the `ParTool` of
/// IntImpParGen); `PConicTool::value(&pconic, 0.5)`.
pub struct PConicTool;

impl PConicTool {
    /// `EpsX(C)`. `PConicTool::eps_x(&pconic)`.
    pub fn eps_x(c: &PConic) -> f64 {
        c.eps_x()
    }
    /// `NbSamples(C)`: the accuracy of the conic. `PConicTool::nb_samples(&pconic)`.
    pub fn nb_samples(c: &PConic) -> i32 {
        c.accuracy()
    }
    /// `NbSamples(C, U0, U1)`: the accuracy, whatever the range.
    /// `PConicTool::nb_samples_in(&pconic, 0.0, 1.0)`.
    pub fn nb_samples_in(c: &PConic, _u0: f64, _u1: f64) -> i32 {
        c.accuracy()
    }
    /// `Value(C, X)`: any kind other than a line, circle, ellipse or parabola is
    /// evaluated as a hyperbola. `PConicTool::value(&pconic, 0.5)`.
    pub fn value(c: &PConic, x: f64) -> Point2 {
        let axe: Axis22d = c.axis2();
        match c.type_curve() {
            CurveType::Line => elclib::line_value_2d(x, axe.x_axis()),
            CurveType::Circle => elclib::circle_value_2d(x, axe, c.param1()),
            CurveType::Ellipse => elclib::ellipse_value_2d(x, axe, c.param1(), c.param2()),
            CurveType::Parabola => elclib::parabola_value_2d(x, axe, c.param1()),
            _ => elclib::hyperbola_value_2d(x, axe, c.param1(), c.param2()),
        }
    }
    /// `D1(C, U, P, T)`: the point and tangent. `PConicTool::d1(&pconic, 0.5, &mut p, &mut t)`.
    pub fn d1(c: &PConic, u: f64, pt: &mut Point2, tan: &mut Vector2) {
        let axe: Axis22d = c.axis2();
        let (p, t) = match c.type_curve() {
            CurveType::Line => elclib::line_d1_2d(u, axe.x_axis()),
            CurveType::Circle => elclib::circle_d1_2d(u, axe, c.param1()),
            CurveType::Ellipse => elclib::ellipse_d1_2d(u, axe, c.param1(), c.param2()),
            CurveType::Parabola => elclib::parabola_d1_2d(u, axe, c.param1()),
            CurveType::Hyperbola => elclib::hyperbola_d1_2d(u, axe, c.param1(), c.param2()),
            _ => return,
        };
        (*pt, *tan) = (p, t);
    }
    /// `D2(C, U, P, T, N)`: the point and the first two derivatives; for a line the
    /// second derivative is not written (it keeps the caller's value), as natively.
    /// `PConicTool::d2(&pconic, 0.5, &mut p, &mut t, &mut n)`.
    pub fn d2(c: &PConic, u: f64, pt: &mut Point2, tan: &mut Vector2, norm: &mut Vector2) {
        let axe: Axis22d = c.axis2();
        let (p, t, n) = match c.type_curve() {
            CurveType::Line => {
                (*pt, *tan) = elclib::line_d1_2d(u, axe.x_axis());
                return;
            }
            CurveType::Circle => elclib::circle_d2_2d(u, axe, c.param1()),
            CurveType::Ellipse => elclib::ellipse_d2_2d(u, axe, c.param1(), c.param2()),
            CurveType::Parabola => elclib::parabola_d2_2d(u, axe, c.param1()),
            CurveType::Hyperbola => elclib::hyperbola_d2_2d(u, axe, c.param1(), c.param2()),
            _ => return,
        };
        (*pt, *tan, *norm) = (p, t, n);
    }
}

impl ParTool for PConicTool {
    type Curve = PConic;
    fn eps_x(curve: &PConic) -> f64 {
        PConicTool::eps_x(curve)
    }
    fn nb_samples(curve: &PConic, u0: f64, u1: f64) -> Result<i32, Failure> {
        Ok(PConicTool::nb_samples_in(curve, u0, u1))
    }
    fn value(curve: &PConic, x: f64) -> Result<Point2, Failure> {
        Ok(PConicTool::value(curve, x))
    }
    fn d1(curve: &PConic, u: f64, p: &mut Point2, t: &mut Vector2) -> Result<(), Failure> {
        PConicTool::d1(curve, u, p, t);
        Ok(())
    }
    fn d2(
        curve: &PConic,
        u: f64,
        p: &mut Point2,
        t: &mut Vector2,
        n: &mut Vector2,
    ) -> Result<(), Failure> {
        PConicTool::d2(curve, u, p, t, n);
        Ok(())
    }
}
