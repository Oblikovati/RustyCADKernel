// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_CurveOnSurface.cxx (EvalD0..EvalDN and the file-local SetLinearForm) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Evaluation of a curve on a surface by the chain rule: `C'(t) = S_u u' + S_v v'`
//! and its second and third orders, on the surface trimmed to the end patch within
//! `PConfusion / 10` of the curve bounds, by ElCLib on a recognised line or circle,
//! on the surface otherwise.
use super::CosState;
use super::locate::TOL;
use crate::adaptor3d::Adaptor3dSurface;
use crate::geom::{GeomCurveD1, GeomCurveD2, GeomCurveD3, GeomSurfaceD2, GeomSurfaceD3};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

const ZERO: Vector3 = Vector3::new(0., 0., 0.);

/// `EvalD0`: ElCLib on a line or circle, else the surface at the 2D point (never the
/// end patches); `d0(&state, 0.5)?`.
pub(super) fn d0(state: &CosState, u: f64) -> Result<Point3, Failure> {
    match state.kind {
        CurveType::Line => Ok(elclib::value(u, &state.line)),
        CurveType::Circle => Ok(elclib::value(u, &state.circle)),
        _ => {
            let puv: Point2 = state.curve()?.d0(u)?;
            state.surface()?.d0(puv.x(), puv.y())
        }
    }
}

// The surface of an evaluation at `u`: the first (last) patch within TOL of the first
// (last) curve parameter, else None (the line, circle or whole-surface branches).
fn end_patch(state: &CosState, u: f64) -> Result<Option<&Handle<dyn Adaptor3dSurface>>, Failure> {
    let curve = state.curve()?;
    let (first, last) = (curve.first_parameter()?, curve.last_parameter()?);
    if (u - first).abs() < TOL && state.first_surface.is_some() {
        return Ok(Some(&state.first_surface));
    }
    if (u - last).abs() < TOL && state.last_surface.is_some() {
        return Ok(Some(&state.last_surface));
    }
    Ok(None)
}

/// `EvalD1`; `d1(&state, 0.5)?`.
pub(super) fn d1(state: &CosState, u: f64) -> Result<GeomCurveD1, Failure> {
    let surface: &dyn Adaptor3dSurface = match end_patch(state, u)? {
        Some(patch) => &**patch,
        None if state.kind == CurveType::Line => {
            let (point, d1) = elclib::d1(u, &state.line);
            return Ok(GeomCurveD1 { point, d1 });
        }
        None if state.kind == CurveType::Circle => {
            let (point, d1) = elclib::d1(u, &state.circle);
            return Ok(GeomCurveD1 { point, d1 });
        }
        None => state.surface()?,
    };
    let (puv, duv): (Point2, Vector2) = state.curve()?.d1(u)?;
    let s = surface.d1(puv.x(), puv.y())?;
    let mut d1 = Vector3::default();
    d1.set_linear_form(duv.x(), s.d1u, duv.y(), s.d1v);
    Ok(GeomCurveD1 { point: s.point, d1 })
}

// C'' from the surface second derivatives: first the curve-second-order and mixed
// terms, then the pure second-order terms on top.
fn second(dw: Vector2, d2w: Vector2, s: &GeomSurfaceD2) -> Vector3 {
    let mut d2 = Vector3::default();
    d2.set_linear_form_three(
        [d2w.x(), d2w.y(), 2. * dw.x() * dw.y()],
        [s.d1u, s.d1v, s.d2uv],
    );
    let partial = d2;
    d2.set_linear_form_offset(dw.x() * dw.x(), s.d2u, dw.y() * dw.y(), s.d2v, partial);
    d2
}

