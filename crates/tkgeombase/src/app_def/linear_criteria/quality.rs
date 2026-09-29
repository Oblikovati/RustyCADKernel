// Created on: 1998-11-30
// Created by: Igor FEOKTISTOV
// Copyright (c) 1998-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_LinearCriteria.cxx (QualityValues, ErrorValues)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The criteria and error values of `AppDef_LinearCriteria`.
use super::super::multi_line::MultiLine;
use super::super::point_tabs::PointTabs;
use super::super::smooth_criterion::{QualityValues, SmoothErrors};
use super::{CriteriaState, domain, non_null};
use crate::app_par_curves::LineTool;
use crate::fem_tool::{Curve, ElementaryCriterion};
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::{Array1, HArray2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bvh::std_max;
use rustycad_tkmath::gp::{Point2, Point3};

// Knots(index) of the curve, checked as natively.
pub(super) fn knot(curve: &Curve, index: i32) -> Result<f64, Failure> {
    curve
        .knots()
        .value(index)
        .copied()
        .map_err(|error| array_failure(error, "FEmTool_Curve::Knots"))
}

// J = sum over the elements of the three element criteria; the coefficient table is
// reallocated only when the element degree changes, and shared with the criteria.
fn criteria_sums(state: &CriteriaState, curve: &Curve) -> Result<[f64; 3], Failure> {
    let (mut sums, mut deg) = ([0.; 3], 0);
    let mut coeff: Handle<HArray2<f64>> = Handle::default();
    for el in 1..=curve.nb_elements() {
        let curdeg: i32 = curve.degree(el)?;
        if deg != curdeg {
            deg = curdeg;
            coeff =
                Handle::new(HArray2::with_bounds(0, deg, 1, curve.dimension()).map_err(
                    |error| array_failure(error, "AppDef_LinearCriteria::QualityValues"),
                )?);
        }
        curve.get_element(
            el,
            &mut non_null(&coeff, "the coefficients")?.change_array2(),
        )?;
        let (u_first, u_last) = (knot(curve, el)?, knot(curve, el + 1)?);
        for (sum, criterion) in sums.iter_mut().zip(&state.criteria) {
            let criterion: &dyn ElementaryCriterion = non_null(criterion, "an element criterion")?;
            criterion.set_coeff(&coeff);
            criterion.set(u_first, u_last);
            *sum += criterion.value()?;
        }
    }
    Ok(sums)
}

// (2) Test the improvement of estimates (overestimated criterion => no minimization).
fn improve_estimates(estimation: &mut [f64; 3], val_cri: [f64; 3], js_min: [f64; 3]) -> i32 {
    let mut icdana: i32 = 0;
    for i in 0..3 {
        if val_cri[i] < 0.8 * estimation[i] && estimation[i] > js_min[i] {
            if icdana < 1 {
                icdana = 1;
            }
            if val_cri[i] < 0.1 * estimation[i] {
                icdana = 2;
            }
            estimation[i] = std_max(1.05 * val_cri[i], js_min[i]);
        }
    }
    icdana
}

// (3) Update the estimates (underestimated criterion => poor conditioning): criterion
// `i` above `trigger` times its estimation grows it by `growth` of the value.
fn update_estimate(
    estimation: &mut f64,
    value: f64,
    [trigger, growth, severe]: [f64; 3],
    icdana: i32,
) -> i32 {
    if value <= *estimation * trigger {
        return icdana;
    }
    *estimation += value * growth;
    if icdana != 0 {
        return 2;
    }
    if value > *estimation * severe { 2 } else { 1 }
}

/// `QualityValues(J1min, J2min, J3min, J1, J2, J3)`: the criteria of the curve
/// (Calculation of ICDANA - see MOTEST.f), updating the estimations.
pub(super) fn quality_values(
    state: &mut CriteriaState,
    curve: &Handle<Curve>,
    js_min: [f64; 3],
) -> Result<QualityValues, Failure> {
    let val_cri: [f64; 3] = criteria_sums(state, non_null(curve, "the curve")?)?;
    let mut icdana: i32 = improve_estimates(&mut state.estimation, val_cri, js_min);
    icdana = update_estimate(&mut state.estimation[0], val_cri[0], [2., 0.1, 10.], icdana);
    icdana = update_estimate(
        &mut state.estimation[1],
        val_cri[1],
        [20., 0.1, 100.],
        icdana,
    );
    icdana = update_estimate(
        &mut state.estimation[2],
        val_cri[2],
        [20., 0.05, 100.],
        icdana,
    );
    Ok(QualityValues {
        icdana,
        criteria: val_cri,
    })
}

/// The distance sums of the error values.
struct ErrorSums {
    max: f64,
    quadratic: f64,
    average: f64,
}

impl ErrorSums {
    fn add(&mut self, square_distance: f64) {
        let dist: f64 = square_distance.sqrt();
        self.max = std_max(self.max, dist);
        self.quadratic += square_distance;
        self.average += dist;
    }
}

// BasePoint(c0 + k), checked.
fn base(base_point: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    base_point
        .value(index)
        .copied()
        .map_err(|error| array_failure(error, "AppDef_LinearCriteria::ErrorValues"))
}

// The distances of the multipoint `i` to the curve point `base_point`.
fn point_errors(
    ssp: &MultiLine,
    i: i32,
    base_point: &Array1<'_, f64>,
    [nb_p3d, nb_p2d]: [i32; 2],
    (tabs, sums): (&mut PointTabs, &mut ErrorSums),
) -> Result<(), Failure> {
    let mut c0: i32 = 0;
    tabs.read_3d(ssp, i)?;
    for ipnt in 1..=nb_p3d {
        let p3d = Point3::new(
            base(base_point, c0 + 1)?,
            base(base_point, c0 + 2)?,
            base(base_point, c0 + 3)?,
        );
        sums.add(p3d.square_distance(&tabs.p3d[ipnt]));
        c0 += 3;
    }
    tabs.read_2d(ssp, i)?;
    for ipnt in 1..=nb_p2d {
        let p2d = Point2::new(base(base_point, c0 + 1)?, base(base_point, c0 + 2)?);
        sums.add(p2d.square_distance(tabs.p2d[ipnt]));
        c0 += 2;
    }
    Ok(())
}

/// `ErrorValues(MaxError, QuadraticError, AverageError)`: the distances of the
/// points to the curve at their parameters; Standard_DomainError when the curve
/// dimension is not `2 NbP2d + 3 NbP3d`.
pub(super) fn error_values(
    state: &CriteriaState,
    curve: &Handle<Curve>,
    ssp: &MultiLine,
) -> Result<SmoothErrors, Failure> {
    let curve: &Curve = non_null(curve, "the curve")?;
    let nb_dim: i32 = curve.dimension();
    let dims: [i32; 2] = [ssp.nb_p3d()?, ssp.nb_p2d()?];
    if nb_dim != 2 * dims[1] + 3 * dims[0] {
        return Err(domain(
            "ErrorValues",
            &format!(
                "curve dimension {nb_dim}; expected 2 NbP2d + 3 NbP3d = {}",
                2 * dims[1] + 3 * dims[0]
            ),
        ));
    }
    let mut base_point: Array1<'static, f64> = Array1::with_bounds(1, nb_dim)
        .map_err(|error| array_failure(error, "AppDef_LinearCriteria::ErrorValues"))?;
    let mut tabs: PointTabs = PointTabs::new(dims)?;
    let mut sums = ErrorSums {
        max: 0.,
        quadratic: 0.,
        average: 0.,
    };
    let parameters = non_null(&state.parameters, "the parameters")?
        .array1()
        .clone();
    for i in parameters.lower()..=parameters.upper() {
        curve.d0(parameters[i], &mut base_point)?;
        point_errors(ssp, i, &base_point, dims, (&mut tabs, &mut sums))?;
    }
    Ok(SmoothErrors {
        max_error: sums.max,
        quadratic_error: sums.quadratic,
        average_error: sums.average,
    })
}
