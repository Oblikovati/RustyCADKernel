//! HLRBRep_SLPropsATool.hxx/.lxx and HLRBRep_SLProps.hxx, OCCT
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::Surface;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkg3d::geom::{GeomSurfaceD1, GeomSurfaceD2};
use rustycad_tkgeombase::geom_lprop::SLPropsBase;
use rustycad_tkgeombase::geom_lprop::surface_access::{PropsBounds, SurfaceTool, ToolAccess};
use rustycad_tkmath::gp::{Point3, Vector3};

/// HLRBRep_SLPropsATool: the static evaluation of an HLR [`Surface`] for the local
/// properties; the bounds are infinite (`RealFirst`, `RealLast`) and the continuity
/// is always 2; `SLPropsATool::value(&surface, 0.5, 0.5)?`.
pub struct SLPropsATool;

/// HLRBRep_SLProps: `GeomLProp_SLPropsBase` on an HLR surface through
/// [`SLPropsATool`]; `SLProps::new(&surface, 2, 1e-7)?`.
pub type SLProps<'a> = SLPropsBase<&'a Surface, ToolAccess<SLPropsATool>>;

impl SLPropsATool {
    /// Value(A, U, V, P); `SLPropsATool::value(&surface, 0.5, 0.5)?`.
    pub fn value(a: &Surface, u: f64, v: f64) -> Result<Point3, Failure> {
        a.value(u, v)
    }
    /// D1(A, U, V, P, D1U, D1V); `SLPropsATool::d1(&surface, 0.5, 0.5)?`.
    pub fn d1(a: &Surface, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        a.d1(u, v)
    }
    /// D2(A, U, V, P, D1U, D1V, D2U, D2V, DUV); `SLPropsATool::d2(&surface, 0.5, 0.5)?`.
    pub fn d2(a: &Surface, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        a.d2(u, v)
    }
    /// DN(A, U, V, Nu, Nv); `SLPropsATool::dn(&surface, 0.5, 0.5, 1, 1)?`.
    pub fn dn(a: &Surface, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        a.dn(u, v, nu, nv)
    }
    /// Continuity(A): 2 ("et boum ! cky le 27 - 04 - 1993");
    /// `SLPropsATool::continuity(&surface)`.
    pub fn continuity(_a: &Surface) -> i32 {
        2
    }
    /// Bounds(A, U1, V1, U2, V2): `(RealFirst, RealFirst, RealLast, RealLast)`;
    /// `SLPropsATool::bounds(&surface)`.
    pub fn bounds(_a: &Surface) -> PropsBounds {
        (real_first(), real_first(), real_last(), real_last())
    }
}

impl<'a> SurfaceTool<&'a Surface> for SLPropsATool {
    fn value(surface: &&'a Surface, u: f64, v: f64) -> Result<Point3, Failure> {
        SLPropsATool::value(surface, u, v)
    }
    fn d1(surface: &&'a Surface, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        SLPropsATool::d1(surface, u, v)
    }
    fn d2(surface: &&'a Surface, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        SLPropsATool::d2(surface, u, v)
    }
    fn bounds(surface: &&'a Surface) -> Result<PropsBounds, Failure> {
        Ok(SLPropsATool::bounds(surface))
    }
}
