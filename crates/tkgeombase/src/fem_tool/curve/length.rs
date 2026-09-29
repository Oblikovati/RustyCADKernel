// Created on: 1997-11-04
// Created by: Roman BORISOV / Igor FEOKTISTOV
// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/FEmTool/FEmTool_Curve.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FEmTool_Curve::Length`: the arc length between two parameters, with the
//! whole-element lengths cached.
use super::Curve;
use super::state::{CurveState, slot};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::plib;

impl Curve {
    /// `Length(FirstU, LastU, Length)`: Standard_OutOfRange when `first_u > last_u`.
    /// Parameters outside the knots extend the first or last element polynomial;
    /// `curve.length(0.0, 1.0)?`.
    pub fn length(&self, first_u: f64, last_u: f64) -> Result<f64, Failure> {
        if first_u > last_u {
            return Err(Failure::out_of_range(&format!(
                "FEmTool_Curve::Length: FirstU {first_u} > LastU {last_u}"
            )));
        }
        let low: i32 = self.low_element(first_u)?;
        let high: i32 = self.high_element(last_u, low)?;
        let first_s: f64 = self.normalized(first_u, low)?;
        let last_s: f64 = self.normalized(last_u, high)?;
        let mut state = self.state.write();
        if low == high {
            return self.element_length(&mut state, low, first_s, last_s);
        }
        let mut length: f64 = 0.;
        if first_s < -1. {
            length += self.element_length(&mut state, low, first_s, -1.)?;
            length += self.whole_length(&mut state, low)?;
        } else {
            length += self.element_length(&mut state, low, first_s, 1.)?;
        }
        if last_s > 1. {
            length += self.element_length(&mut state, high, 1., last_s)?;
            length += self.whole_length(&mut state, high)?;
        } else {
            length += self.element_length(&mut state, high, -1., last_s)?;
        }
        for index in low + 1..high {
            length += self.whole_length(&mut state, index)?;
        }
        Ok(length)
    }

    // Low: element 1 before the first knot, else the first element containing
    // `first_u`, clamped to NbElements.
    fn low_element(&self, first_u: f64) -> Result<i32, Failure> {
        if self.knot(1)? > first_u {
            return Ok(1);
        }
        let mut low: i32 = 1;
        while low <= self.nb_elements
            && !(first_u >= self.knot(low)? && first_u <= self.knot(low + 1)?)
        {
            low += 1;
        }
        Ok(low.min(self.nb_elements))
    }

    // High: element 1 before the first knot, else the first element from `low`
    // containing `last_u`; NbElements after the last knot.
    fn high_element(&self, last_u: f64, low: i32) -> Result<i32, Failure> {
        let mut high: i32 = 1;
        if self.knot(1)? <= last_u {
            high = low;
            while high <= self.nb_elements
                && !(last_u >= self.knot(high)? && last_u <= self.knot(high + 1)?)
            {
                high += 1;
            }
        }
        if self.knot(self.nb_elements + 1)? < last_u {
            high = self.nb_elements;
        }
        Ok(high)
    }

    // `(2 u - K(i) - K(i+1)) / (K(i+1) - K(i))`: `u` mapped onto [-1, 1] of element i.
    fn normalized(&self, u: f64, index: i32) -> Result<f64, Failure> {
        let (first, last) = (self.knot(index)?, self.knot(index + 1)?);
        Ok((2. * u - first - last) / (last - first))
    }

    // PLib::EvalLength of element `index` between the normalized parameters.
    fn element_length(
        &self,
        state: &mut CurveState,
        index: i32,
        from: f64,
        to: f64,
    ) -> Result<f64, Failure> {
        let position: usize = slot(&state.degrees, index, "Length")?;
        if !state.has_poly[position] {
            self.update(state, index, 0)?;
        }
        let start: usize = state.element_start(index)?;
        let polynomial: &[f64] = state.poly.get(start..).unwrap_or(&[]);
        plib::eval_length(
            state.degrees[position],
            self.dimension,
            polynomial,
            from,
            to,
        )
    }

    // The cached length of the whole element `index` (computed on [-1, 1] when -1).
    fn whole_length(&self, state: &mut CurveState, index: i32) -> Result<f64, Failure> {
        let position: usize = slot(&state.degrees, index, "Length")?;
        if state.lengths[position] < 0. {
            state.lengths[position] = self.element_length(state, index, -1., 1.)?;
        }
        Ok(state.lengths[position])
    }
}
