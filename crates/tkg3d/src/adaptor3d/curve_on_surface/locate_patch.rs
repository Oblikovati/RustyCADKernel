// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_CurveOnSurface.cxx (Locate1Coord and Locate2Coord on a B-spline surface, LocatePart, LocatePart_RevExt, LocatePart_Offset, EvalFirstLastSurf) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `EvalFirstLastSurf`: the surface trimmed to the knot patch that the curve enters
//! at its first parameter (and, backwards, at its last), evaluated instead of the
//! whole surface at the curve ends.
use super::locate::{
    Patch, TOL, component, directed, find_bounds, hunt, locate1_curve, locate2_range, reverse_param,
};
use super::{Adaptor3dCurveOnSurface, CosState};
use crate::adaptor3d::Adaptor3dSurface;
use crate::geom::GeomBSplineSurface;
use crate::geom::errors::not_implemented;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Point2, Vector2};

// The knots of one direction of a B-spline surface, with its first and last indices.
struct Direction<'a> {
    surface: &'a GeomBSplineSurface,
    u: bool,
}
impl Direction<'_> {
    fn knot(&self, index: i32) -> Result<f64, Failure> {
        if self.u {
            self.surface.u_knot(index)
        } else {
            self.surface.v_knot(index)
        }
    }
    fn knots(&self) -> Array1<'static, f64> {
        if self.u {
            self.surface.u_knots()
        } else {
            self.surface.v_knots()
        }
    }
    fn range(&self) -> Result<(i32, i32), Failure> {
        if self.u {
            Ok((
                self.surface.first_u_knot_index()?,
                self.surface.last_u_knot_index()?,
            ))
        } else {
            Ok((
                self.surface.first_v_knot_index()?,
                self.surface.last_v_knot_index()?,
            ))
        }
    }
}

/// `Locate1Coord` on a B-spline surface: the knot span of coordinate `index` (1 = U,
/// 2 = V) that the curve enters; returns `DIsNull`, true when the coordinate is on a
/// knot and its derivative is null (the span is then left unset);
/// `locate1_surface(1, (uv, duv), &s, &mut patch)?`.
pub(super) fn locate1_surface(
    index: i32,
    (uv, duv): (Point2, Vector2),
    surface: &GeomBSplineSurface,
    patch: &mut Patch,
) -> Result<bool, Failure> {
    if index != 1 && index != 2 {
        return Ok(false);
    }
    let knots = Direction {
        surface,
        u: index == 1,
    };
    let (comp, dcomp) = component(index, uv, duv);
    let (down, up) = knots.range()?;
    let mut i: i32 = down;
    while (knots.knot(i)? - comp).abs() > TOL && i != up {
        i += 1;
    }
    let cur: f64 = knots.knot(i)?;
    if (comp - cur).abs() <= TOL {
        let found = find_bounds(&knots.knots(), cur, dcomp, (down, up))?;
        let (bnd1, bnd2) = reverse_param(found.first, found.second);
        if !found.der_null {
            patch.set(index, knots.knot(bnd1)?, knots.knot(bnd2)?);
        }
        return Ok(found.der_null);
    }
    locate1_surface_between(index, (comp, dcomp), &knots, (down, up), patch)?;
    Ok(false)
}

// The coordinate off the knots: the first span below the first knot, the last span
// above the last knot (its corners written in reverse, as natively), else the span
// containing it, from the coordinate towards the direction.
// The negated comparisons keep the native NaN behaviour.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn locate1_surface_between(
    index: i32,
    (comp, dcomp): (f64, f64),
    knots: &Direction<'_>,
    (down, up): (i32, i32),
    patch: &mut Patch,
) -> Result<(), Failure> {
    if comp < knots.knot(down)? {
        patch.set(index, knots.knot(down)?, knots.knot(down + 1)?);
        return Ok(());
    }
    if comp > knots.knot(up)? {
        patch.set(index, knots.knot(up)?, knots.knot(up - 1)?);
        return Ok(());
    }
    let (mut f, mut l) = (knots.knot(down)?, knots.knot(up)?);
    let mut i: i32 = down;
    if !(comp < f) && !(comp > l) {
        loop {
            f = knots.knot(i)?;
            let outside: bool = f >= comp || {
                l = knots.knot(i + 1)?;
                l <= comp
            };
            if !(outside && i < up) {
                break;
            }
            i += 1;
        }
    } else {
        (f, l) = reverse_param(f, l);
    }
    if i == up {
        patch.set(index, comp, knots.knot(i)?);
    } else if dcomp.abs() > TOL {
        directed(index, (comp, dcomp), (f, l), patch);
    } else if dcomp.abs() < TOL {
        patch.set(index, f, l);
    }
    Ok(())
}

/// `Locate2Coord` on a B-spline surface: for a coordinate on the knot `Hunt` finds,
/// the span on the side the other coordinate's derivative points to (clamped to the
/// end spans); `locate2_surface(1, (uv, duv), &s, &mut patch)?`.
pub(super) fn locate2_surface(
    index: i32,
    (uv, duv): (Point2, Vector2),
    surface: &GeomBSplineSurface,
    patch: &mut Patch,
) -> Result<(), Failure> {
    let knots = Direction {
        surface,
        u: index == 1,
    };
    let (comp, dcomp) = match index {
        1 => (uv.x(), duv.y()),
        2 => (uv.y(), duv.x()),
        _ => return Ok(()),
    };
    let (low, up) = knots.range()?;
    let mut n: i32 = 0;
    if dcomp > 0. && dcomp.abs() > TOL {
        hunt(&knots.knots(), comp, &mut n)?;
        // Limit case: Hunt() caught the upper knot. Take the last span.
        if n >= up {
            n = up - 1;
        }
        let (a, b) = reverse_param(knots.knot(n)?, knots.knot(n + 1)?);
        patch.set(index, a, b);
    } else if dcomp < 0. && dcomp.abs() > TOL {
        hunt(&knots.knots(), comp, &mut n)?;
        // Limit case: Hunt() caught the lower knot. Take the first span.
        if n <= low {
            n = low + 1;
        }
        let (a, b) = reverse_param(knots.knot(n - 1)?, knots.knot(n)?);
        patch.set(index, a, b);
    }
    Ok(())
}

