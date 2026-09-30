// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_RepUtils.pxx (TryEvalSurface*) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Surface evaluation through a descriptor, with the parameter map (scales and
//! U/V swap) applied to the partial derivatives.
use crate::geom::errors;
use crate::geom::{GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3};
use crate::geom_eval::{Map2d, RepSurfaceDesc};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point3, Vector3};

// The representation to evaluate and the mapped parameters, or `None` when the
// native TryEvalSurface* returns false before evaluating.
struct Redirect {
    surface: Handle<dyn GeomSurface>,
    u: f64,
    v: f64,
    map: Option<Map2d>,
}

// Order limit, domain and parameter map shared by the TryEvalSurface* functions.
fn redirect(
    desc: &Handle<dyn RepSurfaceDesc>,
    order: usize,
    u: f64,
    v: f64,
) -> Result<Option<Redirect>, Failure> {
    let Some(desc) = desc.get() else {
        return Ok(None);
    };
    if desc.max_deriv_order().is_some_and(|limit| limit < order) {
        return Ok(None);
    }
    let surface: Handle<dyn GeomSurface> = desc.representation();
    if surface.is_null() {
        return Err(errors::program("GeomEval_RepUtils: null Representation"));
    }
    let Some((domain, map)) = desc.mapping() else {
        return Ok(Some(Redirect {
            surface,
            u,
            v,
            map: None,
        }));
    };
    let (u_rep, v_rep) = map.map(u, v);
    if domain.is_some_and(|domain| !domain.contains(u_rep, v_rep)) {
        return Ok(None);
    }
    let map = Some(map);
    Ok(Some(Redirect {
        surface,
        u: u_rep,
        v: v_rep,
        map,
    }))
}

/// Point through the descriptor (`TryEvalSurfaceD0`); `try_eval_surface_d0(&desc, u, v)?`.
pub fn try_eval_surface_d0(
    desc: &Handle<dyn RepSurfaceDesc>,
    u: f64,
    v: f64,
) -> Result<Option<Point3>, Failure> {
    let Some(target) = redirect(desc, 0, u, v)? else {
        return Ok(None);
    };
    target.surface.eval_d0(target.u, target.v).map(Some)
}

/// Point and first derivatives through the descriptor (`TryEvalSurfaceD1`);
/// `try_eval_surface_d1(&desc, u, v)?`.
pub fn try_eval_surface_d1(
    desc: &Handle<dyn RepSurfaceDesc>,
    u: f64,
    v: f64,
) -> Result<Option<GeomSurfaceD1>, Failure> {
    let Some(target) = redirect(desc, 1, u, v)? else {
        return Ok(None);
    };
    let rep: GeomSurfaceD1 = target.surface.eval_d1(target.u, target.v)?;
    let Some(map) = target.map else {
        return Ok(Some(rep));
    };
    let (d1u, d1v) = if map.swap_uv {
        (rep.d1v * map.scale_v, rep.d1u * map.scale_u)
    } else {
        (rep.d1u * map.scale_u, rep.d1v * map.scale_v)
    };
    Ok(Some(GeomSurfaceD1 {
        point: rep.point,
        d1u,
        d1v,
    }))
}

/// Point and derivatives up to order 2 through the descriptor (`TryEvalSurfaceD2`);
/// `try_eval_surface_d2(&desc, u, v)?`.
pub fn try_eval_surface_d2(
    desc: &Handle<dyn RepSurfaceDesc>,
    u: f64,
    v: f64,
) -> Result<Option<GeomSurfaceD2>, Failure> {
    let Some(target) = redirect(desc, 2, u, v)? else {
        return Ok(None);
    };
    let rep: GeomSurfaceD2 = target.surface.eval_d2(target.u, target.v)?;
    let Some(map) = target.map else {
        return Ok(Some(rep));
    };
    Ok(Some(mapped_d2(rep, map)))
}

