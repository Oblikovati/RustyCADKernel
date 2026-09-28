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
// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_CurveOnSurface.cxx (CompareBounds, Hunt, ReverseParam, FindBounds, Locate1Coord on a curve, Locate2Coord on a range) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The knot-patch search of `EvalFirstLastSurf`: from a point `UV` of the curve and
//! its direction `DUV`, the parameter rectangle `[LeftBot, RightTop]` of the surface
//! patch the curve leaves the point into. One coordinate is located at a time, on
//! the knots of a B-spline curve or surface, or on a parameter range.
use crate::geom::GeomBSplineCurve;
use crate::geom::errors::{array_bounds, not_implemented};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Vector2};

/// `PConfusion() / 10`, the knot tolerance of every search.
pub(super) const TOL: f64 = precision::p_confusion() / 10.;

/// The patch corners `LeftBot` and `RightTop` (gp_Pnt2d, zero until written).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Patch {
    pub(super) left_bot: Point2,
    pub(super) right_top: Point2,
}
impl Patch {
    /// Set coordinate `index` (1 = X, 2 = Y) of both corners; `patch.set(1, a, b)`.
    pub(super) fn set(&mut self, index: i32, left: f64, right: f64) {
        if index == 1 {
            self.left_bot.set_x(left);
            self.right_top.set_x(right);
        } else if index == 2 {
            self.left_bot.set_y(left);
            self.right_top.set_y(right);
        }
    }
    /// `CompareBounds` (SVV): order each coordinate of the corners; `patch.compare()`.
    pub(super) fn compare(&mut self) {
        let (lx, ly) = (self.left_bot.x(), self.left_bot.y());
        let (rx, ry) = (self.right_top.x(), self.right_top.y());
        if lx > rx {
            self.left_bot.set_x(rx);
            self.right_top.set_x(lx);
        }
        if ly > ry {
            self.left_bot.set_y(ry);
            self.right_top.set_y(ly);
        }
    }
    /// `ReverseParam` on both coordinates of the corners; `patch.order()`.
    pub(super) fn order(&mut self) {
        let (u1, u2) = reverse_param(self.left_bot.x(), self.right_top.x());
        self.set(1, u1, u2);
        let (v1, v2) = reverse_param(self.left_bot.y(), self.right_top.y());
        self.set(2, v1, v2);
    }
}

/// The coordinate `index` (1 = X, 2 = Y) of a point and of a direction.
pub(super) fn component(index: i32, uv: Point2, duv: Vector2) -> (f64, f64) {
    match index {
        1 => (uv.x(), duv.x()),
        2 => (uv.y(), duv.y()),
        _ => (0., 0.),
    }
}

/// `ReverseParam`: the pair in increasing order; `reverse_param(2.0, 1.0) == (1.0, 2.0)`.
pub(super) fn reverse_param<T: PartialOrd>(first: T, second: T) -> (T, T) {
    if first > second {
        (second, first)
    } else {
        (first, second)
    }
}

fn knot(knots: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    knots.value(index).copied().map_err(array_bounds)
}

/// `Hunt`: the index of the knot equal to `coord` (called only when one is); a coord
/// exactly `TOL` from its knot leaves `found` unchanged, and one farther from every
/// knot raises Standard_NotImplemented. The native scan reads one past the array
/// when no knot matches; here that read raises Standard_OutOfRange;
/// `hunt(&knots, 0.5, &mut n)?`.
pub(super) fn hunt(knots: &Array1<'_, f64>, coord: f64, found: &mut i32) -> Result<(), Failure> {
    let mut i: i32 = 1;
    while i <= knots.upper() && (coord - knot(knots, i)?).abs() > TOL {
        i += 1;
    }
    let distance: f64 = (coord - knot(knots, i)?).abs();
    if distance < TOL {
        *found = i;
    } else if distance > TOL {
        return Err(not_implemented("Adaptor3d_CurveOnSurface:Hunt"));
    }
    Ok(())
}

/// The outputs of FindBounds: the knot interval and whether the derivative is null.
pub(super) struct Bounds {
    pub(super) first: i32,
    pub(super) second: i32,
    pub(super) der_null: bool,
}

/// `FindBounds`: from the knot `coord` of the interval `(first, second)`, the knot
/// span the curve enters along `der` (both neighbours for a null derivative);
/// `find_bounds(&knots, 0.5, 1.0, (1, 4))?`.
pub(super) fn find_bounds(
    knots: &Array1<'_, f64>,
    coord: f64,
    der: f64,
    (first, second): (i32, i32),
) -> Result<Bounds, Failure> {
    let mut n: i32 = 0;
    hunt(knots, coord, &mut n)?;
    let der_null: bool = der.abs() <= TOL;
    if n == first {
        return Ok(Bounds {
            first: n,
            second: n + 1,
            der_null,
        });
    }
    if n == second {
        return Ok(Bounds {
            first: n - 1,
            second: n,
            der_null,
        });
    }
    if der.abs() > TOL {
        let (a, b) = if der > 0. { (n, n + 1) } else { (n - 1, n) };
        return Ok(Bounds {
            first: a,
            second: b,
            der_null: false,
        });
    }
    if der_null {
        return Ok(Bounds {
            first: n - 1,
            second: n + 1,
            der_null: true,
        });
    }
    // A NaN derivative matches neither native test: the bounds are kept.
    Ok(Bounds {
        first,
        second,
        der_null: false,
    })
}

