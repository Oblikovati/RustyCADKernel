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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BSplineCurve_1.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Read access: local evaluation, knots, multiplicities, poles and weights.
use super::Geom2dBSplineCurve;
use crate::geom2d::arrays::{at, check_one_based, copy_same_length, put};
use crate::geom2d::errors;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::BSplKnotDistribution;
use rustycad_tkmath::gp::{Point2, Vector2};

// The deprecated `Knots(K)` / `KnotSequence(K)`: K's bounds must lie inside the source's.
fn copy_inside(
    source: &Array1<'_, f64>,
    target: &mut Array1<'_, f64>,
    member: &str,
) -> Result<(), Failure> {
    if target.lower() < source.lower() || target.upper() > source.upper() {
        return Err(errors::domain(&format!(
            "Geom2d_BSplineCurve::{member}: bounds [{}, {}]; expected inside [{}, {}]",
            target.lower(),
            target.upper(),
            source.lower(),
            source.upper()
        )));
    }
    for index in target.lower()..=target.upper() {
        put(target, index, at(source, index)?)?;
    }
    Ok(())
}

impl Geom2dBSplineCurve {
    /// Point located between knots `from_k1` and `to_k2` (`LocalValue`); `curve.local_value(0.5, 1, 2)?`.
    pub fn local_value(&self, u: f64, from_k1: i32, to_k2: i32) -> Result<Point2, Failure> {
        self.local_d0(u, from_k1, to_k2)
    }
    /// `LocalD0`; `curve.local_d0(0.5, 1, 2)?`.
    pub fn local_d0(&self, u: f64, from_k1: i32, to_k2: i32) -> Result<Point2, Failure> {
        self.state.read().local_d0(u, from_k1, to_k2)
    }
    /// `LocalD1`; `let (p, v1) = curve.local_d1(0.5, 1, 2)?;`.
    pub fn local_d1(&self, u: f64, from_k1: i32, to_k2: i32) -> Result<(Point2, Vector2), Failure> {
        self.state.read().local_d1(u, from_k1, to_k2)
    }
    /// `LocalD2`; `let (p, v1, v2) = curve.local_d2(0.5, 1, 2)?;`.
    pub fn local_d2(
        &self,
        u: f64,
        from_k1: i32,
        to_k2: i32,
    ) -> Result<(Point2, Vector2, Vector2), Failure> {
        self.state.read().local_d2(u, from_k1, to_k2)
    }
    /// `LocalD3`; `let (p, v1, v2, v3) = curve.local_d3(0.5, 1, 2)?;`.
    pub fn local_d3(
        &self,
        u: f64,
        from_k1: i32,
        to_k2: i32,
    ) -> Result<(Point2, Vector2, Vector2, Vector2), Failure> {
        self.state.read().local_d3(u, from_k1, to_k2)
    }
    /// `LocalDN`; `curve.local_dn(0.5, 1, 2, 2)?`.
    pub fn local_dn(&self, u: f64, from_k1: i32, to_k2: i32, n: i32) -> Result<Vector2, Failure> {
        self.state.read().local_dn(u, from_k1, to_k2, n)
    }
    /// Whether the weights differ (`IsRational`); `curve.is_rational()`.
    pub fn is_rational(&self) -> bool {
        self.state.read().rational
    }
    /// The degree (`Degree`); `curve.degree()`.
    pub fn degree(&self) -> i32 {
        self.state.read().deg
    }
    /// Index of the first knot of the parameter range (`FirstUKnotIndex`); `curve.first_u_knot_index()?`.
    pub fn first_u_knot_index(&self) -> Result<i32, Failure> {
        self.state.read().first_u_knot_index()
    }
    /// Index of the last knot of the parameter range (`LastUKnotIndex`); `curve.last_u_knot_index()?`.
    pub fn last_u_knot_index(&self) -> Result<i32, Failure> {
        self.state.read().last_u_knot_index()
    }
    /// Knot `index` (`Knot`); `curve.knot(1)?`.
    pub fn knot(&self, index: i32) -> Result<f64, Failure> {
        let state = self.state.read();
        check_one_based(index, state.knots.len(), "Geom2d_BSplineCurve::Knot")?;
        at(&state.knots, index)
    }
    /// Copy the knots of `target`'s bounds (deprecated `Knots(K)`); `curve.knots_into(&mut k)?`.
    pub fn knots_into(&self, target: &mut Array1<'_, f64>) -> Result<(), Failure> {
        copy_inside(&self.state.read().knots, target, "Knots")
    }
    /// The knots, indexed from 1 (`Knots()`); `curve.knots()`.
    pub fn knots(&self) -> Array1<'static, f64> {
        self.state.read().knots.clone()
    }
    /// Copy the flat knots of `target`'s bounds (deprecated `KnotSequence(K)`);
    /// `curve.knot_sequence_into(&mut k)?`.
    pub fn knot_sequence_into(&self, target: &mut Array1<'_, f64>) -> Result<(), Failure> {
        copy_inside(&self.state.read().flat_knots, target, "KnotSequence")
    }
    /// The knots repeated by multiplicity (`KnotSequence()`); `curve.knot_sequence()`.
    pub fn knot_sequence(&self) -> Array1<'static, f64> {
        self.state.read().flat_knots.clone()
    }
    /// The knot distribution (`KnotDistribution`); `curve.knot_distribution()`.
    pub fn knot_distribution(&self) -> BSplKnotDistribution {
        self.state.read().knot_set
    }
    /// The knot interval `(I1, I2)` of `u`; equal indices when `u` is a knot within the
    /// tolerance, 0 or NbKnots+1 outside (`LocateU`); `curve.locate_u(0.5, 1e-9, false)?`.
    pub fn locate_u(
        &self,
        u: f64,
        parametric_tolerance: f64,
        with_knot_repetition: bool,
    ) -> Result<(i32, i32), Failure> {
        self.state
            .read()
            .locate_u(u, parametric_tolerance, with_knot_repetition)
    }
    /// Multiplicity of knot `index` (`Multiplicity`); `curve.multiplicity(1)?`.
    pub fn multiplicity(&self, index: i32) -> Result<i32, Failure> {
        let state = self.state.read();
        check_one_based(
            index,
            state.mults.len(),
            "Geom2d_BSplineCurve::Multiplicity",
        )?;
        at(&state.mults, index)
    }
    /// Copy the multiplicities into `target` of equal length (deprecated `Multiplicities(M)`);
    /// `curve.multiplicities_into(&mut m)?`.
    pub fn multiplicities_into(&self, target: &mut Array1<'_, i32>) -> Result<(), Failure> {
        copy_same_length(
            &self.state.read().mults,
            target,
            "Geom2d_BSplineCurve::Multiplicities",
        )
    }
    /// The multiplicities, indexed from 1 (`Multiplicities()`); `curve.multiplicities()`.
    pub fn multiplicities(&self) -> Array1<'static, i32> {
        self.state.read().mults.clone()
    }
    /// Number of knots (`NbKnots`); `curve.nb_knots()`.
    pub fn nb_knots(&self) -> i32 {
        self.state.read().knots.len() as i32
    }
    /// Number of poles (`NbPoles`); `curve.nb_poles()`.
    pub fn nb_poles(&self) -> i32 {
        self.state.read().poles.len() as i32
    }
    /// Pole `index` (`Pole`); `curve.pole(1)?`.
    pub fn pole(&self, index: i32) -> Result<Point2, Failure> {
        let state = self.state.read();
        check_one_based(index, state.poles.len(), "Geom2d_BSplineCurve::Pole")?;
        at(&state.poles, index)
    }
    /// Copy the poles into `target` of equal length (deprecated `Poles(P)`); `curve.poles_into(&mut p)?`.
    pub fn poles_into(&self, target: &mut Array1<'_, Point2>) -> Result<(), Failure> {
        copy_same_length(
            &self.state.read().poles,
            target,
            "Geom2d_BSplineCurve::Poles",
        )
    }
    /// The poles, indexed from 1 (`Poles()`); `curve.poles()`.
    pub fn poles(&self) -> Array1<'static, Point2> {
        self.state.read().poles.clone()
    }
    /// Weight `index`; 1 for a non-rational curve (`Weight`); `curve.weight(1)?`.
    pub fn weight(&self, index: i32) -> Result<f64, Failure> {
        let state = self.state.read();
        check_one_based(index, state.poles.len(), "Geom2d_BSplineCurve::Weight")?;
        if !state.rational {
            return Ok(1.);
        }
        at(&state.weights, index)
    }
    /// Copy the weights into `target` of equal length, all 1 for a non-rational curve
    /// (deprecated `Weights(W)`); `curve.weights_into(&mut w)?`.
    pub fn weights_into(&self, target: &mut Array1<'_, f64>) -> Result<(), Failure> {
        let state = self.state.read();
        if target.len() != state.poles.len() {
            return Err(errors::dimension(&format!(
                "Geom2d_BSplineCurve::Weights: length {}; expected {}",
                target.len(),
                state.poles.len()
            )));
        }
        if state.rational {
            target
                .as_mut_slice()
                .copy_from_slice(state.weights.as_slice());
            return Ok(());
        }
        target.init(1.);
        Ok(())
    }
    /// The weights of a rational curve, `None` otherwise (`Weights()`); `curve.weights()`.
    pub fn weights(&self) -> Option<Array1<'static, f64>> {
        self.state.read().weights_view().cloned()
    }
    /// Read the stored weights in place, unit weights (a non-owning shared view) for a
    /// non-rational curve (`WeightsArray`, which natively returns a stable reference);
    /// `curve.weights_array(|weights| weights.len())`.
    pub fn weights_array<R>(&self, read: impl FnOnce(&Array1<'static, f64>) -> R) -> R {
        read(&self.state.read().weights)
    }
}
