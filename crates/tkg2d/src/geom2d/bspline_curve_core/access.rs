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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BSplineCurve_1.cxx and ModelingData/TKG3d/Geom/Geom_BSplineCurve_1.cxx (read access) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Checked read access to knots, multiplicities, poles and weights.
use super::{BSplineCurveCore, BSplineCurveKind};
use crate::geom2d::arrays::{at, check_one_based, copy_same_length};
use crate::geom2d::errors;
use crate::geom2d::spline_data::copy_inside;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

impl<K: BSplineCurveKind> BSplineCurveCore<K> {
    // `Class::Member`, the name the index and length failures carry.
    fn member(member: &str) -> String {
        format!("{}::{member}", K::BSPLINE_CLASS)
    }
    /// `Knot(Index)`; `core.knot(1)?`.
    pub fn knot(&self, index: i32) -> Result<f64, Failure> {
        check_one_based(index, self.knots.len(), &Self::member("Knot"))?;
        at(&self.knots, index)
    }
    /// Deprecated `Knots(K)`: the knots of `target`'s bounds; `core.knots_into(&mut k)?`.
    pub fn knots_into(&self, target: &mut Array1<'_, f64>) -> Result<(), Failure> {
        copy_inside(&self.knots, target, &Self::member("Knots"))
    }
    /// Deprecated `KnotSequence(K)`: the flat knots of `target`'s bounds;
    /// `core.knot_sequence_into(&mut k)?`.
    pub fn knot_sequence_into(&self, target: &mut Array1<'_, f64>) -> Result<(), Failure> {
        copy_inside(&self.flat_knots, target, &Self::member("KnotSequence"))
    }
    /// `Multiplicity(Index)`; `core.multiplicity(1)?`.
    pub fn multiplicity(&self, index: i32) -> Result<i32, Failure> {
        check_one_based(index, self.mults.len(), &Self::member("Multiplicity"))?;
        at(&self.mults, index)
    }
    /// Deprecated `Multiplicities(M)` into an array of equal length;
    /// `core.multiplicities_into(&mut m)?`.
    pub fn multiplicities_into(&self, target: &mut Array1<'_, i32>) -> Result<(), Failure> {
        copy_same_length(&self.mults, target, &Self::member("Multiplicities"))
    }
    /// `Pole(Index)`; `core.pole(1)?`.
    pub fn pole(&self, index: i32) -> Result<K::Point, Failure> {
        check_one_based(index, self.poles.len(), &Self::member("Pole"))?;
        at(&self.poles, index)
    }
    /// Deprecated `Poles(P)` into an array of equal length; `core.poles_into(&mut p)?`.
    pub fn poles_into(&self, target: &mut Array1<'_, K::Point>) -> Result<(), Failure> {
        copy_same_length(&self.poles, target, &Self::member("Poles"))
    }
    /// `Weight(Index)`: 1 for a non-rational curve; `core.weight(1)?`.
    pub fn weight(&self, index: i32) -> Result<f64, Failure> {
        check_one_based(index, self.poles.len(), &Self::member("Weight"))?;
        if !self.rational {
            return Ok(1.);
        }
        at(&self.weights, index)
    }
    /// Deprecated `Weights(W)` into an array of equal length, all 1 for a non-rational
    /// curve; `core.weights_into(&mut w)?`.
    pub fn weights_into(&self, target: &mut Array1<'_, f64>) -> Result<(), Failure> {
        if target.len() != self.poles.len() {
            return Err(errors::dimension(&format!(
                "{}: length {}; expected {}",
                Self::member("Weights"),
                target.len(),
                self.poles.len()
            )));
        }
        if self.rational {
            target
                .as_mut_slice()
                .copy_from_slice(self.weights.as_slice());
            return Ok(());
        }
        target.init(1.);
        Ok(())
    }
}
