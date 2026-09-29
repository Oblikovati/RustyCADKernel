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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_LinearCriteria.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppDef_LinearCriteria`: the smoothing criterion of AppDef_Variational, the
//! weighted sum of the least-squares distance to the points and the tension,
//! flexion and jerk energies (FEmTool_LinearTension, _LinearFlexion, _LinearJerk).
mod curve;
mod hessian;
mod quality;
mod tables;

use super::multi_line::MultiLine;
use super::smooth_criterion::{QualityValues, SmoothCriterion, SmoothErrors};
use crate::fem_tool::{AssemblyTable, Curve, ElementaryCriterion};
use crate::native_failure::{allocation_failure, array_failure, failure};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, HArray1, HArray2};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::math::{Matrix, Vector};

/// The mutable fields of `AppDef_LinearCriteria`, shared by every handle.
pub(super) struct CriteriaState {
    parameters: Handle<HArray1<f64>>,
    cache: Handle<HArray1<f64>>,
    criteria: [Handle<dyn ElementaryCriterion>; 3],
    estimation: [f64; 3],
    quadratic_weight: f64,
    quality_weight: f64,
    percent: [f64; 3],
    pnt_weight: Array1<'static, f64>,
    curve: Handle<Curve>,
    length: f64,
    // myE: the element of the cache (0 when invalid), IF..IL its points.
    element: i32,
    first_index: i32,
    last_index: i32,
}

/// `AppDef_LinearCriteria`: the criterion of the points `FirstPoint..=LastPoint` of
/// a MultiLine. The state is behind a lock, so every handle sees it and the
/// native non-const members take `&self`.
///
/// ```
/// use rustycad_tkgeombase::app_def::{LinearCriteria, MultiLine, SmoothCriterion};
/// use rustycad_tkernel::ncollection::Array1;
/// use rustycad_tkmath::gp::Point3;
/// let p = [Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 0.0), Point3::new(2.0, 0.0, 0.0)];
/// let criterion = LinearCriteria::new(&MultiLine::from_points(&Array1::from_shared(&p, 1)), 1, 3)?;
/// criterion.set_weight(2.0, 0.5, [1.0, 1.0, 2.0])?;
/// assert_eq!(criterion.get_weight(), (2.0, 0.5));
/// assert!(criterion.assembly_table().is_err()); // no curve yet
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct LinearCriteria {
    parent: StandardTransient,
    ssp: MultiLine,
    state: SharedState<CriteriaState>,
}
impl_transient!(
    LinearCriteria,
    "AppDef_LinearCriteria",
    dyn SmoothCriterion,
    parent,
    interfaces: [dyn SmoothCriterion]
);

// Standard_DomainError of a member called before SetCurve, or with a bad argument.
fn domain(member: &str, detail: &str) -> Failure {
    failure(
        "Standard_DomainError",
        &format!("AppDef_LinearCriteria::{member}: {detail}"),
    )
}

impl LinearCriteria {
    /// `AppDef_LinearCriteria(SSP, FirstPoint, LastPoint)`: keeps a copy of the line
    /// (sharing its multipoints), point weights 1 on `FirstPoint..=LastPoint` and
    /// zero estimations and weights; Standard_OutOfMemory when `LastPoint <
    /// FirstPoint - 1` (a wrapped allocation size natively);
    /// `LinearCriteria::new(&line, 1, 10)?`.
    pub fn new(ssp: &MultiLine, first_point: i32, last_point: i32) -> Result<Self, Failure> {
        let mut pnt_weight: Array1<'static, f64> = Array1::with_bounds(first_point, last_point)
            .map_err(|error| allocation_failure(error, "AppDef_LinearCriteria"))?;
        pnt_weight.init(1.);
        let state: CriteriaState = CriteriaState {
            parameters: Handle::default(),
            cache: Handle::default(),
            criteria: Default::default(),
            estimation: [0.; 3],
            quadratic_weight: 0.,
            quality_weight: 0.,
            percent: [0.; 3],
            pnt_weight,
            curve: Handle::default(),
            length: 0.,
            element: 0,
            first_index: 0,
            last_index: 0,
        };
        Ok(Self {
            parent: StandardTransient::default(),
            ssp: ssp.clone(),
            state: SharedState::new(state),
        })
    }

    // The curve, or Standard_DomainError before SetCurve (the native null check).
    fn checked_curve(&self, member: &str) -> Result<Handle<Curve>, Failure> {
        let curve: Handle<Curve> = self.state.read().curve.clone();
        if curve.is_null() {
            return Err(domain(member, "no curve; expected SetCurve first"));
        }
        Ok(curve)
    }
}

