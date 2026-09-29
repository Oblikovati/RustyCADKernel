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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_TrimmedCurve.cxx and ModelingData/TKG3d/Geom/Geom_TrimmedCurve.cxx (SetTrim, Reverse and the whole-period test that both classes repeat) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The trim bounds of `Geom2d_TrimmedCurve` and `Geom_TrimmedCurve`: both classes
//! store a basis copy and two parameters, and run the same SetTrim and Reverse.
use super::errors;
use super::native::std_min;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkmath::elclib;

/// The basis curve queries that SetTrim and Reverse make; implemented for
/// `dyn Geom2dCurve` and `dyn GeomCurve`; `basis.trim_basis_range()`.
pub trait TrimBasis: RuntimeTyped {
    /// `(FirstParameter, LastParameter)`.
    fn trim_basis_range(&self) -> (f64, f64);
    /// `IsPeriodic`.
    fn trim_basis_periodic(&self) -> bool;
    /// `Period`.
    fn trim_basis_period(&self) -> Result<f64, Failure>;
    /// `ReversedParameter(U)`.
    fn trim_basis_reversed_parameter(&self, u: f64) -> Result<f64, Failure>;
    /// `Reverse`.
    fn trim_basis_reverse(&self) -> Result<(), Failure>;
}

/// The fields of a trimmed curve (`basisCurve`, `uTrim1`, `uTrim2`);
/// `TrimmedBounds { basis, u_trim1: 0.0, u_trim2: 1.0 }`.
pub struct TrimmedBounds<B: ?Sized + TrimBasis> {
    /// The private basis copy.
    pub basis: Handle<B>,
    /// The first trim parameter.
    pub u_trim1: f64,
    /// The second trim parameter.
    pub u_trim2: f64,
}

impl<B: ?Sized + TrimBasis> TrimmedBounds<B> {
    /// `Reverse`: reverse the basis and trim it to the reversed bounds;
    /// `bounds.reverse("Geom2d_TrimmedCurve")?`.
    pub fn reverse(&mut self, class: &str) -> Result<(), Failure> {
        let u1: f64 = self.basis.trim_basis_reversed_parameter(self.u_trim2)?;
        let u2: f64 = self.basis.trim_basis_reversed_parameter(self.u_trim1)?;
        self.basis.trim_basis_reverse()?;
        self.set_trim(u1, u2, true, false, class)
    }
    /// `SetTrim`: Standard_ConstructionError naming `class` when `u1 == u2`; like the
    /// native code, the out of range failure is raised after the new bounds are
    /// stored; `bounds.set_trim(0.0, 1.0, true, true, "Geom_TrimmedCurve")?`.
    pub fn set_trim(
        &mut self,
        u1: f64,
        u2: f64,
        sense: bool,
        adjust_periodic: bool,
        class: &str,
    ) -> Result<(), Failure> {
        if u1 == u2 {
            return Err(errors::construction(&format!(
                "{class}::U1 == U2 = {u1}; expected distinct parameters"
            )));
        }
        let (udeb, ufin) = self.basis.trim_basis_range();
        let same_sense: bool = if self.basis.trim_basis_periodic() {
            self.trim_periodic([u1, u2], [udeb, ufin], adjust_periodic);
            sense
        } else {
            self.trim_bounded(u1, u2, [udeb, ufin])?;
            if u1 < u2 { sense } else { !sense }
        };
        if !same_sense {
            self.reverse(class)?;
        }
        Ok(())
    }
    fn trim_periodic(&mut self, trim: [f64; 2], range: [f64; 2], adjust: bool) {
        self.u_trim1 = trim[0];
        self.u_trim2 = trim[1];
        if !adjust {
            return;
        }
        let tolerance: f64 = std_min(
            (self.u_trim2 - self.u_trim1).abs() / 2.0,
            precision::p_confusion(),
        );
        (self.u_trim1, self.u_trim2) =
            elclib::adjust_periodic(range[0], range[1], tolerance, self.u_trim1, self.u_trim2);
    }
    fn trim_bounded(&mut self, u1: f64, u2: f64, range: [f64; 2]) -> Result<(), Failure> {
        (self.u_trim1, self.u_trim2) = if u1 < u2 { (u1, u2) } else { (u2, u1) };
        if (range[0] - self.u_trim1 > precision::p_confusion())
            || (self.u_trim2 - range[1] > precision::p_confusion())
        {
            // Both classes' messages name Geom_TrimmedCurve.
            return Err(errors::construction(&format!(
                "Geom_TrimmedCurve::parameters out of range: [{}, {}]; expected within [{}, {}]",
                self.u_trim1, self.u_trim2, range[0], range[1]
            )));
        }
        Ok(())
    }
    /// Whether the trimmed length is a whole number of basis periods (the shared
    /// test of IsClosed and IsPeriodic); `bounds.spans_whole_periods()?`.
    pub fn spans_whole_periods(&self) -> Result<bool, Failure> {
        if !self.basis.trim_basis_periodic() {
            return Ok(false);
        }
        let period: f64 = self.basis.trim_basis_period()?;
        let length: f64 = self.u_trim2 - self.u_trim1;
        Ok(length > precision::p_confusion()
            && rustycad_cmath::remainder(length, period).abs() <= precision::p_confusion())
    }
}
