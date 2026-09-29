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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_Variational.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppDef_Variational`: the smoothing of N points with constraints by
//! minimization of a quadratic criterion and of the variational criteria, to obtain
//! a "fair curve" (Jeannine PANCIATICI 06/06/96; Igor FEOKTISTOV 14/12/98).
mod accessors;
mod adjusting;
mod approximate;
mod constraints;
mod cutting;
mod dump;
mod estimations;
mod motor;
mod motor_support;
mod optimization;
mod project;
mod setters;
mod smoothing;

use super::linear_criteria::LinearCriteria;
use super::multi_line::MultiLine;
use super::point_tabs::PointTabs;
use super::smooth_criterion::SmoothCriterion;
use crate::app_par_curves::{Constraints, LineTool, MultiBSpCurve};
use crate::native_failure::{allocation_failure, array_failure, failure};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::Shape;

/// The optional arguments of the constructor, with the native defaults.
///
/// ```
/// use rustycad_tkgeombase::app_def::VariationalOptions;
/// let options = VariationalOptions { max_degree: 10, ..VariationalOptions::default() };
/// assert_eq!((options.max_segment, options.nb_iterations), (100, 2));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VariationalOptions {
    /// `MaxDegree` (14); at least 1, at most 30 (larger values are clamped).
    pub max_degree: i32,
    /// `MaxSegment` (100); at least 1.
    pub max_segment: i32,
    /// `Continuity` (C2); C0, C1 or C2.
    pub continuity: Shape,
    /// `WithMinMax` (false): minimize the maximum error too.
    pub with_min_max: bool,
    /// `WithCutting` (true): new knots may be inserted.
    pub with_cutting: bool,
    /// `Tolerance` (1.0).
    pub tolerance: f64,
    /// `NbIterations` (2).
    pub nb_iterations: i32,
}

impl Default for VariationalOptions {
    fn default() -> Self {
        Self {
            max_degree: 14,
            max_segment: 100,
            continuity: Shape::C2,
            with_min_max: false,
            with_cutting: true,
            tolerance: 1.0,
            nb_iterations: 2,
        }
    }
}

/// The constraint tables `Init` builds from the constraint couples.
#[derive(Clone, Default)]
struct ConstraintTables {
    // myTabConstraints: the unit tangents and the curvatures, 2 * Dimension per constraint.
    values: Array1<'static, f64>,
    nb_pass_points: i32,
    nb_tang_points: i32,
    nb_curv_points: i32,
    // myTypConstraints: (point index, 0 pass / 1 tangency / 2 curvature) pairs.
    types: Array1<'static, i32>,
    // myTtheta, myTfthet: the normal frames of the tangency and curvature constraints.
    ttheta: Array1<'static, f64>,
    tfthet: Array1<'static, f64>,
}

impl ConstraintTables {
    // Pass, tangency and curvature points together.
    fn nb_constr(&self) -> i32 {
        self.nb_pass_points + self.nb_tang_points + self.nb_curv_points
    }

    // Standard_OutOfRange outside the table, as the native checked Value.
    fn typ(&self, index: i32) -> Result<i32, Failure> {
        self.types
            .value(index)
            .copied()
            .map_err(|error| array_failure(error, "AppDef_Variational: myTypConstraints"))
    }

    fn value(&self, index: i32) -> Result<f64, Failure> {
        self.values
            .value(index)
            .copied()
            .map_err(|error| array_failure(error, "AppDef_Variational: myTabConstraints"))
    }
}

/// The results of an approximation.
#[derive(Clone, Default)]
struct VariationalResults {
    // myCriterium: the quadratic error, then the tension, flexion and jerk criteria.
    criterium: [f64; 4],
    mbsp_curve: MultiBSpCurve,
    max_error: f64,
    max_error_index: i32,
    average_error: f64,
}

