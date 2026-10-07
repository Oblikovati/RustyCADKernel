//! HLRBRep_SurfaceTool.hxx/.lxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{BSurfaceTool, Surface};
use crate::hlr_algo::c_int_truncation;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3,
};
use rustycad_tkgeomalgo::int_curve_surface::InterSurfaceTool;
use rustycad_tkgeomalgo::int_imp::ThePSurfaceTool;
use rustycad_tkmath::geom_abs::{Shape, SurfaceType};
use rustycad_tkmath::gp::{
    Axis1, Cone3, Cylinder3, Direction3, Plane3, Point3, Sphere3, Torus3, Vector3,
};
use std::marker::PhantomData;

/// HLRBRep_SurfaceTool: the static access to an HLR [`Surface`] that the
/// line-surface intersection templates (InterCSurf and its helpers) call. The
/// native tool takes `const HLRBRep_Surface*`; the templates hold the surface by
/// value, so the tool names the borrow's lifetime (`SurfaceTool<'a>` reads
/// `&'a Surface`); `SurfaceTool::nb_samples_u(&surface)`.
pub struct SurfaceTool<'a>(PhantomData<&'a Surface>);

impl SurfaceTool<'_> {
    /// FirstUParameter(S); `SurfaceTool::first_u_parameter(&surface)?`.
    pub fn first_u_parameter(s: &Surface) -> Result<f64, Failure> {
        s.first_u_parameter()
    }
    /// FirstVParameter(S); `SurfaceTool::first_v_parameter(&surface)?`.
    pub fn first_v_parameter(s: &Surface) -> Result<f64, Failure> {
        s.first_v_parameter()
    }
    /// LastUParameter(S); `SurfaceTool::last_u_parameter(&surface)?`.
    pub fn last_u_parameter(s: &Surface) -> Result<f64, Failure> {
        s.last_u_parameter()
    }
    /// LastVParameter(S); `SurfaceTool::last_v_parameter(&surface)?`.
    pub fn last_v_parameter(s: &Surface) -> Result<f64, Failure> {
        s.last_v_parameter()
    }
    /// NbUIntervals(S, Sh); `SurfaceTool::nb_u_intervals(&surface, Shape::C2)?`.
    pub fn nb_u_intervals(s: &Surface, sh: Shape) -> Result<i32, Failure> {
        s.nb_u_intervals(sh)
    }
    /// NbVIntervals(S, Sh); `SurfaceTool::nb_v_intervals(&surface, Shape::C2)?`.
    pub fn nb_v_intervals(s: &Surface, sh: Shape) -> Result<i32, Failure> {
        s.nb_v_intervals(sh)
    }
    /// UIntervals(S, T, Sh); `SurfaceTool::u_intervals(&surface, &mut tab, Shape::C2)?`.
    pub fn u_intervals(s: &Surface, tab: &mut Array1<'_, f64>, sh: Shape) -> Result<(), Failure> {
        BSurfaceTool::u_intervals(s.surface(), tab, sh)
    }
    /// VIntervals(S, T, Sh); `SurfaceTool::v_intervals(&surface, &mut tab, Shape::C2)?`.
    pub fn v_intervals(s: &Surface, tab: &mut Array1<'_, f64>, sh: Shape) -> Result<(), Failure> {
        BSurfaceTool::v_intervals(s.surface(), tab, sh)
    }
    /// UTrim(S, First, Last, Tol); `SurfaceTool::u_trim(&surface, 0.0, 1.0, 1e-7)?`.
    pub fn u_trim(
        s: &Surface,
        first: f64,
        last: f64,
        tol: f64,
    ) -> Result<Adaptor3dSurfaceHandle, Failure> {
        BSurfaceTool::u_trim(s.surface(), first, last, tol)
    }
    /// VTrim(S, First, Last, Tol); `SurfaceTool::v_trim(&surface, 0.0, 1.0, 1e-7)?`.
    pub fn v_trim(
        s: &Surface,
        first: f64,
        last: f64,
        tol: f64,
    ) -> Result<Adaptor3dSurfaceHandle, Failure> {
        BSurfaceTool::v_trim(s.surface(), first, last, tol)
    }
    /// IsUClosed(S); `SurfaceTool::is_u_closed(&surface)?`.
    pub fn is_u_closed(s: &Surface) -> Result<bool, Failure> {
        s.is_u_closed()
    }
    /// IsVClosed(S); `SurfaceTool::is_v_closed(&surface)?`.
    pub fn is_v_closed(s: &Surface) -> Result<bool, Failure> {
        s.is_v_closed()
    }
    /// IsUPeriodic(S); `SurfaceTool::is_u_periodic(&surface)?`.
    pub fn is_u_periodic(s: &Surface) -> Result<bool, Failure> {
        s.is_u_periodic()
    }
    /// UPeriod(S); `SurfaceTool::u_period(&surface)?`.
    pub fn u_period(s: &Surface) -> Result<f64, Failure> {
        s.u_period()
    }
    /// IsVPeriodic(S); `SurfaceTool::is_v_periodic(&surface)?`.
    pub fn is_v_periodic(s: &Surface) -> Result<bool, Failure> {
        s.is_v_periodic()
    }
    /// VPeriod(S); `SurfaceTool::v_period(&surface)?`.
    pub fn v_period(s: &Surface) -> Result<f64, Failure> {
        s.v_period()
    }
    /// Value(S, U, V); `SurfaceTool::value(&surface, 0.5, 0.5)?`.
    pub fn value(s: &Surface, u: f64, v: f64) -> Result<Point3, Failure> {
        BSurfaceTool::value(s.surface(), u, v)
    }
    /// D0(S, U, V, P); `SurfaceTool::d0(&surface, 0.5, 0.5)?`.
    pub fn d0(s: &Surface, u: f64, v: f64) -> Result<Point3, Failure> {
        BSurfaceTool::d0(s.surface(), u, v)
    }
    /// D1(S, U, V, P, D1U, D1V); `SurfaceTool::d1(&surface, 0.5, 0.5)?`.
    pub fn d1(s: &Surface, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        BSurfaceTool::d1(s.surface(), u, v)
    }
    /// D2(S, U, V, ...); `SurfaceTool::d2(&surface, 0.5, 0.5)?`.
    pub fn d2(s: &Surface, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        BSurfaceTool::d2(s.surface(), u, v)
    }
    /// D3(S, U, V, ...); `SurfaceTool::d3(&surface, 0.5, 0.5)?`.
    pub fn d3(s: &Surface, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        BSurfaceTool::d3(s.surface(), u, v)
    }
    /// DN(S, U, V, Nu, Nv); `SurfaceTool::dn(&surface, 0.5, 0.5, 1, 1)?`.
    pub fn dn(s: &Surface, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        BSurfaceTool::dn(s.surface(), u, v, nu, nv)
    }
    /// UResolution(S, R3d); `SurfaceTool::u_resolution(&surface, 1e-7)?`.
    pub fn u_resolution(s: &Surface, r3d: f64) -> Result<f64, Failure> {
        BSurfaceTool::u_resolution(s.surface(), r3d)
    }
    /// VResolution(S, R3d); `SurfaceTool::v_resolution(&surface, 1e-7)?`.
    pub fn v_resolution(s: &Surface, r3d: f64) -> Result<f64, Failure> {
        BSurfaceTool::v_resolution(s.surface(), r3d)
    }
    /// GetType(S): the HLR classification; `SurfaceTool::get_type(&surface)`.
    pub fn get_type(s: &Surface) -> SurfaceType {
        s.get_type()
    }
    /// Plane(S); `SurfaceTool::plane(&surface)?`.
    pub fn plane(s: &Surface) -> Result<Plane3, Failure> {
        s.plane()
    }
    /// Cylinder(S); `SurfaceTool::cylinder(&surface)?`.
    pub fn cylinder(s: &Surface) -> Result<Cylinder3, Failure> {
        s.cylinder()
    }
    /// Cone(S); `SurfaceTool::cone(&surface)?`.
    pub fn cone(s: &Surface) -> Result<Cone3, Failure> {
        s.cone()
    }
    /// Torus(S); `SurfaceTool::torus(&surface)?`.
    pub fn torus(s: &Surface) -> Result<Torus3, Failure> {
        s.torus()
    }
    /// Sphere(S); `SurfaceTool::sphere(&surface)?`.
    pub fn sphere(s: &Surface) -> Result<Sphere3, Failure> {
        s.sphere()
    }
    /// Bezier(S); `SurfaceTool::bezier(&surface)?`.
    pub fn bezier(s: &Surface) -> Result<Handle<GeomBezierSurface>, Failure> {
        BSurfaceTool::bezier(s.surface())
    }
    /// BSpline(S); `SurfaceTool::bspline(&surface)?`.
    pub fn bspline(s: &Surface) -> Result<Handle<GeomBSplineSurface>, Failure> {
        BSurfaceTool::bspline(s.surface())
    }
    /// AxeOfRevolution(S); `SurfaceTool::axe_of_revolution(&surface)?`.
    pub fn axe_of_revolution(s: &Surface) -> Result<Axis1, Failure> {
        s.axis()
    }
    /// Direction(S); `SurfaceTool::direction(&surface)?`.
    pub fn direction(s: &Surface) -> Result<Direction3, Failure> {
        BSurfaceTool::direction(s.surface())
    }
    /// BasisCurve(S); `SurfaceTool::basis_curve(&surface)?`.
    pub fn basis_curve(s: &Surface) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        BSurfaceTool::basis_curve(s.surface())
    }
    /// BasisSurface(S); `SurfaceTool::basis_surface(&surface)?`.
    pub fn basis_surface(s: &Surface) -> Result<Adaptor3dSurfaceHandle, Failure> {
        s.surface().basis_surface()
    }
    /// OffsetValue(S); `SurfaceTool::offset_value(&surface)?`.
    pub fn offset_value(s: &Surface) -> Result<f64, Failure> {
        s.surface().offset_value()
    }
    /// NbSamplesU(S): 2 on a plane, `3 + NbUPoles` on a Bezier, `NbUKnots * UDegree`
    /// on a B-spline (at least 2), 20 on a torus, else 10;
    /// `SurfaceTool::nb_samples_u(&surface)?`.
    pub fn nb_samples_u(s: &Surface) -> Result<i32, Failure> {
        Ok(match s.get_type() {
            SurfaceType::Plane => 2,
            SurfaceType::BezierSurface => 3 + s.nb_u_poles()?,
            SurfaceType::BSplineSurface => (s.nb_u_knots()? * s.u_degree()?).max(2),
            SurfaceType::Torus => 20,
            _ => 10,
        })
    }
    /// NbSamplesV(S): 2 on a plane, `3 + NbVPoles` on a Bezier, `NbVKnots * VDegree`
    /// on a B-spline (at least 2), 15 on the elementary and swept surfaces, else 10;
    /// `SurfaceTool::nb_samples_v(&surface)?`.
    pub fn nb_samples_v(s: &Surface) -> Result<i32, Failure> {
        Ok(match s.get_type() {
            SurfaceType::Plane => 2,
            SurfaceType::BezierSurface => 3 + s.nb_v_poles()?,
            SurfaceType::BSplineSurface => (s.nb_v_knots()? * s.v_degree()?).max(2),
            SurfaceType::Cylinder
            | SurfaceType::Cone
            | SurfaceType::Sphere
            | SurfaceType::Torus
            | SurfaceType::SurfaceOfRevolution
            | SurfaceType::SurfaceOfExtrusion => 15,
            _ => 10,
        })
    }
    /// NbSamplesU(S, U1, U2): above 10 samples, scaled by `(int)((U2 - U1) /
    /// (First - Last))` (the native ratio has the reversed span) and kept in
    /// `[5, NbSamplesU(S)]`; `SurfaceTool::nb_samples_u_in(&surface, 0.0, 1.0)?`.
    pub fn nb_samples_u_in(s: &Surface, u1: f64, u2: f64) -> Result<i32, Failure> {
        let nbs = Self::nb_samples_u(s)?;
        if nbs <= 10 {
            return Ok(nbs);
        }
        let (uf, ul) = (s.first_u_parameter()?, s.last_u_parameter()?);
        Ok(scaled_samples(nbs, (u2 - u1) / (uf - ul)))
    }
    /// NbSamplesV(S, V1, V2): as [`SurfaceTool::nb_samples_u_in`] in V;
    /// `SurfaceTool::nb_samples_v_in(&surface, 0.0, 1.0)?`.
    pub fn nb_samples_v_in(s: &Surface, v1: f64, v2: f64) -> Result<i32, Failure> {
        let nbs = Self::nb_samples_v(s)?;
        if nbs <= 10 {
            return Ok(nbs);
        }
        let (vf, vl) = (s.first_v_parameter()?, s.last_v_parameter()?);
        Ok(scaled_samples(nbs, (v2 - v1) / (vf - vl)))
    }
}

