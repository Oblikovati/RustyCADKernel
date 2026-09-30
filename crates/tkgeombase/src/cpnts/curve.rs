// Rust translation of the static curve helpers of ModelingData/TKGeomBase/CPnts
// (CPnts_AbscissaPoint.cxx `f3d`, `f2d`, `order`; CPnts_UniformDeflection.cxx `D03d`,
// `D02d`, `D23d`, `D22d`) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The curve that CPnts and GCPnts evaluate: an `Adaptor3d_Curve` or an
//! `Adaptor2d_Curve2d`, the `void* myCurve` plus `bool my3d` of the native classes.
//!
//! The native overloads and the static helpers repeat each evaluation for both curve
//! kinds; [`PntsCurve`] holds the pair once. A 2D point or vector is lifted to 3D with
//! a null Z, as `D02d` and `D22d` do.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// A borrowed 3D or 2D adapted curve; `PntsCurve::from(&adaptor as &dyn Adaptor3dCurve)`.
///
/// Every CPnts and GCPnts member that takes an `Adaptor3d_Curve` or an
/// `Adaptor2d_Curve2d` takes `impl Into<PntsCurve>`, so a `&GeomAdaptorCurve`, a
/// `&Geom2dAdaptorCurve` or a `&dyn` adaptor can be passed directly.
#[derive(Clone, Copy)]
pub enum PntsCurve<'a> {
    /// An `Adaptor3d_Curve`.
    Curve3d(&'a dyn Adaptor3dCurve),
    /// An `Adaptor2d_Curve2d`, evaluated in the plane Z = 0.
    Curve2d(&'a dyn Adaptor2dCurve2d),
}

impl<'a> From<&'a dyn Adaptor3dCurve> for PntsCurve<'a> {
    fn from(curve: &'a dyn Adaptor3dCurve) -> Self {
        Self::Curve3d(curve)
    }
}
impl<'a> From<&'a dyn Adaptor2dCurve2d> for PntsCurve<'a> {
    fn from(curve: &'a dyn Adaptor2dCurve2d) -> Self {
        Self::Curve2d(curve)
    }
}
impl<'a> From<&'a GeomAdaptorCurve> for PntsCurve<'a> {
    fn from(curve: &'a GeomAdaptorCurve) -> Self {
        Self::Curve3d(curve)
    }
}
impl<'a> From<&'a Geom2dAdaptorCurve> for PntsCurve<'a> {
    fn from(curve: &'a Geom2dAdaptorCurve) -> Self {
        Self::Curve2d(curve)
    }
}

// Both adaptor traits declare the same scalar members; forward one to the curve kind.
macro_rules! forward {
    ($self:ident . $member:ident ( $($arg:expr),* )) => {
        match $self {
            PntsCurve::Curve3d(curve) => curve.$member($($arg),*),
            PntsCurve::Curve2d(curve) => curve.$member($($arg),*),
        }
    };
}

fn lift_point(point: Point2) -> Point3 {
    Point3::new(point.x(), point.y(), 0.0)
}

fn lift_vector(vector: Vector2) -> Vector3 {
    Vector3::new(vector.x(), vector.y(), 0.0)
}

impl PntsCurve<'_> {
    /// `FirstParameter()`; `curve.first_parameter()?`.
    pub fn first_parameter(self) -> Result<f64, Failure> {
        forward!(self.first_parameter())
    }
    /// `LastParameter()`; `curve.last_parameter()?`.
    pub fn last_parameter(self) -> Result<f64, Failure> {
        forward!(self.last_parameter())
    }
    /// `NbIntervals(S)`; `curve.nb_intervals(Shape::CN)?`.
    pub fn nb_intervals(self, shape: Shape) -> Result<i32, Failure> {
        forward!(self.nb_intervals(shape))
    }
    /// `Intervals(T, S)`; `curve.intervals(&mut bounds, Shape::CN)?`.
    pub fn intervals(self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        forward!(self.intervals(bounds, shape))
    }
    /// `IsPeriodic()`; `curve.is_periodic()?`.
    pub fn is_periodic(self) -> Result<bool, Failure> {
        forward!(self.is_periodic())
    }
    /// `IsRational()`; `curve.is_rational()?`.
    pub fn is_rational(self) -> Result<bool, Failure> {
        forward!(self.is_rational())
    }
    /// `Resolution(R3d)`; `curve.resolution(1e-7)?`.
    pub fn resolution(self, r3d: f64) -> Result<f64, Failure> {
        forward!(self.resolution(r3d))
    }
    /// `GetType()`; `curve.get_type()?`.
    pub fn get_type(self) -> Result<CurveType, Failure> {
        forward!(self.get_type())
    }
    /// `Circle().Radius()`; `curve.circle_radius()?`.
    pub fn circle_radius(self) -> Result<f64, Failure> {
        match self {
            PntsCurve::Curve3d(curve) => Ok(curve.circle()?.radius()),
            PntsCurve::Curve2d(curve) => Ok(curve.circle()?.radius()),
        }
    }
    /// `DN(U, N).Magnitude()`; `curve.dn_magnitude(0.0, 1)?`.
    pub fn dn_magnitude(self, u: f64, n: i32) -> Result<f64, Failure> {
        match self {
            PntsCurve::Curve3d(curve) => Ok(curve.dn(u, n)?.magnitude()),
            PntsCurve::Curve2d(curve) => Ok(curve.dn(u, n)?.magnitude()),
        }
    }
}

