// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of the loop and frame code shared by ModelingData/TKG2d/Geom2dGridEval/*.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Grid plumbing shared by the evaluators: 1-based result arrays filled in parameter
//! order, and the pre-extracted conic frame with the native operation order.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction2, Point2, Vector2};

/// An empty result array (`NCollection_Array1<T>()`, bounds `1..0`).
pub(super) fn empty<T>() -> Array1<'static, T> {
    Array1::default()
}

/// A default-filled result array of bounds `1..len` (`NCollection_Array1<T>(1, len)`).
pub(super) fn result_array<T: Default>(len: usize) -> Array1<'static, T> {
    let mut array: Array1<'static, T> = Array1::with_size(len);
    array.update_lower_bound(1);
    array
}

/// Evaluates `eval` at each parameter from `Lower()` to `Upper()` into slot
/// `i - Lower() + 1`; the caller has already rejected an empty parameter array.
pub(super) fn map_grid<T: Default>(
    params: &Array1<'_, f64>,
    mut eval: impl FnMut(f64) -> T,
) -> Array1<'static, T> {
    let mut result: Array1<'static, T> = result_array(params.len());
    for (slot, &u) in result.iter_mut().zip(params.iter()) {
        *slot = eval(u);
    }
    result
}

/// `map_grid` for evaluators that can raise: the first failure aborts the grid, as
/// the native exception leaves the loop.
pub(super) fn try_map_grid<T: Default>(
    params: &Array1<'_, f64>,
    mut eval: impl FnMut(f64) -> Result<T, Failure>,
) -> Result<Array1<'static, T>, Failure> {
    let mut result: Array1<'static, T> = result_array(params.len());
    for (slot, &u) in result.iter_mut().zip(params.iter()) {
        *slot = eval(u)?;
    }
    Ok(result)
}

/// Fills every slot of a `1..len` array with the same value (the constant-derivative
/// loops `for (i = 1; i <= aNb; ++i) aResult.SetValue(i, aConst)`).
pub(super) fn constant_grid<T: Default + Copy>(len: usize, value: T) -> Array1<'static, T> {
    let mut result: Array1<'static, T> = result_array(len);
    result.init(value);
    result
}

/// One derivative of every record of a grid, in order (the `aResult.SetValue(i,
/// aGrid.Value(i).DK)` loops of the offset and fallback DN).
pub(super) fn column<R>(
    grid: &Array1<'_, R>,
    pick: impl Fn(&R) -> Vector2,
) -> Array1<'static, Vector2> {
    let mut result: Array1<'static, Vector2> = result_array(grid.len());
    for (slot, record) in result.iter_mut().zip(grid.iter()) {
        *slot = pick(record);
    }
    result
}

/// Center and axis coordinates that the conic evaluators pre-extract
/// (`aCX, aCY, aXX, aXY, aYX, aYY`).
#[derive(Clone, Copy)]
pub(super) struct GridFrame {
    cx: f64,
    cy: f64,
    xx: f64,
    xy: f64,
    yx: f64,
    yy: f64,
}

impl GridFrame {
    /// Frame of a center and its X and Y directions.
    pub(super) fn new(center: Point2, x_dir: Direction2, y_dir: Direction2) -> Self {
        Self {
            cx: center.x(),
            cy: center.y(),
            xx: x_dir.x(),
            xy: x_dir.y(),
            yx: y_dir.x(),
            yy: y_dir.y(),
        }
    }
    /// `a * X + b * Y`, evaluated as `(a * aXX + b * aYX, a * aXY + b * aYY)`.
    pub(super) fn combined(&self, a: f64, b: f64) -> Vector2 {
        Vector2::new(a * self.xx + b * self.yx, a * self.xy + b * self.yy)
    }
    /// `a * X`, as `(a * aXX, a * aXY)`.
    pub(super) fn scaled_x(&self, a: f64) -> Vector2 {
        Vector2::new(a * self.xx, a * self.xy)
    }
    /// `C + a * X + b * Y`, evaluated left to right as `aCX + a * aXX + b * aYX`.
    pub(super) fn offset_point(&self, a: f64, b: f64) -> Point2 {
        Point2::new(
            self.cx + a * self.xx + b * self.yx,
            self.cy + a * self.xy + b * self.yy,
        )
    }
    /// `C + v`, for the circle, whose formula scales the whole combination by `R`.
    pub(super) fn translated(&self, v: Vector2) -> Point2 {
        Point2::new(self.cx + v.x(), self.cy + v.y())
    }
}

/// A frame with major and minor radii: the ellipse and hyperbola terms
/// `C + MajR * a * X + MinR * b * Y` and `MajR * a * X + MinR * b * Y`.
#[derive(Clone, Copy)]
pub(super) struct RadiiFrame {
    pub(super) frame: GridFrame,
    pub(super) major: f64,
    pub(super) minor: f64,
}

impl RadiiFrame {
    /// `aCX + aMajR * a * aXX + aMinR * b * aYX` (and Y).
    pub(super) fn point(&self, a: f64, b: f64) -> Point2 {
        self.frame.offset_point(self.major * a, self.minor * b)
    }
    /// `aMajR * a * aXX + aMinR * b * aYX` (and Y); a negated coefficient gives the
    /// same bits as the native `-aMajR * sinU` spelling.
    pub(super) fn vector(&self, a: f64, b: f64) -> Vector2 {
        self.frame.combined(self.major * a, self.minor * b)
    }
}

/// The derivative coefficients of `(cos, sin)` for derivative order `n >= 1`, cyclic
/// with period 4 (the `switch ((theN - 1) % 4)` of the circle and ellipse DN).
pub(super) fn cyclic_coefficients(n: i32, cos_u: f64, sin_u: f64) -> (f64, f64) {
    match (n - 1) % 4 {
        0 => (-sin_u, cos_u),
        1 => (-cos_u, -sin_u),
        2 => (sin_u, -cos_u),
        _ => (cos_u, sin_u),
    }
}