// `n *= (int)ratio; if (n > nbs) n = nbs; if (n < 5) n = 5;` with the x86 truncation
// and wrapping integer product of the native code.
fn scaled_samples(nbs: i32, ratio: f64) -> i32 {
    let n = nbs.wrapping_mul(c_int_truncation(ratio));
    n.min(nbs).max(5)
}

impl<'a> ThePSurfaceTool for SurfaceTool<'a> {
    type Surface = &'a Surface;
    fn first_u_parameter(surface: &&'a Surface) -> Result<f64, Failure> {
        surface.first_u_parameter()
    }
    fn first_v_parameter(surface: &&'a Surface) -> Result<f64, Failure> {
        surface.first_v_parameter()
    }
    fn last_u_parameter(surface: &&'a Surface) -> Result<f64, Failure> {
        surface.last_u_parameter()
    }
    fn last_v_parameter(surface: &&'a Surface) -> Result<f64, Failure> {
        surface.last_v_parameter()
    }
    fn u_resolution(surface: &&'a Surface, r3d: f64) -> Result<f64, Failure> {
        SurfaceTool::u_resolution(surface, r3d)
    }
    fn v_resolution(surface: &&'a Surface, r3d: f64) -> Result<f64, Failure> {
        SurfaceTool::v_resolution(surface, r3d)
    }
    fn value(surface: &&'a Surface, u: f64, v: f64) -> Result<Point3, Failure> {
        SurfaceTool::value(surface, u, v)
    }
    fn d1(surface: &&'a Surface, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        SurfaceTool::d1(surface, u, v)
    }
}

