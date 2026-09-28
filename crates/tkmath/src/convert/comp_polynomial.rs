// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/Convert/Convert_CompPolynomialToPoles.hxx/.cxx (constructors and accessors) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Piecewise polynomial N-dimensional curves converted to one B-spline curve.
use super::arrays::one_based;
use super::cos_sin::construction;
use crate::plib::access::{array_value, grid_value};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

/// Poles, knots and multiplicities of the B-spline equivalent to polynomial spans
/// with the requested continuity (`Convert_CompPolynomialToPoles`). Coefficients are
/// the C array `[NumCurves][MaxDegree + 1][Dimension]` in canonical form; span `n`
/// is defined on `PolynomialIntervals(n, *)` and mapped linearly onto
/// `[TrueIntervals(n), TrueIntervals(n + 1)]`;
/// `CompPolynomialToPoles::from_single_span(1, 1, 1, &coeffs, &domain, &target)?.degree() == 1`.
pub struct CompPolynomialToPoles {
    pub(super) flat_knots: Array1<'static, f64>,
    pub(super) knots: Array1<'static, f64>,
    pub(super) mults: Array1<'static, i32>,
    pub(super) poles: Array2<'static, f64>,
    pub(super) degree: i32,
    pub(super) done: bool,
}

/// The span tables shared by the constructors: coefficient counts per span,
/// coefficients, polynomial domains (one row of two bounds per span) and the true
/// parameter breakpoints; `PolynomialSpans { num_coeff_per_curve: &n, coefficients: &c, polynomial_intervals: &p, true_intervals: &t }`.
#[derive(Clone, Copy)]
pub struct PolynomialSpans<'r, 'a> {
    /// Coefficient count (degree + 1) of each span.
    pub num_coeff_per_curve: &'r Array1<'a, i32>,
    /// Canonical coefficients `[span][MaxDegree + 1][Dimension]`.
    pub coefficients: &'r Array1<'a, f64>,
    /// Polynomial domain of each span, `[span][2]`.
    pub polynomial_intervals: &'r Array2<'a, f64>,
    /// Breakpoints of the composite parameterisation.
    pub true_intervals: &'r Array1<'a, f64>,
}

fn bad_arguments(detail: String) -> Failure {
    construction(&format!(
        "Convert_CompPolynomialToPoles:bad arguments: {detail}"
    ))
}

fn too_great(continuity: i32, degree: i32) -> Failure {
    construction(&format!(
        "Convert_CompPolynomialToPoles:Continuity is too great: {continuity} > degree {degree}"
    ))
}

impl CompPolynomialToPoles {
    /// Spans joined with one continuity (the handle constructor); a missing table,
    /// a non-positive count or a domain table without two columns is a construction error;
    /// `CompPolynomialToPoles::from_uniform_continuity(2, 0, 1, 1, Some(spans))?`.
    pub fn from_uniform_continuity(
        num_curves: i32,
        continuity: i32,
        dimension: i32,
        max_degree: i32,
        spans: Option<PolynomialSpans<'_, '_>>,
    ) -> Result<Self, Failure> {
        let Some(spans) = spans.filter(|s| s.polynomial_intervals.nb_columns() == 2) else {
            return Err(bad_arguments(
                "missing tables or domains without 2 columns".into(),
            ));
        };
        if num_curves <= 0 || continuity < 0 || max_degree <= 0 || dimension <= 0 {
            return Err(bad_arguments(format!(
                "NumCurves {num_curves}, Continuity {continuity}, MaxDegree {max_degree}, Dimension {dimension}"
            )));
        }
        let degree: i32 = largest_degree(spans.num_coeff_per_curve, num_curves)?;
        if (continuity > degree) && (num_curves > 1) {
            return Err(too_great(continuity, degree));
        }
        let mut result = Self::spans_frame(degree, num_curves, spans.true_intervals)?;
        let multiplicities: i32 = result.degree - continuity;
        for ii in 2..num_curves + 1 {
            result.mults[ii] = multiplicities;
        }
        result.set_end_mults(num_curves);
        result.perform(num_curves, max_degree, dimension, spans)?;
        Ok(result)
    }

    /// Spans joined with their own continuities, read at `continuity(2..NumCurves)`;
    /// `CompPolynomialToPoles::from_continuities(2, 1, 1, &continuity, spans)?`.
    pub fn from_continuities(
        num_curves: i32,
        dimension: i32,
        max_degree: i32,
        continuity: &Array1<'_, i32>,
        spans: PolynomialSpans<'_, '_>,
    ) -> Result<Self, Failure> {
        let columns: usize = spans.polynomial_intervals.nb_columns();
        if num_curves <= 0 || max_degree <= 0 || dimension <= 0 || columns != 2 {
            return Err(bad_arguments(format!(
                "NumCurves {num_curves}, MaxDegree {max_degree}, Dimension {dimension}, {columns} domain columns"
            )));
        }
        let degree: i32 = largest_degree(spans.num_coeff_per_curve, num_curves)?;
        let mut result = Self::spans_frame(degree, num_curves, spans.true_intervals)?;
        for ii in 2..num_curves + 1 {
            let value: i32 = array_value(continuity, ii)?;
            if (value > result.degree) && (num_curves > 1) {
                return Err(too_great(value, result.degree));
            }
            result.mults[ii] = result.degree - value;
        }
        result.set_end_mults(num_curves);
        result.perform(num_curves, max_degree, dimension, spans)?;
        Ok(result)
    }