impl PntsCurve<'_> {
    /// The point of parameter `u` (`D03d`, `D02d`); `curve.lifted_d0(0.5)?`.
    pub fn lifted_d0(self, u: f64) -> Result<Point3, Failure> {
        match self {
            PntsCurve::Curve3d(curve) => curve.d0(u),
            PntsCurve::Curve2d(curve) => Ok(lift_point(curve.d0(u)?)),
        }
    }
    /// The point and first derivative of parameter `u` (`D1`, lifted as `D22d` does);
    /// `let (p, v) = curve.lifted_d1(0.5)?;`.
    pub fn lifted_d1(self, u: f64) -> Result<(Point3, Vector3), Failure> {
        match self {
            PntsCurve::Curve3d(curve) => curve.d1(u),
            PntsCurve::Curve2d(curve) => {
                let (point, d1) = curve.d1(u)?;
                Ok((lift_point(point), lift_vector(d1)))
            }
        }
    }
    /// The point and two derivatives of parameter `u` (`D23d`, `D22d`);
    /// `let (p, v1, v2) = curve.lifted_d2(0.5)?;`.
    pub fn lifted_d2(self, u: f64) -> Result<(Point3, Vector3, Vector3), Failure> {
        match self {
            PntsCurve::Curve3d(curve) => curve.d2(u),
            PntsCurve::Curve2d(curve) => {
                let (point, d1, d2) = curve.d2(u)?;
                Ok((lift_point(point), lift_vector(d1), lift_vector(d2)))
            }
        }
    }
    /// The speed `|C'(u)|` integrated for the curve length (`f3d`, `f2d`);
    /// `curve.speed(0.5)?`.
    pub fn speed(self, u: f64) -> Result<f64, Failure> {
        match self {
            PntsCurve::Curve3d(curve) => Ok(curve.d1(u)?.1.magnitude()),
            PntsCurve::Curve2d(curve) => Ok(curve.d1(u)?.1.magnitude()),
        }
    }
    /// The Gauss integration order of the curve length (`order`): 2 for a line, 5 for a
    /// parabola, `min(24, 2 * degree)` for a Bezier curve, `min(24, 2 * poles - 1)` for
    /// a B-spline curve and 10 otherwise; `curve.gauss_order()?`.
    ///
    /// The 3D overload reads `Degree()` and `NbPoles()` from the adaptor, the 2D one
    /// from `Bezier()` and `BSpline()`, as natively.
    pub fn gauss_order(self) -> Result<i32, Failure> {
        Ok(match self.get_type()? {
            CurveType::Line => 2,
            CurveType::Parabola => 5,
            CurveType::BezierCurve => 24.min(2 * self.bezier_degree()?),
            CurveType::BSplineCurve => 24.min(2 * self.bspline_nb_poles()? - 1),
            _ => 10,
        })
    }
    fn bezier_degree(self) -> Result<i32, Failure> {
        match self {
            PntsCurve::Curve3d(curve) => curve.degree(),
            PntsCurve::Curve2d(curve) => Ok(curve.bezier()?.degree()),
        }
    }
    fn bspline_nb_poles(self) -> Result<i32, Failure> {
        match self {
            PntsCurve::Curve3d(curve) => curve.nb_poles(),
            PntsCurve::Curve2d(curve) => Ok(curve.bspline()?.nb_poles()),
        }
    }
}
