// Rust translation of ModelingData/TKGeomBase/GCPnts/GCPnts_TCurveTypes.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GCPnts_TCurveTypes`: the types that the GCPnts templates take from their curve
//! parameter, `Adaptor3d_Curve` or `Adaptor2d_Curve2d`.
use crate::cpnts::PntsCurve;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBezierCurve, GeomCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::gp::{Direction2, Direction3, Line2, Line3, Point2, Point3};

/// The members GCPnts reads from `Geom_BezierCurve`, `Geom_BSplineCurve` and their
/// 2D twins (`TCurveTypes::BezierCurve`, `TCurveTypes::BSplineCurve`);
/// `bezier.pole_count()`.
pub trait PoleCurve {
    /// `NbPoles()`.
    fn pole_count(&self) -> i32;
    /// `IsRational()`.
    fn rational(&self) -> bool;
    /// `Degree()`.
    fn pole_degree(&self) -> i32;
    /// `FirstParameter()`.
    fn start_parameter(&self) -> f64;
    /// `DN(U, N).Magnitude()`.
    fn derivative_magnitude(&self, u: f64, n: i32) -> Result<f64, Failure>;
}

/// The object of a handle an adaptor returned; Standard_NullObject for a null handle,
/// which the native code would dereference; `live(&curve.bezier_curve()?)?`.
pub(crate) fn live<T: ?Sized + RuntimeTyped>(handle: &Handle<T>) -> Result<&T, Failure> {
    handle.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "GCPnts: the adaptor returned a null curve handle, expected a Bezier or B-spline curve",
            "",
        )
    })
}

macro_rules! pole_curve {
    ($($curve:ty => $geom:ident),* $(,)?) => {$(
        impl PoleCurve for $curve {
            fn pole_count(&self) -> i32 {
                self.nb_poles()
            }
            fn rational(&self) -> bool {
                self.is_rational()
            }
            fn pole_degree(&self) -> i32 {
                self.degree()
            }
            fn start_parameter(&self) -> f64 {
                $geom::first_parameter(self)
            }
            fn derivative_magnitude(&self, u: f64, n: i32) -> Result<f64, Failure> {
                Ok($geom::dn(self, u, n)?.magnitude())
            }
        }
    )*};
}
pole_curve!(
    GeomBezierCurve => GeomCurve,
    GeomBSplineCurve => GeomCurve,
    Geom2dBezierCurve => Geom2dCurve,
    Geom2dBSplineCurve => Geom2dCurve,
);

/// `GCPnts_TCurveTypes<TheCurve>`: the point, line and pole-curve types of a curve
/// kind, with the members GCPnts evaluates on them; implemented for
/// `dyn Adaptor3dCurve`, `dyn Adaptor2dCurve2d`, `GeomAdaptorCurve` and
/// `Geom2dAdaptorCurve`; `curve.pnts_curve().first_parameter()?`.
///
/// The native `DistFunction` and `DistFunctionMV` typedefs are the generic
/// [`super::TDistFunction`] and [`super::TDistFunctionMV`] over the curve type.
pub trait TCurveTypes {
    /// `Point`: `gp_Pnt` or `gp_Pnt2d`.
    type Point: Copy;
    /// The chord line of `DistFunction`: `gp_Lin` or `gp_Lin2d`.
    type Line: Clone;
    /// `BezierCurve`: `Geom_BezierCurve` or `Geom2d_BezierCurve`.
    type BezierCurve: PoleCurve + RuntimeTyped;
    /// `BSplineCurve`: `Geom_BSplineCurve` or `Geom2d_BSplineCurve`.
    type BSplineCurve: PoleCurve + RuntimeTyped;
    /// The curve as CPnts takes it; `curve.pnts_curve()`.
    fn pnts_curve(&self) -> PntsCurve<'_>;
    /// `Value(U)`; `curve.point_at(0.5)?`.
    fn point_at(&self, u: f64) -> Result<Self::Point, Failure>;
    /// `Bezier()`; `curve.bezier_curve()?`.
    fn bezier_curve(&self) -> Result<Handle<Self::BezierCurve>, Failure>;
    /// `BSpline()`; `curve.bspline_curve()?`.
    fn bspline_curve(&self) -> Result<Handle<Self::BSplineCurve>, Failure>;
    /// `P1.Distance(P2)`.
    fn point_distance(p1: Self::Point, p2: Self::Point) -> f64;
    /// `P1.SquareDistance(P2)`.
    fn point_square_distance(p1: Self::Point, p2: Self::Point) -> f64;
    /// `gp_Lin(P1, P2.XYZ() - P1.XYZ())`; Standard_ConstructionError for equal points.
    fn chord_line(p1: Self::Point, p2: Self::Point) -> Result<Self::Line, Failure>;
    /// `Lin.SquareDistance(P)`.
    fn line_square_distance(line: &Self::Line, point: Self::Point) -> f64;
}