/// `LocatePart` on a B-spline surface: both coordinates, and for one on a knot with
/// a null derivative the span on the side of the other derivative;
/// `locate_part((uv, duv), &*surface, &mut patch)?`.
pub(super) fn locate_part(
    point: (Point2, Vector2),
    surface: &dyn Adaptor3dSurface,
    patch: &mut Patch,
) -> Result<(), Failure> {
    let bspline: Handle<GeomBSplineSurface> = surface.bspline()?;
    let du_null: bool = locate1_surface(1, point, &bspline, patch)?;
    let dv_null: bool = locate1_surface(2, point, &bspline, patch)?;
    if du_null && !dv_null {
        locate2_surface(1, point, &bspline, patch)?;
    } else if dv_null && !du_null {
        locate2_surface(2, point, &bspline, patch)?;
    }
    Ok(())
}

/// `LocatePart_RevExt`: on a swept surface with a B-spline basis, the basis knot span
/// in the swept direction and the range in the other; false for another basis;
/// `locate_part_rev_ext((uv, duv), &*surface, &mut patch)?`.
pub(super) fn locate_part_rev_ext(
    point: (Point2, Vector2),
    surface: &dyn Adaptor3dSurface,
    patch: &mut Patch,
) -> Result<bool, Failure> {
    let basis = surface.basis_curve()?;
    if basis.get_type()? != CurveType::BSplineCurve {
        return Ok(false);
    }
    let curve = basis.bspline()?;
    if surface.get_type()? == SurfaceType::SurfaceOfExtrusion {
        locate1_curve(1, point, &curve, patch)?;
        let range = (surface.first_v_parameter()?, surface.last_v_parameter()?);
        locate2_range(2, point, range, patch);
    } else if surface.get_type()? == SurfaceType::SurfaceOfRevolution {
        locate1_curve(2, point, &curve, patch)?;
        let range = (surface.first_u_parameter()?, surface.last_u_parameter()?);
        locate2_range(1, point, range, patch);
    }
    patch.order();
    Ok(true)
}

/// `LocatePart_Offset`: the patch of the offset basis (swept or B-spline); false for
/// any other basis; `locate_part_offset((uv, duv), &*surface, &mut patch)?`.
pub(super) fn locate_part_offset(
    point: (Point2, Vector2),
    surface: &dyn Adaptor3dSurface,
    patch: &mut Patch,
) -> Result<bool, Failure> {
    let basis = surface.basis_surface()?;
    match basis.get_type()? {
        SurfaceType::SurfaceOfRevolution | SurfaceType::SurfaceOfExtrusion => {
            locate_part_rev_ext(point, &*basis, patch)
        }
        SurfaceType::BSplineSurface => {
            locate_part(point, &*basis, patch)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

impl Adaptor3dCurveOnSurface {
    /// `EvalFirstLastSurf`: store the surfaces trimmed to the patches entered at the
    /// first parameter and, backwards, at the last one (the whole surface where the
    /// curve has a null derivative or the basis is not a spline); `self.eval_first_last_surf()?`.
    pub(super) fn eval_first_last_surf(&self) -> Result<(), Failure> {
        let state: CosState = self.snapshot();
        let curve = state.curve()?;
        // The corners are shared by both ends, as the native locals are.
        let mut patch = Patch::default();
        let (uv, duv) = curve.d1(curve.first_parameter()?)?;
        let first = end_surface(&state.surface, (uv, duv), &mut patch)?;
        self.state.write().first_surface = first;
        let (uv, duv) = curve.d1(curve.last_parameter()?)?;
        // We want the other part.
        let last = end_surface(&state.surface, (uv, duv.reversed()), &mut patch)?;
        self.state.write().last_surface = last;
        Ok(())
    }
}

// The trimmed surface at one end, or the whole surface.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn end_surface(
    surface: &Handle<dyn Adaptor3dSurface>,
    point: (Point2, Vector2),
    patch: &mut Patch,
) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
    // Natively `if (DUV.Magnitude() <= Tol) Ok = false`: a NaN derivative keeps Ok.
    let mut ok: bool = !(point.1.magnitude() <= TOL);
    if ok {
        ok = match surface.get_type()? {
            SurfaceType::BSplineSurface => locate_part(point, &**surface, patch).map(|_| true)?,
            SurfaceType::SurfaceOfRevolution | SurfaceType::SurfaceOfExtrusion => {
                locate_part_rev_ext(point, &**surface, patch)?
            }
            SurfaceType::OffsetSurface => locate_part_offset(point, &**surface, patch)?,
            _ => {
                return Err(not_implemented(
                    "Adaptor3d_CurveOnSurface::EvalFirstLastSurf",
                ));
            }
        };
    }
    if !ok {
        return Ok(surface.clone());
    }
    patch.compare();
    let (left, right) = (patch.left_bot, patch.right_top);
    let trimmed = surface.u_trim(left.x(), right.x(), TOL)?;
    trimmed.v_trim(left.y(), right.y(), TOL)
}