/// The dereference of a handle the native code assumes non-null (a crash natively).
pub(super) fn non_null<'a, T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    handle: &'a Handle<T>,
    what: &str,
) -> Result<&'a T, Failure> {
    handle.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            &format!("AppDef_LinearCriteria: {what} is null; expected it set first"),
        )
    })
}

impl SmoothCriterion for LinearCriteria {
    fn set_parameters(&self, parameters: &Handle<HArray1<f64>>) {
        let mut state = self.state.write();
        state.parameters = parameters.clone();
        state.element = 0; // Cache become invalid.
    }
    fn set_curve(&self, curve: &Handle<Curve>) -> Result<(), Failure> {
        curve::set_curve(&mut self.state.write(), curve)
    }
    fn get_curve(&self) -> Handle<Curve> {
        self.state.read().curve.clone()
    }
    fn set_estimation(&self, e1: f64, e2: f64, e3: f64) {
        self.state.write().estimation = [e1, e2, e3];
    }
    fn est_length(&self) -> f64 {
        self.state.read().length
    }
    fn set_est_length(&self, length: f64) {
        self.state.write().length = length;
    }
    fn get_estimation(&self) -> [f64; 3] {
        self.state.read().estimation
    }
    fn assembly_table(&self) -> Result<AssemblyTable, Failure> {
        tables::assembly_table(&self.checked_curve("AssemblyTable")?)
    }
    fn dependence_table(&self) -> Result<Handle<HArray2<i32>>, Failure> {
        tables::dependence_table(&self.checked_curve("DependenceTable")?)
    }
    fn quality_values(&self, j1min: f64, j2min: f64, j3min: f64) -> Result<QualityValues, Failure> {
        let curve: Handle<Curve> = self.checked_curve("QualityValues")?;
        quality::quality_values(&mut self.state.write(), &curve, [j1min, j2min, j3min])
    }
    fn error_values(&self) -> Result<SmoothErrors, Failure> {
        let curve: Handle<Curve> = self.checked_curve("ErrorValues")?;
        quality::error_values(&self.state.read(), &curve, &self.ssp)
    }
    fn hessian(
        &self,
        element: i32,
        dimension1: i32,
        dimension2: i32,
        h: &mut Matrix<'_>,
    ) -> Result<(), Failure> {
        let curve: Handle<Curve> = self.checked_curve("Hessian")?;
        let dependence = tables::dependence_table(&curve)?;
        let coupled: i32 = *dependence
            .array2()
            .value(dimension1, dimension2)
            .map_err(|error| array_failure(error, "AppDef_LinearCriteria::Hessian"))?;
        if coupled == 0 {
            return Err(domain(
                "Hessian",
                &format!("dimensions {dimension1} and {dimension2} are independent"),
            ));
        }
        hessian::hessian(
            &mut self.state.write(),
            &curve,
            (element, [dimension1, dimension2]),
            h,
        )
    }
    fn gradient(&self, element: i32, dimension: i32, g: &mut Vector<'_>) -> Result<(), Failure> {
        // The native messages name ErrorValues here.
        let curve: Handle<Curve> = self.checked_curve("ErrorValues")?;
        let target = hessian::GradientTarget::new(&self.ssp, dimension)?;
        hessian::gradient(
            &mut self.state.write(),
            &curve,
            element,
            (&self.ssp, &target),
            g,
        )
    }
    fn input_vector(&self, x: &Vector<'_>, ass_table: &AssemblyTable) -> Result<(), Failure> {
        let curve: Handle<Curve> = self.state.read().curve.clone();
        hessian::input_vector(non_null(&curve, "the curve")?, x, ass_table)
    }
    fn set_weight(
        &self,
        quadratic_weight: f64,
        quality_weight: f64,
        [p1, p2, p3]: [f64; 3],
    ) -> Result<(), Failure> {
        if quadratic_weight < 0. || quality_weight < 0. {
            return Err(domain(
                "SetWeight",
                &format!("weights {quadratic_weight}, {quality_weight}; expected >= 0"),
            ));
        }
        if p1 < 0. || p2 < 0. || p3 < 0. {
            return Err(domain(
                "SetWeight",
                &format!("percents {p1}, {p2}, {p3}; expected >= 0"),
            ));
        }
        let mut state = self.state.write();
        state.quadratic_weight = quadratic_weight;
        state.quality_weight = quality_weight;
        let total: f64 = p1 + p2 + p3;
        state.percent = [p1 / total, p2 / total, p3 / total];
        Ok(())
    }
    fn get_weight(&self) -> (f64, f64) {
        let state = self.state.read();
        (state.quadratic_weight, state.quality_weight)
    }
    fn set_point_weights(&self, weight: &Array1<'_, f64>) {
        self.state.write().pnt_weight.assign(weight);
    }
}