/// `Locate1Coord` on a B-spline curve (the basis of a swept surface): the knot
/// span of coordinate `index` that the curve enters; `locate1_curve(1, uv, duv, &c, &mut p)?`.
pub(super) fn locate1_curve(
    index: i32,
    (uv, duv): (Point2, Vector2),
    curve: &GeomBSplineCurve,
    patch: &mut Patch,
) -> Result<(), Failure> {
    let (comp, dcomp) = component(index, uv, duv);
    let knots: Array1<'static, f64> = curve.knots();
    let (lo, up) = (curve.first_u_knot_index()?, curve.last_u_knot_index()?);
    let mut i: i32 = lo;
    while (curve.knot(i)? - comp).abs() > TOL && i != up {
        i += 1;
    }
    let cur: f64 = curve.knot(i)?;
    if (comp - cur).abs() <= TOL {
        return locate1_curve_on_knot(index, (cur, comp, dcomp), curve, (&knots, lo, up), patch);
    }
    locate1_curve_between(index, (comp, dcomp), curve, (lo, up), patch)
}

// The coordinate on a knot: FindBounds between the first and last knot indices; a
// null derivative at an end knot takes the end span.
fn locate1_curve_on_knot(
    index: i32,
    (cur, comp, dcomp): (f64, f64, f64),
    curve: &GeomBSplineCurve,
    (knots, lo, up): (&Array1<'_, f64>, i32, i32),
    patch: &mut Patch,
) -> Result<(), Failure> {
    let found: Bounds = find_bounds(knots, cur, dcomp, (lo, up))?;
    let (bnd1, bnd2) = reverse_param(found.first, found.second);
    if !found.der_null {
        patch.set(index, curve.knot(bnd1)?, curve.knot(bnd2)?);
    } else if (comp - curve.knot(lo)?).abs() <= TOL {
        patch.set(index, curve.knot(lo)?, curve.knot(lo + 1)?);
    } else if (comp - curve.knot(up)?).abs() <= TOL {
        patch.set(index, curve.knot(up - 1)?, curve.knot(up)?);
    } else {
        patch.set(index, curve.knot(bnd1)?, curve.knot(bnd2)?);
    }
    Ok(())
}

// The coordinate inside a span (skl 28.03.2002 for OCC233): from the coordinate to
// the span end the curve moves towards, the whole span for a null derivative.
fn locate1_curve_between(
    index: i32,
    (comp, dcomp): (f64, f64),
    curve: &GeomBSplineCurve,
    (lo, up): (i32, i32),
    patch: &mut Patch,
) -> Result<(), Failure> {
    let (mut f, mut l): (f64, f64) = (0., 0.);
    let mut i: i32 = lo;
    while i < up {
        f = curve.knot(i)?;
        l = curve.knot(i + 1)?;
        if f < comp && l > comp {
            break;
        }
        i += 1;
    }
    let (f, l) = reverse_param(f, l);
    if i == up {
        patch.set(index, comp, curve.knot(i)?);
    } else if dcomp.abs() < TOL {
        patch.set(index, f, l);
    } else if dcomp.abs() > TOL {
        directed(index, (comp, dcomp), (f, l), patch);
    }
    Ok(())
}

/// From the coordinate to the end `(f, l)` of its interval that the direction points
/// to; nothing for a zero direction; `directed(1, (c, d), (f, l), &mut patch)`.
pub(super) fn directed(
    index: i32,
    (comp, dcomp): (f64, f64),
    (f, l): (f64, f64),
    patch: &mut Patch,
) {
    if dcomp > 0. {
        patch.set(index, comp, l);
    } else if dcomp < 0. {
        patch.set(index, f, comp);
    }
}

/// `Locate2Coord` on the range `[i1, i2]` of a swept surface's other direction;
/// `locate2_range(2, uv, duv, (v1, v2), &mut patch)`.
pub(super) fn locate2_range(
    index: i32,
    (uv, duv): (Point2, Vector2),
    (i1, i2): (f64, f64),
    patch: &mut Patch,
) {
    let (comp, dcomp) = component(index, uv, duv);
    if comp != i1 && comp != i2 {
        if dcomp.abs() > TOL && dcomp < 0. {
            patch.set(index, i1, comp);
        } else if dcomp.abs() > TOL && dcomp > 0. {
            patch.set(index, comp, i2);
        } else {
            patch.set(index, i1, i2);
        }
    } else if (comp - i1).abs() < TOL || (comp - i2).abs() < TOL {
        patch.set(index, i1, i2);
    }
}
