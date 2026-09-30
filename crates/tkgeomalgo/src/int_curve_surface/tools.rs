// Rust translation of the CurveTool and SurfaceTool parameters of
// ModelingAlgorithms/TKGeomAlgo/IntCurveSurface/IntCurveSurface_*.pxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The type parameters of the IntCurveSurface templates: the calls the `.pxx`
//! bodies make on `CurveTool` and `SurfaceTool`, under their native names
//! (docs/geomalgo-design.md section 11). They extend the IntImp tool traits, which
//! hold the calls of the CS function and of the exact intersector.
use crate::int_imp::{HSurfaceTool, TheCurve3dTool, ThePSurfaceTool};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::adaptor3d::h_surface_tool::{self, Adaptor3dSurfaceHandle};
use rustycad_tkmath::geom_abs::{CurveType, Shape, SurfaceType};
use rustycad_tkmath::gp::{
    Axis1, Circle3, Cone3, Cylinder3, Direction3, Ellipse3, Hyperbola3, Line3, Parabola3, Plane3,
    Point3, Sphere3, Torus3,
};

/// `CurveTool` of the IntCurveSurface templates (IntCurveSurface_TheHCurveTool,
/// HLRBRep_LineTool); `C::get_type(&curve)?` classifies a curve of a tool `C`.
pub trait InterCurveTool: TheCurve3dTool {
    /// `GetType(C)`.
    fn get_type(curve: &Self::Curve) -> Result<CurveType, Failure>;
    /// `Line(C)`.
    fn line(curve: &Self::Curve) -> Result<Line3, Failure>;
    /// `Circle(C)`.
    fn circle(curve: &Self::Curve) -> Result<Circle3, Failure>;
    /// `Ellipse(C)`.
    fn ellipse(curve: &Self::Curve) -> Result<Ellipse3, Failure>;
    /// `Parabola(C)`.
    fn parabola(curve: &Self::Curve) -> Result<Parabola3, Failure>;
    /// `Hyperbola(C)`.
    fn hyperbola(curve: &Self::Curve) -> Result<Hyperbola3, Failure>;
    /// `NbIntervals(C, S)`.
    fn nb_intervals(curve: &Self::Curve, shape: Shape) -> Result<i32, Failure>;
    /// `Intervals(C, T, S)`.
    fn intervals(
        curve: &Self::Curve,
        bounds: &mut Array1<'_, f64>,
        shape: Shape,
    ) -> Result<(), Failure>;
    /// `IsPeriodic(C)`.
    fn is_periodic(curve: &Self::Curve) -> Result<bool, Failure>;
    /// `Period(C)`.
    fn period(curve: &Self::Curve) -> Result<f64, Failure>;
    /// `D0(C, U, P)`.
    fn d0(curve: &Self::Curve, u: f64) -> Result<Point3, Failure>;
    /// `NbSamples(C, U0, U1)`.
    fn nb_samples(curve: &Self::Curve, u0: f64, u1: f64) -> Result<i32, Failure>;
    /// `SamplePars(C, U0, U1, Defl, NbMin)`: the sample parameters, from index 1.
    fn sample_pars(
        curve: &Self::Curve,
        u0: f64,
        u1: f64,
        defl: f64,
        nb_min: i32,
    ) -> Result<Array1<'static, f64>, Failure>;
}

/// `SurfaceTool` of the IntCurveSurface templates (Adaptor3d_HSurfaceTool,
/// HLRBRep_SurfaceTool); `S::get_type(&surface)?` classifies a surface of a tool `S`.
pub trait InterSurfaceTool: ThePSurfaceTool {
    /// `GetType(S)`.
    fn get_type(surface: &Self::Surface) -> Result<SurfaceType, Failure>;
    /// `Plane(S)`.
    fn plane(surface: &Self::Surface) -> Result<Plane3, Failure>;
    /// `Cylinder(S)`.
    fn cylinder(surface: &Self::Surface) -> Result<Cylinder3, Failure>;
    /// `Cone(S)`.
    fn cone(surface: &Self::Surface) -> Result<Cone3, Failure>;
    /// `Sphere(S)`.
    fn sphere(surface: &Self::Surface) -> Result<Sphere3, Failure>;
    /// `Torus(S)`.
    fn torus(surface: &Self::Surface) -> Result<Torus3, Failure>;
    /// `NbSamplesU(S, u1, u2)`.
    fn nb_samples_u(surface: &Self::Surface, u1: f64, u2: f64) -> Result<i32, Failure>;
    /// `NbSamplesV(S, v1, v2)`.
    fn nb_samples_v(surface: &Self::Surface, v1: f64, v2: f64) -> Result<i32, Failure>;
    /// `NbUIntervals(S, Sh)`.
    fn nb_u_intervals(surface: &Self::Surface, shape: Shape) -> Result<i32, Failure>;
    /// `NbVIntervals(S, Sh)`.
    fn nb_v_intervals(surface: &Self::Surface, shape: Shape) -> Result<i32, Failure>;
    /// `UIntervals(S, T, Sh)`.
    fn u_intervals(
        surface: &Self::Surface,
        bounds: &mut Array1<'_, f64>,
        shape: Shape,
    ) -> Result<(), Failure>;
    /// `VIntervals(S, T, Sh)`.
    fn v_intervals(
        surface: &Self::Surface,
        bounds: &mut Array1<'_, f64>,
        shape: Shape,
    ) -> Result<(), Failure>;
    /// `UTrim(S, First, Last, Tol)`: an Adaptor3d handle for every instance.
    fn u_trim(
        surface: &Self::Surface,
        first: f64,
        last: f64,
        tol: f64,
    ) -> Result<Adaptor3dSurfaceHandle, Failure>;
    /// `D0(S, U, V, P)`.
    fn d0(surface: &Self::Surface, u: f64, v: f64) -> Result<Point3, Failure>;
    /// `IsUClosed(S)`.
    fn is_u_closed(surface: &Self::Surface) -> Result<bool, Failure>;
    /// `IsVClosed(S)`.
    fn is_v_closed(surface: &Self::Surface) -> Result<bool, Failure>;
    /// `IsUPeriodic(S)`.
    fn is_u_periodic(surface: &Self::Surface) -> Result<bool, Failure>;
    /// `IsVPeriodic(S)`.
    fn is_v_periodic(surface: &Self::Surface) -> Result<bool, Failure>;
    /// `UPeriod(S)`.
    fn u_period(surface: &Self::Surface) -> Result<f64, Failure>;
    /// `VPeriod(S)`.
    fn v_period(surface: &Self::Surface) -> Result<f64, Failure>;
    /// `BasisSurface(S)`: an Adaptor3d handle for every instance.
    fn basis_surface(surface: &Self::Surface) -> Result<Adaptor3dSurfaceHandle, Failure>;
    /// `OffsetValue(S)`.
    fn offset_value(surface: &Self::Surface) -> Result<f64, Failure>;
    /// `Direction(S)`: the extrusion direction.
    fn direction(surface: &Self::Surface) -> Result<Direction3, Failure>;
    /// `BasisCurve(S)`: an Adaptor3d handle for every instance.
    fn basis_curve(surface: &Self::Surface) -> Result<Handle<dyn Adaptor3dCurve>, Failure>;
    /// `AxeOfRevolution(S)`.
    fn axe_of_revolution(surface: &Self::Surface) -> Result<Axis1, Failure>;
}

