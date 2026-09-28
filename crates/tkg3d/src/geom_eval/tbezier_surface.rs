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
// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_TBezierSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomEval_TBezierSurface`: a tensor-product trigonometric Bezier surface over
//! `{1, sin(a t), cos(a t), ..., sin(n a t), cos(n a t)}` in U and V, on
//! `[0, Pi/alphaU] x [0, Pi/alphaV]`.
mod evaluation;
mod surface_traits;
use super::bezier_grid::{PoleGrid, check_weight_grid, owned_grid};
use crate::geom::errors;
use evaluation::TBezierSurfaceView;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, StandardTransient};
use rustycad_tkmath::gp::Point3;

const CLASS: &str = "GeomEval_TBezierSurface";

/// A rational or non-rational T-Bezier surface with `(2nU + 1) x (2nV + 1)` poles;
/// immutable like every eval surface (reversal, iso extraction and `Transform` raise
/// Standard_NotImplemented); `Handle::new(GeomEvalTBezierSurface::new(&poles, 1.0, 1.0)?)`.
pub struct GeomEvalTBezierSurface {
    parent: StandardTransient,
    poles: Array2<'static, Point3>,
    weights: Array2<'static, f64>,
    alpha_u: f64,
    alpha_v: f64,
    rational: bool,
}
impl_transient!(
    GeomEvalTBezierSurface,
    "GeomEval_TBezierSurface",
    dyn crate::geom::GeomBoundedSurface,
    parent,
    interfaces: [
        dyn crate::geom::GeomGeometry,
        dyn crate::geom::GeomSurface,
        dyn crate::geom::GeomBoundedSurface
    ]
);
impl Clone for GeomEvalTBezierSurface {
    fn clone(&self) -> Self {
        self.copied()
    }
}

// `validateDimension`: an odd pole count >= 3 in one direction.
fn check_dimension(size: usize, direction: &str) -> Result<(), Failure> {
    if size < 3 || size.is_multiple_of(2) {
        return Err(errors::construction(&format!(
            "{CLASS}: {size} {direction} poles; expected an odd {direction} pole count >= 3"
        )));
    }
    Ok(())
}

fn check_alpha(alpha: f64, name: &str) -> Result<(), Failure> {
    if alpha <= 0.0 {
        return Err(errors::construction(&format!(
            "{CLASS}: {name} = {alpha}; expected > 0"
        )));
    }
    Ok(())
}

// The checks both constructors share, in native order.
fn check_poles_and_alphas(poles: &Array2<'_, Point3>, alphas: [f64; 2]) -> Result<(), Failure> {
    check_dimension(poles.nb_rows(), "U")?;
    check_dimension(poles.nb_columns(), "V")?;
    check_alpha(alphas[0], "alphaU")?;
    check_alpha(alphas[1], "alphaV")
}