    /// One span of `degree` on the domain `polynomial_intervals` mapped onto the
    /// first two `true_intervals`;
    /// `CompPolynomialToPoles::from_single_span(3, 1, 1, &coeffs, &domain, &target)?`.
    pub fn from_single_span(
        dimension: i32,
        max_degree: i32,
        degree: i32,
        coefficients: &Array1<'_, f64>,
        polynomial_intervals: &Array1<'_, f64>,
        true_intervals: &Array1<'_, f64>,
    ) -> Result<Self, Failure> {
        if max_degree <= 0 || dimension <= 0 || polynomial_intervals.len() != 2 {
            return Err(bad_arguments(format!(
                "MaxDegree {max_degree}, Dimension {dimension}, {} domain bounds",
                polynomial_intervals.len()
            )));
        }
        let mut domain: Array2<'static, f64> =
            Array2::with_bounds(1, 1, 1, 2).map_err(|e| bad_arguments(e.to_string()))?;
        domain[(1, 1)] = array_value(polynomial_intervals, polynomial_intervals.lower())?;
        domain[(1, 2)] = array_value(polynomial_intervals, polynomial_intervals.upper())?;
        let mut counts: Array1<'static, i32> = one_based(1)?;
        counts[1] = degree.wrapping_add(1);
        let mut result = Self::empty(degree, 2)?;
        result.knots[1] = array_value(true_intervals, true_intervals.lower())?;
        result.knots[2] = array_value(true_intervals, true_intervals.lower() + 1)?;
        result.mults.init(degree.wrapping_add(1));
        let spans = PolynomialSpans {
            num_coeff_per_curve: &counts,
            coefficients,
            polynomial_intervals: &domain,
            true_intervals,
        };
        result.perform(1, max_degree, dimension, spans)?;
        Ok(result)
    }

    fn empty(degree: i32, nb_knots: i32) -> Result<Self, Failure> {
        Ok(Self {
            flat_knots: Array1::default(),
            knots: one_based(nb_knots)?,
            mults: one_based(nb_knots)?,
            poles: Array2::default(),
            degree,
            done: false,
        })
    }

    // Knots = the true breakpoints read from their lower bound.
    fn spans_frame(
        degree: i32,
        num_curves: i32,
        true_intervals: &Array1<'_, f64>,
    ) -> Result<Self, Failure> {
        let mut result = Self::empty(degree, num_curves + 1)?;
        let lower: i32 = true_intervals.lower();
        for ii in 1..=num_curves + 1 {
            result.knots[ii] = array_value(true_intervals, lower + ii - 1)?;
        }
        Ok(result)
    }

    fn set_end_mults(&mut self, num_curves: i32) {
        self.mults[1] = self.degree + 1;
        self.mults[num_curves + 1] = self.degree + 1;
    }

    /// Number of poles, 0 before a successful conversion; `conversion.nb_poles()`.
    pub fn nb_poles(&self) -> i32 {
        if self.done {
            self.poles.nb_rows() as i32
        } else {
            0
        }
    }

    /// Poles `[1..NbPoles][1..Dimension]`; `conversion.poles()?[(1, 1)]`.
    pub fn poles(&self) -> Result<&Array2<'static, f64>, Failure> {
        self.check_done("Poles")?;
        Ok(&self.poles)
    }

    /// Copy of the poles when done (deprecated handle overload); `conversion.poles_copy()`.
    pub fn poles_copy(&self) -> Option<Array2<'static, f64>> {
        self.done.then(|| {
            let mut copy: Array2<'static, f64> = Array2::default();
            copy.assign(&self.poles);
            copy
        })
    }

    /// Degree, 0 before a successful conversion; `conversion.degree()`.
    pub fn degree(&self) -> i32 {
        if self.done { self.degree } else { 0 }
    }

    /// Number of knots, 0 before a successful conversion; `conversion.nb_knots()`.
    pub fn nb_knots(&self) -> i32 {
        if self.done {
            self.knots.len() as i32
        } else {
            0
        }
    }

    /// Knots; `conversion.knots()?[1]`.
    pub fn knots(&self) -> Result<&Array1<'static, f64>, Failure> {
        self.check_done("Knots")?;
        Ok(&self.knots)
    }

    /// Copy of the knots when done (deprecated handle overload); `conversion.knots_copy()`.
    pub fn knots_copy(&self) -> Option<Array1<'static, f64>> {
        self.done.then(|| self.knots.clone())
    }

    /// Multiplicities; `conversion.multiplicities()?[1]`.
    pub fn multiplicities(&self) -> Result<&Array1<'static, i32>, Failure> {
        self.check_done("Multiplicities")?;
        Ok(&self.mults)
    }

    /// Copy of the multiplicities when done (deprecated handle overload);
    /// `conversion.multiplicities_copy()`.
    pub fn multiplicities_copy(&self) -> Option<Array1<'static, i32>> {
        self.done.then(|| self.mults.clone())
    }

    /// True after a successful conversion; `conversion.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    fn check_done(&self, accessor: &str) -> Result<(), Failure> {
        if self.done {
            return Ok(());
        }
        Err(Failure::with_type(
            "StdFail_NotDone",
            &format!("Convert_CompPolynomialToPoles::{accessor}: the conversion is not done"),
            "",
        ))
    }
}

// The largest coefficient count - 1 of the first `num_curves` spans, at least 0.
fn largest_degree(counts: &Array1<'_, i32>, num_curves: i32) -> Result<i32, Failure> {
    let mut degree: i32 = 0;
    for ii in counts.lower()..=counts.lower() + (num_curves - 1) {
        degree = degree.max(array_value(counts, ii)?.wrapping_sub(1));
    }
    Ok(degree)
}

/// Reads `PolynomialIntervals(row, column)` with the native bound check;
/// `domain_bound(&intervals, 1, 2)?`.
pub(super) fn domain_bound(
    intervals: &Array2<'_, f64>,
    row: i32,
    column: i32,
) -> Result<f64, Failure> {
    grid_value(intervals, row, column)
}