/// `AppDef_Variational`: approximates the points `FirstPoint..=LastPoint` of a
/// MultiLine by a B-spline MultiCurve minimizing the distance to the points plus the
/// tension, flexion and jerk of the curve ([`LinearCriteria`]), under pass, tangency
/// and curvature constraints.
///
/// The native constructor leaves the errors and the criteria uninitialized; the
/// port starts them at 0.
///
/// ```
/// use rustycad_tkgeombase::app_def::{MultiLine, Variational, VariationalOptions};
/// use rustycad_tkgeombase::app_par_curves::{Constraint, ConstraintCouple};
/// use rustycad_tkernel::ncollection::{Array1, HArray1};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::Point3;
/// let points: Vec<Point3> = (0..8).map(|i| {
///     let t = f64::from(i) / 7.0;
///     Point3::new(t, t * t, 0.0)
/// }).collect();
/// let line = MultiLine::from_points(&Array1::from_shared(&points, 1));
/// let ends = [ConstraintCouple::new(1, Constraint::PassPoint), ConstraintCouple::new(8, Constraint::PassPoint)];
/// let constraints = Handle::new(HArray1::from_array(&Array1::from_shared(&ends, 1)));
/// // A size-error tolerance of 1e-6 weights the distance to the points heavily.
/// let options = VariationalOptions { tolerance: 1e-6, ..VariationalOptions::default() };
/// let mut smoothing = Variational::new(&line, 1, 8, &constraints, options)?;
/// smoothing.approximate()?;
/// assert!(smoothing.is_done());
/// assert!(smoothing.max_error()? < 1e-3);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct Variational {
    ssp: MultiLine,
    nb_p3d: i32,
    nb_p2d: i32,
    dimension: i32,
    first_point: i32,
    last_point: i32,
    nb_points: i32,
    // myTabPoints: the coordinates of every point, Dimension values per point.
    tab_points: Array1<'static, f64>,
    constraints: Constraints,
    tables: ConstraintTables,
    options: VariationalOptions,
    niv_cont: i32,
    percent: [f64; 3],
    smooth_criterion: Handle<dyn SmoothCriterion>,
    parameters: Handle<HArray1<f64>>,
    knots: Handle<HArray1<f64>>,
    results: VariationalResults,
    is_created: bool,
    is_done: bool,
    is_over_constr: bool,
}

// Standard_DomainError of an argument outside its range.
fn domain(detail: &str) -> Failure {
    failure(
        "Standard_DomainError",
        &format!("AppDef_Variational: {detail}"),
    )
}

// Standard_ConstructionError of the constructor, Init and the setters.
fn construction(detail: &str) -> Failure {
    failure(
        "Standard_ConstructionError",
        &format!("AppDef_Variational: {detail}"),
    )
}

// C0/C1/C2 as 0/1/2; Standard_ConstructionError for another continuity.
fn niv_cont_of(continuity: Shape) -> Result<i32, Failure> {
    match continuity {
        Shape::C0 => Ok(0),
        Shape::C1 => Ok(1),
        Shape::C2 => Ok(2),
        other => Err(construction(&format!(
            "continuity {other:?}; expected C0, C1 or C2"
        ))),
    }
}

// The argument checks of the constructor, before any field depends on the line.
fn checked_options(options: VariationalOptions) -> Result<VariationalOptions, Failure> {
    if options.max_degree < 1 {
        return Err(domain(&format!(
            "MaxDegree {}; expected >= 1",
            options.max_degree
        )));
    }
    if options.max_segment < 1 {
        return Err(domain(&format!(
            "MaxSegment {}; expected >= 1",
            options.max_segment
        )));
    }
    Ok(VariationalOptions {
        max_degree: options.max_degree.min(30),
        ..options
    })
}

impl Variational {
    /// `AppDef_Variational(SSP, FirstPoint, LastPoint, TheConstraints, MaxDegree,
    /// MaxSegment, Continuity, WithMinMax, WithCutting, Tolerance, NbIterations)`:
    /// reads the points, builds the constraint tables and, unless the problem is
    /// over-constrained (`(MaxDegree - Continuity) * MaxSegment - Nc0 - 2 Nc1 -
    /// 3 Nc2 < 0`), the initial smoothing criterion. Standard_DomainError for a bad
    /// degree or segment count, Standard_ConstructionError for a bad continuity, no
    /// point, a null chord length or an inconsistent constraint;
    /// `Variational::new(&line, 1, 20, &constraints, VariationalOptions::default())?`.
    pub fn new(
        ssp: &MultiLine,
        first_point: i32,
        last_point: i32,
        the_constraints: &Constraints,
        options: VariationalOptions,
    ) -> Result<Self, Failure> {
        let options: VariationalOptions = checked_options(options)?;
        let niv_cont: i32 = niv_cont_of(options.continuity)?;
        let (nb_p2d, nb_p3d) = (ssp.nb_p2d()?, ssp.nb_p3d()?);
        let knots: Array1<'static, f64> = Array1::from_shared(&[0., 1.], 1).clone();
        let criterion: LinearCriteria = LinearCriteria::new(ssp, first_point, last_point)?;
        let parameters: HArray1<f64> = HArray1::with_bounds(first_point, last_point)
            .map_err(|error| allocation_failure(error, "AppDef_Variational"))?;
        let nb_points: i32 = last_point - first_point + 1;
        if nb_points <= 0 {
            return Err(construction(&format!(
                "{nb_points} points; expected FirstPoint <= LastPoint"
            )));
        }
        let mut variational: Self = Self {
            ssp: ssp.clone(),
            nb_p3d,
            nb_p2d,
            dimension: 2 * nb_p2d + 3 * nb_p3d,
            first_point,
            last_point,
            nb_points,
            tab_points: Array1::default(),
            constraints: the_constraints.clone(),
            tables: ConstraintTables::default(),
            options,
            niv_cont,
            percent: [0.4, 0.2, 0.4],
            smooth_criterion: Handle::new(criterion).cast(),
            parameters: Handle::new(parameters),
            knots: Handle::new(HArray1::from_owned(knots)),
            results: VariationalResults::default(),
            is_created: false,
            is_done: false,
            is_over_constr: false,
        };
        variational.tab_points = variational.read_points()?;
        variational.init()?;
        Ok(variational)
    }