impl InterSurfaceTool for HSurfaceTool {
    fn get_type(surface: &Self::Surface) -> Result<SurfaceType, Failure> {
        h_surface_tool::get_type(surface)
    }
    fn plane(surface: &Self::Surface) -> Result<Plane3, Failure> {
        h_surface_tool::plane(surface)
    }
    fn cylinder(surface: &Self::Surface) -> Result<Cylinder3, Failure> {
        h_surface_tool::cylinder(surface)
    }
    fn cone(surface: &Self::Surface) -> Result<Cone3, Failure> {
        h_surface_tool::cone(surface)
    }
    fn sphere(surface: &Self::Surface) -> Result<Sphere3, Failure> {
        h_surface_tool::sphere(surface)
    }
    fn torus(surface: &Self::Surface) -> Result<Torus3, Failure> {
        h_surface_tool::torus(surface)
    }
    fn nb_samples_u(surface: &Self::Surface, u1: f64, u2: f64) -> Result<i32, Failure> {
        h_surface_tool::nb_samples_u_range(surface, u1, u2)
    }
    fn nb_samples_v(surface: &Self::Surface, v1: f64, v2: f64) -> Result<i32, Failure> {
        h_surface_tool::nb_samples_v_range(surface, v1, v2)
    }
    fn nb_u_intervals(surface: &Self::Surface, shape: Shape) -> Result<i32, Failure> {
        h_surface_tool::nb_u_intervals(surface, shape)
    }
    fn nb_v_intervals(surface: &Self::Surface, shape: Shape) -> Result<i32, Failure> {
        h_surface_tool::nb_v_intervals(surface, shape)
    }
    fn u_intervals(
        surface: &Self::Surface,
        bounds: &mut Array1<'_, f64>,
        shape: Shape,
    ) -> Result<(), Failure> {
        h_surface_tool::u_intervals(surface, bounds, shape)
    }
    fn v_intervals(
        surface: &Self::Surface,
        bounds: &mut Array1<'_, f64>,
        shape: Shape,
    ) -> Result<(), Failure> {
        h_surface_tool::v_intervals(surface, bounds, shape)
    }
    fn u_trim(
        surface: &Self::Surface,
        first: f64,
        last: f64,
        tol: f64,
    ) -> Result<Adaptor3dSurfaceHandle, Failure> {
        h_surface_tool::u_trim(surface, first, last, tol)
    }
    fn d0(surface: &Self::Surface, u: f64, v: f64) -> Result<Point3, Failure> {
        h_surface_tool::d0(surface, u, v)
    }
    fn is_u_closed(surface: &Self::Surface) -> Result<bool, Failure> {
        h_surface_tool::is_u_closed(surface)
    }
    fn is_v_closed(surface: &Self::Surface) -> Result<bool, Failure> {
        h_surface_tool::is_v_closed(surface)
    }
    fn is_u_periodic(surface: &Self::Surface) -> Result<bool, Failure> {
        h_surface_tool::is_u_periodic(surface)
    }
    fn is_v_periodic(surface: &Self::Surface) -> Result<bool, Failure> {
        h_surface_tool::is_v_periodic(surface)
    }
    fn u_period(surface: &Self::Surface) -> Result<f64, Failure> {
        h_surface_tool::u_period(surface)
    }
    fn v_period(surface: &Self::Surface) -> Result<f64, Failure> {
        h_surface_tool::v_period(surface)
    }
    fn basis_surface(surface: &Self::Surface) -> Result<Adaptor3dSurfaceHandle, Failure> {
        h_surface_tool::basis_surface(surface)
    }
    fn offset_value(surface: &Self::Surface) -> Result<f64, Failure> {
        h_surface_tool::offset_value(surface)
    }
    fn direction(surface: &Self::Surface) -> Result<Direction3, Failure> {
        h_surface_tool::direction(surface)
    }
    fn basis_curve(surface: &Self::Surface) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        h_surface_tool::basis_curve(surface)
    }
    fn axe_of_revolution(surface: &Self::Surface) -> Result<Axis1, Failure> {
        h_surface_tool::axe_of_revolution(surface)
    }
}