// The D2 scaling of TryEvalSurfaceD2Mapped.
fn mapped_d2(rep: GeomSurfaceD2, map: Map2d) -> GeomSurfaceD2 {
    let (su, sv) = (map.scale_u, map.scale_v);
    let (su2, sv2, susv) = (su * su, sv * sv, su * sv);
    if map.swap_uv {
        return GeomSurfaceD2 {
            point: rep.point,
            d1u: rep.d1v * sv,
            d1v: rep.d1u * su,
            d2u: rep.d2v * sv2,
            d2v: rep.d2u * su2,
            d2uv: rep.d2uv * susv,
        };
    }
    GeomSurfaceD2 {
        point: rep.point,
        d1u: rep.d1u * su,
        d1v: rep.d1v * sv,
        d2u: rep.d2u * su2,
        d2v: rep.d2v * sv2,
        d2uv: rep.d2uv * susv,
    }
}

/// Point and derivatives up to order 3 through the descriptor (`TryEvalSurfaceD3`);
/// `try_eval_surface_d3(&desc, u, v)?`.
pub fn try_eval_surface_d3(
    desc: &Handle<dyn RepSurfaceDesc>,
    u: f64,
    v: f64,
) -> Result<Option<GeomSurfaceD3>, Failure> {
    let Some(target) = redirect(desc, 3, u, v)? else {
        return Ok(None);
    };
    let rep: GeomSurfaceD3 = target.surface.eval_d3(target.u, target.v)?;
    let Some(map) = target.map else {
        return Ok(Some(rep));
    };
    if map.swap_uv {
        return Ok(Some(swapped_d3(rep, map.scale_u, map.scale_v)));
    }
    Ok(Some(scaled_d3(rep, map.scale_u, map.scale_v)))
}

// The swapped branch of TryEvalSurfaceD3Mapped, with its native products.
fn swapped_d3(rep: GeomSurfaceD3, su: f64, sv: f64) -> GeomSurfaceD3 {
    GeomSurfaceD3 {
        point: rep.point,
        d1u: rep.d1v * sv,
        d1v: rep.d1u * su,
        d2u: rep.d2v * (sv * sv),
        d2v: rep.d2u * (su * su),
        d2uv: rep.d2uv * (su * sv),
        d3u: rep.d3v * (sv * sv * sv),
        d3v: rep.d3u * (su * su * su),
        d3uuv: rep.d3uvv * (sv * sv * su),
        d3uvv: rep.d3uuv * (su * su * sv),
    }
}

// The direct branch of TryEvalSurfaceD3Mapped, with its native products.
fn scaled_d3(rep: GeomSurfaceD3, su: f64, sv: f64) -> GeomSurfaceD3 {
    GeomSurfaceD3 {
        point: rep.point,
        d1u: rep.d1u * su,
        d1v: rep.d1v * sv,
        d2u: rep.d2u * (su * su),
        d2v: rep.d2v * (sv * sv),
        d2uv: rep.d2uv * (su * sv),
        d3u: rep.d3u * (su * su * su),
        d3v: rep.d3v * (sv * sv * sv),
        d3uuv: rep.d3uuv * (su * su * sv),
        d3uvv: rep.d3uvv * (su * sv * sv),
    }
}

/// Derivative of orders `(nu, nv)` through the descriptor; the orders are swapped
/// with the parameters and the scale is `ScaleU^nu * ScaleV^nv` with `std::pow`;
/// `None` for negative orders or `nu + nv < 1` (`TryEvalSurfaceDN`);
/// `try_eval_surface_dn(&desc, u, v, 1, 1)?`.
pub fn try_eval_surface_dn(
    desc: &Handle<dyn RepSurfaceDesc>,
    u: f64,
    v: f64,
    nu: i32,
    nv: i32,
) -> Result<Option<Vector3>, Failure> {
    let total: i32 = nu + nv;
    if nu < 0 || nv < 0 || total < 1 {
        return Ok(None);
    }
    let Some(target) = redirect(desc, total as usize, u, v)? else {
        return Ok(None);
    };
    let Some(map) = target.map else {
        return target.surface.eval_dn(target.u, target.v, nu, nv).map(Some);
    };
    let (nu_rep, nv_rep) = if map.swap_uv { (nv, nu) } else { (nu, nv) };
    let rep: Vector3 = target.surface.eval_dn(target.u, target.v, nu_rep, nv_rep)?;
    let scale: f64 = map.scale_u.powf(f64::from(nu_rep)) * map.scale_v.powf(f64::from(nv_rep));
    Ok(Some(rep * scale))
}
