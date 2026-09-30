// Rust translation of ModelingData/TKGeomBase/GeomLProp/GeomLProp_SurfaceUtils.hxx
// (bounds helpers and access policies) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The access policies of `LProp_SurfaceUtils` and its bounds helpers.
//!
//! `DirectAccess` evaluates a surface through its own members ([`DirectSurface`],
//! implemented by the `Geom_Surface` and `Adaptor3d_Surface` handles) and
//! `ToolAccess<Tool>` through the static members of a tool ([`SurfaceTool`]). A null
//! handle gives Standard_NullObject where the native `Deref` dereferences it.
use crate::lprop::curve_access::live;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomSurface, GeomSurfaceD1, GeomSurfaceD2};
use rustycad_tkmath::gp::Point3;
use std::marker::PhantomData;

/// Parametric bounds in the `LProp_SurfaceUtils` order `(U1, V1, U2, V2)`.
pub type PropsBounds = (f64, f64, f64, f64);

/// `GetSurfBounds(const Geom_Surface&, ...)`: `Bounds(U1, U2, V1, V2)` reordered;
/// `get_surf_bounds_geom(&*surface)`.
pub fn get_surf_bounds_geom(surface: &dyn GeomSurface) -> PropsBounds {
    let (u1, u2, v1, v2) = surface.bounds();
    (u1, v1, u2, v2)
}

/// `GetSurfBounds(const Adaptor3d_Surface&, ...)`: the four parameter bounds read
/// in the order U1, V1, U2, V2; `get_surf_bounds_adaptor(&*adaptor)?`.
pub fn get_surf_bounds_adaptor(surface: &dyn Adaptor3dSurface) -> Result<PropsBounds, Failure> {
    let u1: f64 = surface.first_u_parameter()?;
    let v1: f64 = surface.first_v_parameter()?;
    let u2: f64 = surface.last_u_parameter()?;
    let v2: f64 = surface.last_v_parameter()?;
    Ok((u1, v1, u2, v2))
}

/// An access policy: how the surface property algorithms evaluate a surface of
/// type `S` (the native `Access` template parameter);
/// `<DirectAccess as SurfaceAccess<_>>::d0(&surface, 0.5, 0.5)?`.
pub trait SurfaceAccess<S> {
    /// Point at `(u, v)` (`Access::D0`).
    fn d0(surface: &S, u: f64, v: f64) -> Result<Point3, Failure>;
    /// Point and first partial derivatives (`Access::D1`).
    fn d1(surface: &S, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure>;
    /// Point and partial derivatives up to order 2 (`Access::D2`).
    fn d2(surface: &S, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure>;
    /// Bounds `(U1, V1, U2, V2)` (`Access::Bounds`).
    fn bounds(surface: &S) -> Result<PropsBounds, Failure>;
}

/// A surface that evaluates itself, as `DirectAccess` requires;
/// `surface.props_d0(0.5, 0.5)?`.
pub trait DirectSurface {
    /// `D0(U, V, P)`.
    fn props_d0(&self, u: f64, v: f64) -> Result<Point3, Failure>;
    /// `D1(U, V, P, D1U, D1V)`.
    fn props_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure>;
    /// `D2(U, V, P, D1U, D1V, D2U, D2V, D2UV)`.
    fn props_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure>;
    /// `GetSurfBounds(Deref(S), U1, V1, U2, V2)`.
    fn props_bounds(&self) -> Result<PropsBounds, Failure>;
}

impl DirectSurface for Handle<dyn GeomSurface> {
    fn props_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        live(self)?.d0(u, v)
    }
    fn props_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        live(self)?.d1(u, v)
    }
    fn props_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        live(self)?.d2(u, v)
    }
    fn props_bounds(&self) -> Result<PropsBounds, Failure> {
        Ok(get_surf_bounds_geom(live(self)?))
    }
}

impl DirectSurface for Handle<dyn Adaptor3dSurface> {
    fn props_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        live(self)?.d0(u, v)
    }
    fn props_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        live(self)?.d1(u, v)
    }
    fn props_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        live(self)?.d2(u, v)
    }
    fn props_bounds(&self) -> Result<PropsBounds, Failure> {
        get_surf_bounds_adaptor(live(self)?)
    }
}

/// `LProp_SurfaceUtils::DirectAccess`: evaluates the surface through its own
/// members; `SLPropsBase::<Handle<dyn GeomSurface>, DirectAccess>`.
#[derive(Clone, Copy, Debug, Default)]
pub struct DirectAccess;

impl<S: DirectSurface> SurfaceAccess<S> for DirectAccess {
    fn d0(surface: &S, u: f64, v: f64) -> Result<Point3, Failure> {
        surface.props_d0(u, v)
    }
    fn d1(surface: &S, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        surface.props_d1(u, v)
    }
    fn d2(surface: &S, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        surface.props_d2(u, v)
    }
    fn bounds(surface: &S) -> Result<PropsBounds, Failure> {
        surface.props_bounds()
    }
}

/// The static tool interface of `ToolAccess` (the HLRBRep surface tool natively):
/// `Tool::Value`, `D1`, `D2` and `Bounds`.
pub trait SurfaceTool<S> {
    /// `Tool::Value(S, U, V, P)`.
    fn value(surface: &S, u: f64, v: f64) -> Result<Point3, Failure>;
    /// `Tool::D1(S, U, V, P, D1U, D1V)`.
    fn d1(surface: &S, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure>;
    /// `Tool::D2(S, U, V, P, D1U, D1V, D2U, D2V, D2UV)`.
    fn d2(surface: &S, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure>;
    /// `Tool::Bounds(S, U1, V1, U2, V2)`.
    fn bounds(surface: &S) -> Result<PropsBounds, Failure>;
}

/// `LProp_SurfaceUtils::ToolAccess<Tool>`: evaluates the surface through the
/// static members of `Tool`; `SLPropsBase::<MySurface, ToolAccess<MyTool>>`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ToolAccess<Tool>(PhantomData<Tool>);

impl<S, Tool: SurfaceTool<S>> SurfaceAccess<S> for ToolAccess<Tool> {
    fn d0(surface: &S, u: f64, v: f64) -> Result<Point3, Failure> {
        Tool::value(surface, u, v)
    }
    fn d1(surface: &S, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        Tool::d1(surface, u, v)
    }
    fn d2(surface: &S, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        Tool::d2(surface, u, v)
    }
    fn bounds(surface: &S) -> Result<PropsBounds, Failure> {
        Tool::bounds(surface)
    }
}