impl<'a> InterSurfaceTool for SurfaceTool<'a> {
    fn get_type(surface: &&'a Surface) -> Result<SurfaceType, Failure> {
        Ok(surface.get_type())
    }
    fn plane(surface: &&'a Surface) -> Result<Plane3, Failure> {
        surface.plane()
    }
    fn cylinder(surface: &&'a Surface) -> Result<Cylinder3, Failure> {
        surface.cylinder()
    }
    fn cone(surface: &&'a Surface) -> Result<Cone3, Failure> {
        surface.cone()
    }
    fn sphere(surface: &&'a Surface) -> Result<Sphere3, Failure> {
        surface.sphere()
    }
    fn torus(surface: &&'a Surface) -> Result<Torus3, Failure> {
        surface.torus()
    }
    fn nb_samples_u(surface: &&'a Surface, u1: f64, u2: f64) -> Result<i32, Failure> {
        SurfaceTool::nb_samples_u_in(surface, u1, u2)
    }
    fn nb_samples_v(surface: &&'a Surface, v1: f64, v2: f64) -> Result<i32, Failure> {
        SurfaceTool::nb_samples_v_in(surface, v1, v2)
    }
    fn nb_u_intervals(surface: &&'a Surface, shape: Shape) -> Result<i32, Failure> {
        surface.nb_u_intervals(shape)
    }
    fn nb_v_intervals(surface: &&'a Surface, shape: Shape) -> Result<i32, Failure> {
        surface.nb_v_intervals(shape)
    }
    fn u_intervals(
        surface: &&'a Surface,
        bounds: &mut Array1<'_, f64>,
        shape: Shape,
    ) -> Result<(), Failure> {
        SurfaceTool::u_intervals(surface, bounds, shape)
    }
    fn v_intervals(
        surface: &&'a Surface,
        bounds: &mut Array1<'_, f64>,
        shape: Shape,
    ) -> Result<(), Failure> {
        SurfaceTool::v_intervals(surface, bounds, shape)
    }
    fn u_trim(
        surface: &&'a Surface,
        first: f64,
        last: f64,
        tol: f64,
    ) -> Result<Adaptor3dSurfaceHandle, Failure> {
        SurfaceTool::u_trim(surface, first, last, tol)
    }
    fn d0(surface: &&'a Surface, u: f64, v: f64) -> Result<Point3, Failure> {
        SurfaceTool::d0(surface, u, v)
    }
    fn is_u_closed(surface: &&'a Surface) -> Result<bool, Failure> {
        surface.is_u_closed()
    }
    fn is_v_closed(surface: &&'a Surface) -> Result<bool, Failure> {
        surface.is_v_closed()
    }
    fn is_u_periodic(surface: &&'a Surface) -> Result<bool, Failure> {
        surface.is_u_periodic()
    }
    fn is_v_periodic(surface: &&'a Surface) -> Result<bool, Failure> {
        surface.is_v_periodic()
    }
    fn u_period(surface: &&'a Surface) -> Result<f64, Failure> {
        surface.u_period()
    }
    fn v_period(surface: &&'a Surface) -> Result<f64, Failure> {
        surface.v_period()
    }
    fn basis_surface(surface: &&'a Surface) -> Result<Adaptor3dSurfaceHandle, Failure> {
        SurfaceTool::basis_surface(surface)
    }
    fn offset_value(surface: &&'a Surface) -> Result<f64, Failure> {
        SurfaceTool::offset_value(surface)
    }
    fn direction(surface: &&'a Surface) -> Result<Direction3, Failure> {
        SurfaceTool::direction(surface)
    }
    fn basis_curve(surface: &&'a Surface) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        SurfaceTool::basis_curve(surface)
    }
    fn axe_of_revolution(surface: &&'a Surface) -> Result<Axis1, Failure> {
        surface.axis()
    }
}