/// `EvalD2`; `d2(&state, 0.5)?`.
pub(super) fn d2(state: &CosState, u: f64) -> Result<GeomCurveD2, Failure> {
    let surface: &dyn Adaptor3dSurface = match end_patch(state, u)? {
        Some(patch) => &**patch,
        None if state.kind == CurveType::Line => {
            let (point, d1) = elclib::d1(u, &state.line);
            return Ok(GeomCurveD2 {
                point,
                d1,
                d2: ZERO,
            });
        }
        None if state.kind == CurveType::Circle => {
            let (point, d1, d2) = elclib::d2(u, &state.circle);
            return Ok(GeomCurveD2 { point, d1, d2 });
        }
        None => state.surface()?,
    };
    let (uv, dw, d2w) = state.curve()?.d2(u)?;
    let s: GeomSurfaceD2 = surface.d2(uv.x(), uv.y())?;
    let mut d1 = Vector3::default();
    d1.set_linear_form(dw.x(), s.d1u, dw.y(), s.d1v);
    Ok(GeomCurveD2 {
        point: s.point,
        d1,
        d2: second(dw, d2w, &s),
    })
}

/// `EvalD3`; `d3(&state, 0.5)?`.
pub(super) fn d3(state: &CosState, u: f64) -> Result<GeomCurveD3, Failure> {
    let surface: &dyn Adaptor3dSurface = match end_patch(state, u)? {
        Some(patch) => &**patch,
        None if state.kind == CurveType::Line => {
            let (point, d1) = elclib::d1(u, &state.line);
            return Ok(GeomCurveD3 {
                point,
                d1,
                d2: ZERO,
                d3: ZERO,
            });
        }
        None if state.kind == CurveType::Circle => {
            let (point, d1, d2, d3) = elclib::d3(u, &state.circle);
            return Ok(GeomCurveD3 { point, d1, d2, d3 });
        }
        None => state.surface()?,
    };
    let (uv, dw, d2w, d3w) = state.curve()?.d3(u)?;
    let s: GeomSurfaceD3 = surface.d3(uv.x(), uv.y())?;
    let second_order = GeomSurfaceD2 {
        point: s.point,
        d1u: s.d1u,
        d1v: s.d1v,
        d2u: s.d2u,
        d2v: s.d2v,
        d2uv: s.d2uv,
    };
    let mut d1 = Vector3::default();
    d1.set_linear_form(dw.x(), s.d1u, dw.y(), s.d1v);
    let d2 = second(dw, d2w, &second_order);
    Ok(GeomCurveD3 {
        point: s.point,
        d1,
        d2,
        d3: third(dw, d2w, d3w, &s),
    })
}

/// The file-local `SetLinearForm`: C''' from the surface derivatives up to the third,
/// in the native grouping (V31..V34; V31 weights D1U by DW.X, not D3W.X, as natively);
/// `third(dw, d2w, d3w, &s)`.
fn third(dw: Vector2, d2w: Vector2, d3w: Vector2, s: &GeomSurfaceD3) -> Vector3 {
    let (x, y) = (dw.x(), dw.y());
    let mut v31 = Vector3::default();
    v31.set_linear_form_three([x, d2w.x() * x, d2w.x() * y], [s.d1u, s.d2u, s.d2uv]);
    let partial = v31;
    v31.set_linear_form_three_offset(
        [d3w.y(), d2w.y() * x, d2w.y() * y],
        [s.d1v, s.d2uv, s.d2v],
        partial,
    );
    let mut v32 = Vector3::default();
    v32.set_linear_form(x * x * y, s.d3uuv, x * y * y, s.d3uvv);
    let partial = v32;
    v32.set_linear_form_offset(
        d2w.x() * y + x * d2w.y(),
        s.d2uv,
        x * y * y,
        s.d3uvv,
        partial,
    );
    let mut v33 = Vector3::default();
    v33.set_linear_form_three(
        [2. * d2w.x() * x, x * x * x, x * x * y],
        [s.d2u, s.d3u, s.d3uuv],
    );
    let mut v34 = Vector3::default();
    v34.set_linear_form_three(
        [2. * d2w.y() * y, y * y * x, y * y * y],
        [s.d2v, s.d3uvv, s.d3v],
    );
    let mut v3 = Vector3::default();
    v3.set_linear_form_three_offset([1., 2., 1.], [v31, v32, v33], v34);
    v3
}
