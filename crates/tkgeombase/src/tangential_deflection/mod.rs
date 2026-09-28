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
// Rust translation of ModelingData/TKGeomBase/GCPnts/GCPnts_TangentialDeflection.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GCPnts_TangentialDeflection`: points on a curve such that the chord and angular
//! deflections between successive points stay below the requested bounds.
//!
//! The ExtremaPC and ExtremaPC2d curve-aware partitions reach it through
//! `Extrema_CurveTool::DeflCurvIntervals` ([`crate::defl_curv_intervals`]). The GCPnts
//! package is not ported yet, so this crate-private module ports the complete class for
//! them, in the native operation order; the GCPnts port will replace it.
mod curve;
mod march;
mod refine;
mod simple;
mod stepping;

pub(crate) use curve::{DeflectionCurve, PlanarCurve, SpatialCurve};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point3;

/// `Us3` of GCPnts_TangentialDeflection.cxx: the literal 0.3333333333333333333333333333
/// rounds to the same double as 1/3.
const US3: f64 = 1.0 / 3.0;

/// The deflection-bounded points of a curve (`GCPnts_TangentialDeflection`); the
/// sequences are 1-based through [`Self::parameter`] and [`Self::value`], as natively;
/// `TangentialDeflection::with_curve(&PlanarCurve(&adaptor), 0.5, 0.01, 2, 1e-9, 1e-7)?`.
#[derive(Clone, Debug, Default)]
pub(crate) struct TangentialDeflection {
    angular_deflection: f64,
    curvature_deflection: f64,
    u_tol: f64,
    min_nb_pnts: i32,
    min_len: f64,
    last_u: f64,
    first_u: f64,
    points: Vec<Point3>,
    parameters: Vec<f64>,
}

/// The deflection criteria of one run: `theAngularDeflection`, `theCurvatureDeflection`,
/// `theMinimumOfPoints`, `theUTol` and `theMinLen` (native defaults 2, 1e-9, 1e-7).
#[derive(Clone, Copy, Debug)]
pub(crate) struct DeflectionCriteria {
    pub angular: f64,
    pub curvature: f64,
    pub minimum_of_points: i32,
    pub u_tol: f64,
    pub min_len: f64,
}

impl TangentialDeflection {
    /// The constructor on the curve bounds (`GCPnts_TangentialDeflection(theC, ...)`);
    /// Standard_ConstructionError for a deflection below Confusion or Angular.
    pub fn with_curve(
        curve: &impl DeflectionCurve,
        criteria: DeflectionCriteria,
    ) -> Result<Self, Failure> {
        let mut result = Self::default();
        result.initialize(curve, criteria)?;
        Ok(result)
    }
    /// The constructor on `[first, last]` (`GCPnts_TangentialDeflection(theC, U1, U2, ...)`).
    // The complete native class is ported; ExtremaPC uses only part of it so far.
    #[allow(dead_code)]
    pub fn with_range(
        curve: &impl DeflectionCurve,
        range: (f64, f64),
        criteria: DeflectionCriteria,
    ) -> Result<Self, Failure> {
        let mut result = Self::default();
        result.initialize_range(curve, range, criteria)?;
        Ok(result)
    }
    /// `Initialize(theC, ...)` over the curve bounds.
    pub fn initialize(
        &mut self,
        curve: &impl DeflectionCurve,
        criteria: DeflectionCriteria,
    ) -> Result<(), Failure> {
        let range: (f64, f64) = (curve.first()?, curve.last()?);
        self.initialize_range(curve, range, criteria)
    }
    /// `Initialize(theC, theFirstParameter, theLastParameter, ...)`.
    pub fn initialize_range(
        &mut self,
        curve: &impl DeflectionCurve,
        range: (f64, f64),
        criteria: DeflectionCriteria,
    ) -> Result<(), Failure> {
        check_deflections(criteria)?;
        self.parameters.clear();
        self.points.clear();
        (self.first_u, self.last_u) = if range.0 < range.1 {
            range
        } else {
            (range.1, range.0)
        };
        self.u_tol = criteria.u_tol;
        self.angular_deflection = criteria.angular;
        self.curvature_deflection = criteria.curvature;
        self.min_nb_pnts = criteria.minimum_of_points.max(2);
        self.min_len = std_max(criteria.min_len, precision::confusion());
        self.perform_by_kind(curve)
    }
    // The switch on GetType() of `initialize`.
    fn perform_by_kind(&mut self, curve: &impl DeflectionCurve) -> Result<(), Failure> {
        match curve.kind()? {
            CurveType::Line => self.perform_linear(curve),
            CurveType::Circle => self.perform_circular(curve),
            CurveType::BSplineCurve if curve.bspline_shape()?.0 == 2 => self.perform_linear(curve),
            CurveType::BezierCurve if curve.bezier_shape()?.0 == 2 => self.perform_linear(curve),
            _ => self.perform_curve(curve),
        }
    }
    /// `AddPoint`: insert a point in parameter order, or replace (when `replace`) the
    /// point whose parameter is within PConfusion; returns its 1-based index.
    // The complete native class is ported; ExtremaPC uses only part of it so far.
    #[allow(dead_code)]
    pub fn add_point(&mut self, point: Point3, parameter: f64, replace: bool) -> i32 {
        let tolerance: f64 = precision::p_confusion();
        for index in 0..self.parameters.len() {
            let distance: f64 = self.parameters[index] - parameter;
            if distance.abs() <= tolerance {
                if replace {
                    self.points[index] = point;
                    self.parameters[index] = parameter;
                }
                return index as i32 + 1;
            }
            if distance > tolerance {
                self.points.insert(index, point);
                self.parameters.insert(index, parameter);
                return index as i32 + 1;
            }
        }
        self.append(parameter, point);
        self.parameters.len() as i32
    }
    /// `NbPoints`.
    pub fn nb_points(&self) -> i32 {
        self.parameters.len() as i32
    }
    /// `Parameter(I)`: Standard_OutOfRange outside `[1, NbPoints]`.
    pub fn parameter(&self, index: i32) -> Result<f64, Failure> {
        sequence_value(&self.parameters, index).copied()
    }
    /// `Value(I)`: Standard_OutOfRange outside `[1, NbPoints]`.
    // The complete native class is ported; ExtremaPC uses only part of it so far.
    #[allow(dead_code)]
    pub fn value(&self, index: i32) -> Result<Point3, Failure> {
        sequence_value(&self.points, index).copied()
    }
    // `myParameters.Append(U); myPoints.Append(P);`
    fn append(&mut self, parameter: f64, point: Point3) {
        self.parameters.push(parameter);
        self.points.push(point);
    }
}

