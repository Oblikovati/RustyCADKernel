// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_CutCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MAT2d_CutCurve`: a curve cut at its extrema of curvature and its inflections.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve, errors};
use rustycad_tkgeombase::geom_lprop::CurAndInf2d;
use rustycad_tkmath::gp::Point2;

/// `MAT2d_CutCurve`: the pieces of a curve between its extrema of curvature and its
/// inflections (none when the curve is not cut). The overloads `Perform(C, aSide)`
/// and `PerformInf(C)` are declared but never defined natively.
/// ```
/// use rustycad_tktopalgo::mat2d::Mat2dCutCurve;
/// let cut = Mat2dCutCurve::new();
/// assert!(cut.un_modified() && cut.nb_curves().is_err());
/// ```
#[derive(Clone, Default)]
pub struct Mat2dCutCurve {
    the_curves: Vec<Handle<Geom2dTrimmedCurve>>,
}

impl Mat2dCutCurve {
    /// `MAT2d_CutCurve()`: no pieces; `Mat2dCutCurve::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `MAT2d_CutCurve(C)`: [`Mat2dCutCurve::perform`] on a new object;
    /// `Mat2dCutCurve::with_curve(&curve)?`.
    pub fn with_curve(c: &Handle<dyn Geom2dCurve>) -> Result<Self, Failure> {
        let mut cut: Self = Self::new();
        cut.perform(c)?;
        Ok(cut)
    }
    /// `Perform(C)`: cuts the curve at the points of `GeomLProp_CurAndInf2d` that are
    /// not at its ends (by parameter and by point); `cut.perform(&curve)?`.
    pub fn perform(&mut self, c: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        self.the_curves.clear();
        let mut sommets = CurAndInf2d::new();
        sommets.perform(c)?;
        if !sommets.is_done() || sommets.is_empty() {
            return Ok(());
        }
        let (mut uf, ul) = (c.first_parameter(), c.last_parameter());
        let (mut pf, pl) = (c.value(uf)?, c.value(ul)?);
        let p_tol: f64 = precision::p_confusion() * 10.;
        let tol: f64 = precision::confusion() * 10.;
        let mut ya_cut: bool = false;
        for i in 1..=sommets.nb_points() {
            let uc: f64 = sommets.parameter(i)?;
            let pc: Point2 = c.value(uc)?;
            if !(uc - uf > p_tol && pc.distance(pf) > tol) {
                continue;
            }
            if ul - uc < p_tol || pl.distance(pc) < tol {
                break;
            }
            self.the_curves
                .push(Handle::new(Geom2dTrimmedCurve::new(c, uf, uc, true, true)?));
            (uf, pf, ya_cut) = (uc, pc, true);
        }
        if ya_cut {
            self.the_curves
                .push(Handle::new(Geom2dTrimmedCurve::new(c, uf, ul, true, true)?));
        }
        Ok(())
    }
    /// `UnModified()`: true when the curve is not cut; `cut.un_modified()`.
    pub fn un_modified(&self) -> bool {
        self.the_curves.is_empty()
    }
    /// `NbCurves()`: the number of pieces (at least 2); Standard_OutOfRange when the
    /// curve is not cut; `cut.nb_curves()?`.
    pub fn nb_curves(&self) -> Result<i32, Failure> {
        if self.un_modified() {
            return Err(errors::out_of_range(
                "MAT2d_CutCurve::NbCurves: the curve is not cut; expected UnModified() false",
            ));
        }
        Ok(self.the_curves.len() as i32)
    }
    /// `Value(Index)`: the piece `index`; Standard_OutOfRange when the curve is not cut
    /// or outside `1..=NbCurves()`; `cut.value(1)?`.
    pub fn value(&self, index: i32) -> Result<Handle<Geom2dTrimmedCurve>, Failure> {
        let count: i32 = self.nb_curves()?;
        if index < 1 || index > count {
            return Err(errors::out_of_range(&format!(
                "MAT2d_CutCurve::Value: index {index}; expected 1..={count}"
            )));
        }
        Ok(self.the_curves[(index - 1) as usize].clone())
    }
}
