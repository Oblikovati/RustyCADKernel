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
// Rust translation of ModelingData/TKGeomBase/FEmTool/FEmTool_Curve.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FEmTool_Curve`: a piecewise polynomial curve whose elements are stored in the
//! Hermite-Jacobi basis, with lazily derived canonical coefficients.
mod elements;
mod eval;
mod length;
mod state;

use crate::native_failure::array_failure;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::plib::HermitJacobi;
use state::CurveState;
use std::sync::RwLockWriteGuard;

/// `FEmTool_Curve`: `NbElements` elements of dimension `Dimension` between the knots
/// `Knots()`; every handle to the curve sees the same state, as natively.
///
/// ```
/// use rustycad_tkgeombase::fem_tool::Curve;
/// use rustycad_tkmath::geom_abs::Shape;
/// use rustycad_tkmath::plib::HermitJacobi;
/// let curve = Curve::new(2, 3, &HermitJacobi::new(5, Shape::C1)?, 1e-6)?;
/// assert_eq!((curve.nb_elements(), curve.dimension(), curve.degree(1)?), (3, 2, 5));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct Curve {
    parent: StandardTransient,
    nb_elements: i32,
    dimension: i32,
    base: HermitJacobi,
    knots: Handle<HArray1<f64>>,
    state: SharedState<CurveState>,
}
impl_transient!(Curve, "FEmTool_Curve", StandardTransient, parent);

impl Curve {
    /// `FEmTool_Curve(Dimension, NbElements, TheBase, Tolerance)`: the tolerance is
    /// unused natively. Every element starts at WorkDegree. The knots and
    /// coefficients are uninitialized natively; they start at 0 here.
    /// `Curve::new(3, 4, &base, 1e-7)?`.
    pub fn new(
        dimension: i32,
        nb_elements: i32,
        base: &HermitJacobi,
        _tolerance: f64,
    ) -> Result<Self, Failure> {
        let knots: HArray1<f64> = HArray1::with_value(1, nb_elements + 1, 0.)
            .map_err(|error| array_failure(error, "FEmTool_Curve"))?;
        Ok(Self {
            parent: StandardTransient::default(),
            nb_elements,
            dimension,
            base: *base,
            knots: Handle::new(knots),
            state: SharedState::new(CurveState::new(dimension, nb_elements, base.work_degree())?),
        })
    }

    /// `Knots()`: the writable knot array `1..=NbElements + 1` (a const member
    /// returning a non-const reference natively). Release the guard before
    /// evaluating the curve; `curve.knots()[1] = 0.0`.
    pub fn knots(&self) -> RwLockWriteGuard<'_, Array1<'static, f64>> {
        self.knots.change_array1()
    }

    /// `NbElements()`; `curve.nb_elements()`.
    pub fn nb_elements(&self) -> i32 {
        self.nb_elements
    }

    /// `Dimension()`; `curve.dimension()`.
    pub fn dimension(&self) -> i32 {
        self.dimension
    }

    /// `Base()`: the Hermite-Jacobi basis; `curve.base().work_degree()`.
    pub fn base(&self) -> &HermitJacobi {
        &self.base
    }

    /// `Degree(IndexOfElement)`: Standard_OutOfRange outside `1..=NbElements` (an
    /// unchecked read natively); `curve.degree(1)?`.
    pub fn degree(&self, index_of_element: i32) -> Result<i32, Failure> {
        self.state.read().degree(index_of_element)
    }

    /// `SetDegree(IndexOfElement, Degree)`: Standard_OutOfRange for a degree above
    /// WorkDegree; `curve.set_degree(1, 3)?`.
    pub fn set_degree(&self, index_of_element: i32, degree: i32) -> Result<(), Failure> {
        if degree > self.base.work_degree() {
            return Err(Failure::out_of_range(&format!(
                "FEmTool_Curve::SetDegree: degree {degree}; expected <= {}",
                self.base.work_degree()
            )));
        }
        let mut state = self.state.write();
        *state.degree_mut(index_of_element)? = degree;
        state.invalidate(index_of_element)
    }

    /// `ReduceDegree(IndexOfElement, Tol, NewDegree, MaxError)`: truncates the
    /// Jacobi part of the element within `tol`, never below `2 * NivConstr + 1`;
    /// returns `(NewDegree, MaxError)`; `curve.reduce_degree(1, 1e-3)?`.
    pub fn reduce_degree(&self, index_of_element: i32, tol: f64) -> Result<(i32, f64), Failure> {
        let mut state = self.state.write();
        let degree: i32 = state.degree(index_of_element)?;
        let start: usize = state.element_start(index_of_element)?;
        let (new_degree, max_error) =
            self.base
                .reduce_degree(self.dimension, degree, tol, state.coeff_from(start)?)?;
        let new_degree: i32 = new_degree.max(2 * self.base.niv_constr() + 1);
        if new_degree < degree {
            *state.degree_mut(index_of_element)? = new_degree;
            state.invalidate(index_of_element)?;
        }
        Ok((new_degree, max_error))
    }

    // Knot `index`, Standard_OutOfRange outside 1..=NbElements+1 (unchecked natively).
    fn knot(&self, index: i32) -> Result<f64, Failure> {
        self.knots
            .array1()
            .value(index)
            .copied()
            .map_err(|error| array_failure(error, "FEmTool_Curve::Knots"))
    }

    // Standard_OutOfRange for an element index outside 1..=NbElements.
    fn check_element(&self, index_of_element: i32) -> Result<(), Failure> {
        if index_of_element > self.nb_elements || index_of_element < 1 {
            return Err(Failure::out_of_range(&format!(
                "FEmTool_Curve: element {index_of_element}; expected 1..={}",
                self.nb_elements
            )));
        }
        Ok(())
    }
}