macro_rules! t_curve_types_3d {
    ($($curve:ty),*) => {$(
        impl TCurveTypes for $curve {
            type Point = Point3;
            type Line = Line3;
            type BezierCurve = GeomBezierCurve;
            type BSplineCurve = GeomBSplineCurve;
            fn pnts_curve(&self) -> PntsCurve<'_> {
                PntsCurve::Curve3d(self)
            }
            fn point_at(&self, u: f64) -> Result<Point3, Failure> {
                Adaptor3dCurve::value(self, u)
            }
            fn bezier_curve(&self) -> Result<Handle<GeomBezierCurve>, Failure> {
                Adaptor3dCurve::bezier(self)
            }
            fn bspline_curve(&self) -> Result<Handle<GeomBSplineCurve>, Failure> {
                Adaptor3dCurve::bspline(self)
            }
            fn point_distance(p1: Point3, p2: Point3) -> f64 {
                p1.distance(&p2)
            }
            fn point_square_distance(p1: Point3, p2: Point3) -> f64 {
                p1.square_distance(&p2)
            }
            fn chord_line(p1: Point3, p2: Point3) -> Result<Line3, Failure> {
                Ok(Line3::new(p1, Direction3::from_xyz(p2.xyz() - p1.xyz())?))
            }
            fn line_square_distance(line: &Line3, point: Point3) -> f64 {
                line.square_distance(point)
            }
        }
    )*};
}
t_curve_types_3d!(dyn Adaptor3dCurve, GeomAdaptorCurve);

macro_rules! t_curve_types_2d {
    ($($curve:ty),*) => {$(
        impl TCurveTypes for $curve {
            type Point = Point2;
            type Line = Line2;
            type BezierCurve = Geom2dBezierCurve;
            type BSplineCurve = Geom2dBSplineCurve;
            fn pnts_curve(&self) -> PntsCurve<'_> {
                PntsCurve::Curve2d(self)
            }
            fn point_at(&self, u: f64) -> Result<Point2, Failure> {
                Adaptor2dCurve2d::value(self, u)
            }
            fn bezier_curve(&self) -> Result<Handle<Geom2dBezierCurve>, Failure> {
                Adaptor2dCurve2d::bezier(self)
            }
            fn bspline_curve(&self) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
                Adaptor2dCurve2d::bspline(self)
            }
            fn point_distance(p1: Point2, p2: Point2) -> f64 {
                p1.distance(p2)
            }
            fn point_square_distance(p1: Point2, p2: Point2) -> f64 {
                p1.square_distance(p2)
            }
            fn chord_line(p1: Point2, p2: Point2) -> Result<Line2, Failure> {
                Ok(Line2::new(p1, Direction2::from_xy(p2.xy() - p1.xy())?))
            }
            fn line_square_distance(line: &Line2, point: Point2) -> f64 {
                line.square_distance(point)
            }
        }
    )*};
}
t_curve_types_2d!(dyn Adaptor2dCurve2d, Geom2dAdaptorCurve);