// Standard_ConstructionError_Raise_if of `initialize`; NaN deflections pass, as natively.
fn check_deflections(criteria: DeflectionCriteria) -> Result<(), Failure> {
    if criteria.curvature < precision::confusion() || criteria.angular < precision::angular() {
        let message = format!(
            "GCPnts_TangentialDeflection::Initialize - Zero Deflection: angular {}, curvature {}; expected >= Precision::Angular() and >= Precision::Confusion()",
            criteria.angular, criteria.curvature
        );
        return Err(Failure::with_type(
            "Standard_ConstructionError",
            &message,
            "",
        ));
    }
    Ok(())
}

// `NCollection_Sequence::Value(I)` of a 1-based sequence.
fn sequence_value<T>(values: &[T], index: i32) -> Result<&T, Failure> {
    let position: Option<usize> = usize::try_from(index).ok().and_then(|i| i.checked_sub(1));
    position.and_then(|i| values.get(i)).ok_or_else(|| {
        let message = format!(
            "NCollection_Sequence::Value: index {index}; expected in [1, {}]",
            values.len()
        );
        Failure::out_of_range(&message)
    })
}

/// `ArcAngularStep`: the angular step of an arc of `radius` for the deflections;
/// Standard_ConstructionError for a negative radius.
pub(crate) fn arc_angular_step(
    radius: f64,
    linear_deflection: f64,
    angular_deflection: f64,
    min_length: f64,
) -> Result<f64, Failure> {
    if radius < 0.0 {
        let message = format!("Negative radius: {radius}; expected >= 0");
        return Err(Failure::with_type(
            "Standard_ConstructionError",
            &message,
            "",
        ));
    }
    let precision: f64 = precision::confusion();
    let (mut du, mut min_size_ang) = (0.0, 0.0);
    if radius > precision {
        du = std_max(1.0 - (linear_deflection / radius), 0.0);
        // It is not suitable to consider min size greater than 1/4 arc len.
        if min_length > precision {
            min_size_ang = rustycad_tkg2d::geom2d::native::std_min(
                min_length / radius,
                std::f64::consts::FRAC_PI_2,
            );
        }
    }
    du = 2.0 * du.acos();
    Ok(std_max(
        rustycad_tkg2d::geom2d::native::std_min(du, angular_deflection),
        min_size_ang,
    ))
}
