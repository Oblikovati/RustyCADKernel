// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_Variational.cxx (Approximate, Distance)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approximate`: the smoothing, then the conversion of the Hermite-Jacobi elements
//! into a B-spline MultiCurve; `Distance`: the point distances to it.
use super::super::point_tabs::PointTabs;
use super::super::smooth_criterion::SmoothCriterion;
use super::{Variational, criterion_ref, not_done};
use crate::app_par_curves::{MultiBSpCurve, MultiCurveEval, MultiPoint};
use crate::fem_tool::Curve;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::{Array1, Array2, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::convert::{CompPolynomialToPoles, PolynomialSpans};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::math::Matrix;

const CONTEXT: &str = "AppDef_Variational::Approximate";

fn array_error(error: rustycad_tkernel::ncollection::ArrayError) -> Failure {
    array_failure(error, CONTEXT)
}

/// The Convert_CompPolynomialToPoles inputs: the span of each element, the
/// coefficient count of each, the coefficients and the breakpoints.
struct ConvertInputs {
    intervals: Array2<'static, f64>,
    nb_coeff: Array1<'static, i32>,
    coeff: Array1<'static, f64>,
    true_intervals: Array1<'static, f64>,
}

impl ConvertInputs {
    // The coefficient count (degree + 1) of each element of `curve`, then the span
    // [-1, 1] of each element.
    fn fill_spans(&mut self, curve: &Curve) -> Result<(), Failure> {
        for ii in 1..=curve.nb_elements() {
            self.nb_coeff
                .set_value(ii, curve.degree(ii)? + 1)
                .map_err(array_error)?;
        }
        for ii in self.intervals.lower_row()..=self.intervals.upper_row() {
            self.intervals.set_value(ii, 1, -1.).map_err(array_error)?;
            self.intervals.set_value(ii, 2, 1.).map_err(array_error)?;
        }
        Ok(())
    }
}

impl Variational {
    /// `Approximate()`: the smoothing with the current fields (with Adjusting when
    /// minimizing the maximum error above the tolerance), converted to a B-spline;
    /// StdFail_NotDone unless created. The done flag is never cleared: a failed
    /// conversion keeps the previous result.
    ///
    /// ```
    /// use rustycad_tkgeombase::app_def::{MultiLine, Variational, VariationalOptions};
    /// use rustycad_tkernel::ncollection::{Array1, HArray1};
    /// use rustycad_tkernel::standard::Handle;
    /// use rustycad_tkmath::gp::Point2;
    /// let points: Vec<Point2> = (0..10).map(|i| Point2::new(f64::from(i), f64::from(i % 3))).collect();
    /// let line = MultiLine::from_points2d(&Array1::from_shared(&points, 1));
    /// let none = Handle::new(HArray1::from_array(&Array1::from_shared(&[], 1)));
    /// let mut smoothing = Variational::new(&line, 1, 10, &none, VariationalOptions::default())?;
    /// smoothing.approximate()?;
    /// assert!(smoothing.is_done() && smoothing.max_error()? > 0.0);
    /// # Ok::<(), rustycad_tkernel::standard::Failure>(())
    /// ```
    pub fn approximate(&mut self) -> Result<(), Failure> {
        if !self.is_created {
            return Err(not_done("Approximate"));
        }
        let mut ecarts: Array1<'static, f64> =
            Array1::with_bounds(self.first_point, self.last_point).map_err(array_error)?;
        let (mut the_curve, (mut w_quadratic, w_quality)) = self.first_smoothing(&mut ecarts)?;
        if self.options.with_min_max && self.options.tolerance < self.results.max_error {
            self.adjust_member_criterion(
                (&mut w_quadratic, w_quality),
                &mut the_curve,
                &mut ecarts,
            )?;
        }
        self.convert(&the_curve)
    }

    // TheMotor with the member criterion from its curve; returns the smoothed curve
    // and the criterion's weights `(WQuadratic, WQuality)`.
    fn first_smoothing(
        &mut self,
        ecarts: &mut Array1<'_, f64>,
    ) -> Result<(Handle<Curve>, (f64, f64)), Failure> {
        let criterion: Handle<dyn SmoothCriterion> = self.smooth_criterion.clone();
        let (w_quadratic, w_quality) = criterion_ref(&criterion)?.get_weight();
        let mut the_curve: Handle<Curve> = criterion_ref(&criterion)?.get_curve();
        self.the_motor(
            criterion_ref(&criterion)?,
            w_quality,
            &mut the_curve,
            ecarts,
        )?;
        Ok((the_curve, (w_quadratic, w_quality)))
    }

    // Adjusting on the member criterion, which it replaces through its reference,
    // even when a later pass raises.
    fn adjust_member_criterion(
        &mut self,
        (w_quadratic, w_quality): (&mut f64, f64),
        the_curve: &mut Handle<Curve>,
        ecarts: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let mut j: Handle<dyn SmoothCriterion> = self.smooth_criterion.clone();
        let adjusted = self.adjusting(&mut j, (w_quadratic, w_quality), the_curve, ecarts);
        self.smooth_criterion = j;
        adjusted
    }

    // The Convert_CompPolynomialToPoles inputs: one span per element on [-1, 1],
    // the element polynomials and degrees, the knots as breakpoints.
    fn convert(&mut self, the_curve: &Handle<Curve>) -> Result<(), Failure> {
        let curve: &Curve = the_curve
            .get()
            .ok_or_else(|| Failure::new("AppDef_Variational::Approximate: null curve"))?;
        let mut inputs: ConvertInputs = self.convert_buffers(curve.nb_elements())?;
        inputs.true_intervals.assign(&curve.knots().clone());
        curve.get_polynom(&mut inputs.coeff)?;
        inputs.fill_spans(curve)?;
        let converter = self.polynomial_converter(curve.nb_elements(), &inputs)?;
        if converter.is_done() {
            self.store_result(&converter)?;
        }
        Ok(())
    }

    // Convert_CompPolynomialToPoles of the `nb_elem` element polynomials with the
    // continuity of the smoothing and MaxDegree.
    fn polynomial_converter(
        &self,
        nb_elem: i32,
        inputs: &ConvertInputs,
    ) -> Result<CompPolynomialToPoles, Failure> {
        let spans = PolynomialSpans {
            num_coeff_per_curve: &inputs.nb_coeff,
            coefficients: &inputs.coeff,
            polynomial_intervals: &inputs.intervals,
            true_intervals: &inputs.true_intervals,
        };
        CompPolynomialToPoles::from_uniform_continuity(
            nb_elem,
            self.niv_cont,
            self.dimension,
            self.options.max_degree,
            Some(spans),
        )
    }

    // The conversion buffers for `nb_elem` elements, sized by MaxSegment and
    // MaxDegree, the coefficients at 0.
    fn convert_buffers(&self, nb_elem: i32) -> Result<ConvertInputs, Failure> {
        let (max_segment, max_degree) = (self.options.max_segment, self.options.max_degree);
        let intervals: Array2<'static, f64> =
            Array2::with_bounds(1, nb_elem, 1, 2).map_err(array_error)?;
        let nb_coeff: Array1<'static, i32> =
            Array1::with_bounds(1, max_segment).map_err(array_error)?;
        let size: i32 = max_segment * (max_degree + 1) * self.dimension;
        let mut coeff: Array1<'static, f64> = Array1::with_bounds(1, size).map_err(array_error)?;
        coeff.init(0.);
        let true_intervals: Array1<'static, f64> =
            Array1::with_bounds(1, nb_elem + 1).map_err(array_error)?;
        Ok(ConvertInputs {
            intervals,
            nb_coeff,
            coeff,
            true_intervals,
        })
    }

    // The poles of the converter as MultiPoints (3D points then 2D points per row).
    fn store_result(&mut self, converter: &CompPolynomialToPoles) -> Result<(), Failure> {
        let poles: &Array2<'static, f64> = converter.poles()?;
        let mut tab_mu: Array1<'static, MultiPoint> =
            Array1::with_bounds(1, converter.nb_poles()).map_err(array_error)?;
        let knots: Array1<'static, f64> = converter.knots()?.clone();
        let mults: Array1<'static, i32> = converter.multiplicities()?.clone();
        let mut tabs: PointTabs = PointTabs::new([self.nb_p3d, self.nb_p2d])?;
        for ipole in poles.lower_row()..=poles.upper_row() {
            let mut index: i32 = poles.lower_col();
            let mut next = || -> Result<f64, Failure> {
                let value: f64 = *poles.value(ipole, index).map_err(array_error)?;
                index += 1;
                Ok(value)
            };
            for jp3d in 1..=self.nb_p3d {
                let point = Point3::new(next()?, next()?, next()?);
                tabs.p3d.set_value(jp3d, point).map_err(array_error)?;
            }
            for jp2d in 1..=self.nb_p2d {
                let point = Point2::new(next()?, next()?);
                tabs.p2d.set_value(jp2d, point).map_err(array_error)?;
            }
            let point: MultiPoint = if self.nb_p2d != 0 && self.nb_p3d != 0 {
                MultiPoint::from_points_and_2d(&tabs.p3d, &tabs.p2d)
            } else if self.nb_p2d != 0 {
                MultiPoint::from_points2d(&tabs.p2d)
            } else {
                MultiPoint::from_points(&tabs.p3d)
            };
            tab_mu.set_value(ipole, point).map_err(array_error)?;
        }
        // A new array: handles from an earlier Knots() keep the old knots.
        self.knots = Handle::new(HArray1::from_array(&knots));
        self.results.mbsp_curve = MultiBSpCurve::from_multi_points(&tab_mu, &knots, &mults);
        self.is_done = true;
        Ok(())
    }

    /// `Distance(mat)`: `mat(k, LowerCol + ipoint - FirstPoint)` is the distance of
    /// point `k` of multipoint `ipoint` (3D points then 2D ones, the rows counted from
    /// 1 whatever `mat.LowerRow()`) to the curve at its parameter; StdFail_NotDone
    /// before a successful approximation; `smoothing.distance(&mut mat)?`.
    pub fn distance(&self, mat: &mut Matrix<'_>) -> Result<(), Failure> {
        if !self.is_done {
            return Err(not_done("Distance"));
        }
        let mut tabs: PointTabs = PointTabs::new([self.nb_p3d, self.nb_p2d])?;
        let j0: i32 = mat.lower_col() - self.first_point;
        let curve: &MultiBSpCurve = &self.results.mbsp_curve;
        for ipoint in self.first_point..=self.last_point {
            let mut index: i32 = 1;
            if self.nb_p3d != 0 {
                tabs.read_3d(&self.ssp, ipoint)?;
                for jp3d in 1..=self.nb_p3d {
                    let p3d: Point3 = tabs.p3d[jp3d];
                    let pt3d: Point3 = curve.value_3d(index, self.parameter(ipoint)?)?;
                    *mat.change_value(index, j0 + ipoint)? = p3d.distance(&pt3d);
                    index += 1;
                }
            }
            if self.nb_p2d != 0 {
                tabs.read_2d(&self.ssp, ipoint)?;
                for jp2d in 1..=self.nb_p2d {
                    let p2d: Point2 = tabs.p2d[jp2d];
                    let pt2d: Point2 = curve.value_2d(index, self.parameter(ipoint)?)?;
                    *mat.change_value(index, j0 + ipoint)? = p2d.distance(pt2d);
                    index += 1;
                }
            }
        }
        Ok(())
    }
}
