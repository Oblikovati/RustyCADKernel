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
// Rust translation of Convert_CompPolynomialToPoles::Perform at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Samples the polynomial spans at the Schoenberg points of the target knots and
//! interpolates them, which reproduces the polynomials exactly.
use super::arrays::one_based;
use super::comp_polynomial::{CompPolynomialToPoles, PolynomialSpans, domain_bound};
use super::cos_sin::construction;
use crate::bsplclib;
use crate::plib;
use crate::plib::access::array_value;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

// The span cursor: `index` counts spans from 2, `true_index` and `domain_row`
// follow it in their own arrays.
struct SpanCursor {
    index: i32,
    true_index: i32,
    domain_row: i32,
}

impl SpanCursor {
    // Moves to the span whose true interval contains the parameter; the breakpoint
    // is read before the span count is tested, as natively.
    fn advance(
        &mut self,
        parameter: f64,
        spans: PolynomialSpans<'_, '_>,
        num_curves: i32,
    ) -> Result<(), Failure> {
        while parameter >= array_value(spans.true_intervals, self.true_index)?
            && self.index <= num_curves
        {
            self.index += 1;
            self.true_index += 1;
            self.domain_row += 1;
        }
        Ok(())
    }
}

impl CompPolynomialToPoles {
    /// Builds the flat knots, samples every span and interpolates (`Perform`);
    /// `conversion.perform(1, 3, 1, spans)?`.
    pub(super) fn perform(
        &mut self,
        num_curves: i32,
        max_degree: i32,
        dimension: i32,
        spans: PolynomialSpans<'_, '_>,
    ) -> Result<(), Failure> {
        let parameters: Array1<'static, f64> = self.schoenberg_parameters()?;
        let rows: i32 = parameters.len() as i32;
        self.poles = Array2::with_bounds(1, rows, 1, dimension).map_err(|error| {
            let message = format!("Convert_CompPolynomialToPoles: {error}");
            Failure::with_type("Standard_OutOfMemory", &message, "")
        })?;
        self.sample_spans(num_curves, max_degree, dimension, spans, &parameters)?;
        self.interpolate_samples(&parameters, dimension)?;
        self.done = true;
        Ok(())
    }

    // The clamped flat knots of the multiplicities and their Schoenberg points.
    fn schoenberg_parameters(&mut self) -> Result<Array1<'static, f64>, Failure> {
        let mut num_flat_knots: i32 = 2 * self.degree + 2;
        for ii in 2..self.mults.len() as i32 {
            num_flat_knots += self.mults[ii];
        }
        self.flat_knots = one_based(num_flat_knots)?;
        let (knots, mults) = (&self.knots, &self.mults);
        bsplclib::knot_sequence_with_degree(
            knots,
            mults,
            self.degree,
            false,
            &mut self.flat_knots,
        )?;
        let mut parameters: Array1<'static, f64> = one_based(num_flat_knots - self.degree - 1)?;
        bsplclib::build_schoenberg_points(self.degree, &self.flat_knots, &mut parameters)?;
        Ok(parameters)
    }

    // Interpolation at the Schoenberg points yields the desired result.
    fn interpolate_samples(
        &mut self,
        parameters: &Array1<'_, f64>,
        dimension: i32,
    ) -> Result<(), Failure> {
        let mut contacts: Array1<'static, i32> = one_based(parameters.len() as i32)?;
        contacts.init(0);
        let mut inversion: i32 = 0;
        let (degree, flat_knots) = (self.degree, &self.flat_knots);
        let poles: &mut [f64] = self.poles.as_mut_slice();
        let order = &contacts;
        bsplclib::interpolate_flat(
            degree,
            flat_knots,
            parameters,
            order,
            dimension,
            poles,
            &mut inversion,
        )?;
        if inversion != 0 {
            return Err(construction(&format!(
                "Convert_CompPolynomialToPoles:inversion_problem at pivot {inversion}"
            )));
        }
        Ok(())
    }

    fn sample_spans(
        &mut self,
        num_curves: i32,
        max_degree: i32,
        dimension: i32,
        spans: PolynomialSpans<'_, '_>,
        parameters: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let mut cursor = SpanCursor {
            index: 2,
            true_index: spans.true_intervals.lower() + 1,
            domain_row: spans.polynomial_intervals.lower_row(),
        };
        let stride: i32 = dimension * (max_degree.max(self.degree) + 1);
        for ii in 1..=parameters.len() as i32 {
            cursor.advance(parameters[ii], spans, num_curves)?;
            let value: f64 = polynomial_parameter(parameters[ii], &cursor, spans)?;
            let first: i32 = ((cursor.index - 2) * stride) + spans.coefficients.lower();
            let counts: &Array1<'_, i32> = spans.num_coeff_per_curve;
            let deg: i32 = array_value(counts, counts.lower() + cursor.index - 2)? - 1;
            let offset: usize = (ii as usize - 1) * dimension as usize;
            let row: &mut [f64] = &mut self.poles.as_mut_slice()[offset..];
            let coefficients: &[f64] = coefficients_from(spans.coefficients, first)?;
            plib::no_derivative_eval_polynomial(
                value,
                deg,
                dimension,
                deg * dimension,
                coefficients,
                row,
            )?;
        }
        Ok(())
    }
}

// Normalized value so that it fits the original intervals for the polynomial
// definition of the curves.
fn polynomial_parameter(
    parameter: f64,
    cursor: &SpanCursor,
    spans: PolynomialSpans<'_, '_>,
) -> Result<f64, Failure> {
    let start: f64 = array_value(spans.true_intervals, cursor.true_index - 1)?;
    let end: f64 = array_value(spans.true_intervals, cursor.true_index)?;
    let mut normalized: f64 = parameter - start;
    normalized /= end - start;
    let domain: &Array2<'_, f64> = spans.polynomial_intervals;
    let low: f64 = domain_bound(domain, cursor.domain_row, domain.lower_col())?;
    let high: f64 = domain_bound(domain, cursor.domain_row, domain.upper_col())?;
    Ok((1.0e0 - normalized) * low + normalized * high)
}

// `(double*)&Coefficients(first)`: the checked element, then the raw tail after it.
fn coefficients_from<'c>(
    coefficients: &'c Array1<'_, f64>,
    first: i32,
) -> Result<&'c [f64], Failure> {
    array_value(coefficients, first)?;
    Ok(&coefficients.as_slice()[(first - coefficients.lower()) as usize..])
}