    // Table of Points initialization: the 3D then 2D coordinates of every point.
    fn read_points(&self) -> Result<Array1<'static, f64>, Failure> {
        let mut tab_points: Array1<'static, f64> =
            Array1::with_bounds(1, self.dimension * self.nb_points)
                .map_err(|error| allocation_failure(error, "AppDef_Variational"))?;
        let mut tabs: PointTabs = PointTabs::new([self.nb_p3d, self.nb_p2d])?;
        let mut index: i32 = 1;
        for ipoint in self.first_point..=self.last_point {
            if self.nb_p2d == 0 && self.nb_p3d != 0 {
                tabs.read_3d(&self.ssp, ipoint)?;
            } else if self.nb_p2d != 0 {
                tabs.read_2d(&self.ssp, ipoint)?;
            } else {
                continue;
            }
            self.push_coordinates(&tabs, &mut tab_points, &mut index)?;
        }
        Ok(tab_points)
    }

    fn push_coordinates(
        &self,
        tabs: &PointTabs,
        tab_points: &mut Array1<'static, f64>,
        index: &mut i32,
    ) -> Result<(), Failure> {
        let mut push = |value: f64| -> Result<(), Failure> {
            tab_points
                .set_value(*index, value)
                .map_err(|error| array_failure(error, "AppDef_Variational: myTabPoints"))?;
            *index += 1;
            Ok(())
        };
        for jp3d in 1..=self.nb_p3d {
            let p = tabs.p3d[jp3d];
            [p.x(), p.y(), p.z()].into_iter().try_for_each(&mut push)?;
        }
        for jp2d in 1..=self.nb_p2d {
            let p = tabs.p2d[jp2d];
            [p.x(), p.y()].into_iter().try_for_each(&mut push)?;
        }
        Ok(())
    }

    // myTabPoints(index), checked.
    fn tab_point(&self, index: i32) -> Result<f64, Failure> {
        self.tab_points
            .value(index)
            .copied()
            .map_err(|error| array_failure(error, "AppDef_Variational: myTabPoints"))
    }

    // myParameters(index), checked.
    fn parameter(&self, index: i32) -> Result<f64, Failure> {
        let parameters = self.parameters.array1();
        parameters
            .value(index)
            .copied()
            .map_err(|error| array_failure(error, "AppDef_Variational: myParameters"))
    }

    // The over-constraint test with `max_segment` segments of degree `max_degree`.
    fn over_constrained(&self, max_degree: i32, niv_cont: i32, max_segment: i32) -> bool {
        let t: &ConstraintTables = &self.tables;
        (max_degree - niv_cont) * max_segment
            - t.nb_pass_points
            - 2 * t.nb_tang_points
            - 3 * t.nb_curv_points
            < 0
    }

    // The smoothing criterion (never null once constructed).
    fn criterion(&self) -> Result<&dyn SmoothCriterion, Failure> {
        criterion_ref(&self.smooth_criterion)
    }
}

// The criterion behind a handle the native code never leaves null.
fn criterion_ref(j: &Handle<dyn SmoothCriterion>) -> Result<&dyn SmoothCriterion, Failure> {
    j.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "AppDef_Variational: null smoothing criterion; expected the LinearCriteria",
        )
    })
}

// StdFail_NotDone of a result member before a successful Approximate.
fn not_done(member: &str) -> Failure {
    failure(
        "StdFail_NotDone",
        &format!("AppDef_Variational::{member}: no approximation done"),
    )
}