impl GeomEvalTBezierSurface {
    /// Non-rational surface (`GeomEval_TBezierSurface(Poles, AlphaU, AlphaV)`);
    /// Standard_ConstructionError unless both pole counts are odd and >= 3 and both
    /// alphas are > 0; `GeomEvalTBezierSurface::new(&poles, 1.0, 1.0)?`.
    pub fn new(poles: &Array2<'_, Point3>, alpha_u: f64, alpha_v: f64) -> Result<Self, Failure> {
        check_poles_and_alphas(poles, [alpha_u, alpha_v])?;
        Ok(Self::stored(
            poles,
            Array2::default(),
            [alpha_u, alpha_v],
            false,
        ))
    }
    /// Rational surface (`GeomEval_TBezierSurface(Poles, Weights, AlphaU, AlphaV)`);
    /// also refuses a weight grid of another shape and weights <= 0. Equal weights
    /// still give a rational surface;
    /// `GeomEvalTBezierSurface::with_weights(&poles, &weights, 1.0, 1.0)?`.
    pub fn with_weights(
        poles: &Array2<'_, Point3>,
        weights: &Array2<'_, f64>,
        alpha_u: f64,
        alpha_v: f64,
    ) -> Result<Self, Failure> {
        check_poles_and_alphas(poles, [alpha_u, alpha_v])?;
        check_weight_grid(CLASS, weights, poles)?;
        let stored_weights = owned_grid(weights);
        Ok(Self::stored(
            poles,
            stored_weights,
            [alpha_u, alpha_v],
            true,
        ))
    }
    fn stored(
        poles: &Array2<'_, Point3>,
        weights: Array2<'static, f64>,
        [alpha_u, alpha_v]: [f64; 2],
        rational: bool,
    ) -> Self {
        Self {
            parent: StandardTransient::default(),
            poles: owned_grid(poles),
            weights,
            alpha_u,
            alpha_v,
            rational,
        }
    }
    // Copy() reruns the matching constructor on valid stored values.
    fn copied(&self) -> Self {
        let alphas: [f64; 2] = [self.alpha_u, self.alpha_v];
        Self::stored(&self.poles, self.weights.clone(), alphas, self.rational)
    }
    /// A copy of the poles grid with its bounds (`Poles`); `surface.poles().nb_rows()`.
    pub fn poles(&self) -> Array2<'static, Point3> {
        self.poles.clone()
    }
    /// A copy of the weights grid, empty for a non-rational surface (`Weights`);
    /// `surface.weights().len()`.
    pub fn weights(&self) -> Array2<'static, f64> {
        self.weights.clone()
    }
    /// The frequency in U (`AlphaU`); `surface.alpha_u()`.
    pub fn alpha_u(&self) -> f64 {
        self.alpha_u
    }
    /// The frequency in V (`AlphaV`); `surface.alpha_v()`.
    pub fn alpha_v(&self) -> f64 {
        self.alpha_v
    }
    /// The number of poles in U, the grid rows (`NbUPoles`); `surface.nb_u_poles()`.
    pub fn nb_u_poles(&self) -> i32 {
        self.poles.nb_rows() as i32
    }
    /// The number of poles in V, the grid columns (`NbVPoles`); `surface.nb_v_poles()`.
    pub fn nb_v_poles(&self) -> i32 {
        self.poles.nb_columns() as i32
    }
    /// The trigonometric order in U, NbUPoles = 2 nU + 1 (`OrderU`); `surface.order_u()`.
    pub fn order_u(&self) -> i32 {
        (self.nb_u_poles() - 1) / 2
    }
    /// The trigonometric order in V, NbVPoles = 2 nV + 1 (`OrderV`); `surface.order_v()`.
    pub fn order_v(&self) -> i32 {
        (self.nb_v_poles() - 1) / 2
    }
    /// Whether the surface was built with weights (`IsRational`); `surface.is_rational()`.
    pub fn is_rational(&self) -> bool {
        self.rational
    }
    fn view(&self) -> TBezierSurfaceView<'_> {
        TBezierSurfaceView {
            grid: PoleGrid {
                poles: self.poles.as_slice(),
                weights: self.rational.then(|| self.weights.as_slice()),
                rows: self.poles.nb_rows(),
                cols: self.poles.nb_columns(),
            },
            alpha_u: self.alpha_u,
            alpha_v: self.alpha_v,
        }
    }
    // Whether the first and last pole rows (`u`) or columns coincide within
    // Precision::Confusion (`IsUClosed`, `IsVClosed`).
    fn boundary_lines_coincide(&self, u: bool) -> bool {
        let (rows, cols) = (self.poles.nb_rows(), self.poles.nb_columns());
        let poles: &[Point3] = self.poles.as_slice();
        let index = |k: usize, last: bool| -> usize {
            match (u, last) {
                (true, false) => k,
                (true, true) => (rows - 1) * cols + k,
                (false, false) => k * cols,
                (false, true) => k * cols + cols - 1,
            }
        };
        let count: usize = if u { cols } else { rows };
        // Only `d > tol` opens the surface, as natively: a NaN distance does not.
        for k in 0..count {
            let distance: f64 = poles[index(k, false)].distance(&poles[index(k, true)]);
            if distance > precision::confusion() {
                return false;
            }
        }
        true
    }
}
