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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_ComputeLine.gxx (fields, constructors, accessors)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_ComputeLine`: the approximation of a MultiLine by Bezier MultiCurves,
//! raising the degree and cutting the line until the tolerances are reached.
mod fit;
mod interpolate;
mod perform;

use super::compute_c_line::sequence_value;
use super::mcurves_to_bsp_curve::MCurvesToBSpCurve;
use super::status::ParametrizationType;
use crate::app_par_curves::{Constraint, ConstraintCouple, Constraints, MultiBSpCurve, MultiCurve};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::math::Vector;

/// The constructor arguments of `Approx_ComputeLine` and `Approx_BSplComputeLine`,
/// with the native defaults (`degreemin = 4`, `degreemax = 8`, `Tolerance3d =
/// 1.0e-3`, `Tolerance2d = 1.0e-6`, `NbIterations = 5`, `cutting = true`,
/// `Approx_ChordLength`, `Squares = false`). The constructors that take initial
/// parameters ignore `parametrization` and use `Approx_IsoParametric`, as natively.
///
/// ```
/// use rustycad_tkgeombase::approx::ComputeLineOptions;
/// let options = ComputeLineOptions { degree_max: 10, ..ComputeLineOptions::default() };
/// assert_eq!(options.nb_iterations, 5);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComputeLineOptions {
    /// `degreemin`.
    pub degree_min: i32,
    /// `degreemax`.
    pub degree_max: i32,
    /// `Tolerance3d`.
    pub tolerance_3d: f64,
    /// `Tolerance2d`.
    pub tolerance_2d: f64,
    /// `NbIterations`: the BFGS iterations of the parameter optimization.
    pub nb_iterations: i32,
    /// `cutting`: whether the line may be cut.
    pub cutting: bool,
    /// `parametrization`.
    pub parametrization: ParametrizationType,
    /// `Squares`: a plain least-squares fit, without parameter optimization.
    pub squares: bool,
}

impl Default for ComputeLineOptions {
    fn default() -> Self {
        Self {
            degree_min: 4,
            degree_max: 8,
            tolerance_3d: 1.0e-3,
            tolerance_2d: 1.0e-6,
            nb_iterations: 5,
            cutting: true,
            parametrization: ParametrizationType::ChordLength,
            squares: false,
        }
    }
}

/// `Approx_ComputeLine` over any [`crate::app_par_curves::LineTool`] (AppDef_Compute,
/// GeomInt_TheComputeLineBezierOfWLApprox...): the Bezier pieces, their parameters
/// and errors.
///
/// The native constructors leave `tolreached` and the current errors
/// uninitialized; the port starts them at false and 0.
///
/// ```
/// use rustycad_tkgeombase::app_def::MultiLine;
/// use rustycad_tkgeombase::approx::{ComputeLine, ComputeLineOptions};
/// use rustycad_tkernel::ncollection::Array1;
/// use rustycad_tkmath::gp::Point2;
/// let points: Vec<Point2> = (0..12).map(|i| {
///     let t = f64::from(i) / 11.0;
///     Point2::new(t, (3.0 * t).sin())
/// }).collect();
/// let line = MultiLine::from_points2d(&Array1::from_shared(&points, 1));
/// let compute = ComputeLine::with_line(&line, ComputeLineOptions::default())?;
/// assert!(compute.is_all_approximated());
/// assert!(compute.nb_multi_curves() >= 1);
/// let (tol3d, _) = compute.error(1)?;
/// assert_eq!(tol3d, 0.0); // no 3D curve
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct ComputeLine {
    multi_curves: Vec<MultiCurve>,
    the_multi_curve: MultiCurve,
    spline: MultiBSpCurve,
    alldone: bool,
    tolreached: bool,
    par: ParametrizationType,
    my_parameters: Option<Array1<'static, f64>>,
    first_param: Option<Array1<'static, f64>>,
    par_seq: Vec<Array1<'static, f64>>,
    tolers3d: Vec<f64>,
    tolers2d: Vec<f64>,
    couples: [ConstraintCouple; 2],
    options: ComputeLineOptions,
    currenttol3d: f64,
    currenttol2d: f64,
    first_c: Constraint,
    last_c: Constraint,
    multi_line_nb: i32,
    is_clear: bool,
}

/// An owned copy of `values` with its bounds (the native HArray1 copies).
pub(crate) fn owned_array(values: &Vector<'_>) -> Result<Array1<'static, f64>, Failure> {
    let mut copy: Array1<'static, f64> = Array1::with_bounds(values.lower(), values.upper())
        .map_err(|e| crate::native_failure::array_failure(e, "Approx"))?;
    for i in values.lower()..=values.upper() {
        copy[i] = *values.value(i)?;
    }
    Ok(copy)
}

impl ComputeLine {
    /// The constructor without a line (`degreemin`... `Squares`): initializes the
    /// fields only; `ComputeLine::new(ComputeLineOptions::default())`.
    pub fn new(options: ComputeLineOptions) -> Self {
        Self {
            multi_curves: Vec::new(),
            the_multi_curve: MultiCurve::new(),
            spline: MultiBSpCurve::new(),
            alldone: false,
            tolreached: false,
            par: options.parametrization,
            my_parameters: None,
            first_param: None,
            par_seq: Vec::new(),
            tolers3d: Vec::new(),
            tolers2d: Vec::new(),
            couples: [ConstraintCouple::default(); 2],
            options,
            currenttol3d: 0.0,
            currenttol2d: 0.0,
            first_c: Constraint::TangencyPoint,
            last_c: Constraint::TangencyPoint,
            multi_line_nb: 0,
            is_clear: false,
        }
    }

    /// The constructor with a line: initializes and performs;
    /// `ComputeLine::with_line(&line, ComputeLineOptions::default())?`.
    pub fn with_line<L: crate::app_par_curves::LineTool + Clone>(
        line: &L,
        options: ComputeLineOptions,
    ) -> Result<Self, Failure> {
        let mut compute: Self = Self::new(options);
        compute.perform(line)?;
        Ok(compute)
    }

    /// The constructor with initial parameters and no line (iso-parametric; the
    /// parameters are used on the first piece only);
    /// `ComputeLine::from_parameters(&u, ComputeLineOptions::default())?`.
    pub fn from_parameters(
        parameters: &Vector<'_>,
        options: ComputeLineOptions,
    ) -> Result<Self, Failure> {
        let mut compute: Self = Self::new(options);
        compute.first_param = Some(owned_array(parameters)?);
        compute.par = ParametrizationType::IsoParametric;
        Ok(compute)
    }

    /// The constructor with a line and initial parameters: initializes and performs;
    /// `ComputeLine::with_line_and_parameters(&line, &u, ComputeLineOptions::default())?`.
    pub fn with_line_and_parameters<L: crate::app_par_curves::LineTool + Clone>(
        line: &L,
        parameters: &Vector<'_>,
        options: ComputeLineOptions,
    ) -> Result<Self, Failure> {
        let mut compute: Self = Self::from_parameters(parameters, options)?;
        compute.perform(line)?;
        Ok(compute)
    }

    /// `Init(degreemin, ..., Squares)`: replaces every option;
    /// `compute.init(ComputeLineOptions::default())`.
    pub fn init(&mut self, options: ComputeLineOptions) {
        self.options = options;
        self.par = options.parametrization;
    }

    /// `SetDegrees(degreemin, degreemax)`; `compute.set_degrees(3, 6)`.
    pub fn set_degrees(&mut self, degree_min: i32, degree_max: i32) {
        self.options.degree_min = degree_min;
        self.options.degree_max = degree_max;
    }

    /// `SetTolerances(Tolerance3d, Tolerance2d)`; `compute.set_tolerances(1e-4, 1e-6)`.
    pub fn set_tolerances(&mut self, tolerance_3d: f64, tolerance_2d: f64) {
        self.options.tolerance_3d = tolerance_3d;
        self.options.tolerance_2d = tolerance_2d;
    }

    /// `SetConstraints(firstC, lastC)`: the constraints of the first and last point
    /// of every piece; `compute.set_constraints(Constraint::PassPoint, Constraint::PassPoint)`.
    pub fn set_constraints(&mut self, first: Constraint, last: Constraint) {
        self.first_c = first;
        self.last_c = last;
    }

    /// `IsAllApproximated()`; `compute.is_all_approximated()`.
    pub fn is_all_approximated(&self) -> bool {
        self.alldone
    }

    /// `IsToleranceReached()`; `compute.is_tolerance_reached()`.
    pub fn is_tolerance_reached(&self) -> bool {
        self.tolreached
    }

    /// `Error(Index, tol3d, tol2d)`: the errors of piece `index`;
    /// Standard_OutOfRange outside `1..=NbMultiCurves()`; `compute.error(1)?`.
    pub fn error(&self, index: i32) -> Result<(f64, f64), Failure> {
        Ok((
            sequence_value(&self.tolers3d, index)?,
            sequence_value(&self.tolers2d, index)?,
        ))
    }

    /// `NbMultiCurves()`; `compute.nb_multi_curves()`.
    pub fn nb_multi_curves(&self) -> i32 {
        self.multi_curves.len() as i32
    }

    /// `Value(Index)`: piece `index`; Standard_OutOfRange outside
    /// `1..=NbMultiCurves()`; `compute.value(1)?`.
    pub fn value(&self, index: i32) -> Result<&MultiCurve, Failure> {
        sequence_slot(&self.multi_curves, index).map(|slot| &self.multi_curves[slot])
    }

    /// `ChangeValue(Index)`: piece `index`, to modify; `compute.change_value(1)?`.
    pub fn change_value(&mut self, index: i32) -> Result<&mut MultiCurve, Failure> {
        sequence_slot(&self.multi_curves, index).map(|slot| &mut self.multi_curves[slot])
    }

    /// `SplineValue()`: the pieces joined by `Approx_MCurvesToBSpCurve`;
    /// `compute.spline_value()?`.
    pub fn spline_value(&mut self) -> Result<&MultiBSpCurve, Failure> {
        let mut join: MCurvesToBSpCurve = MCurvesToBSpCurve::new();
        join.perform_sequence(&self.multi_curves)?;
        self.spline = join.value().clone();
        Ok(&self.spline)
    }

    /// `Parametrization()`; `compute.parametrization()`.
    pub fn parametrization(&self) -> ParametrizationType {
        self.par
    }

    /// `Parameters(Index)`: the parameters of the points of piece `index`;
    /// `compute.parameters(1)?`.
    pub fn parameters(&self, index: i32) -> Result<&Array1<'static, f64>, Failure> {
        sequence_slot(&self.par_seq, index).map(|slot| &self.par_seq[slot])
    }

    // myConstraints: the couples of the current piece, with the stored constraints.
    fn set_couples(&mut self, first: i32, last: i32) {
        self.couples = [
            ConstraintCouple::new(first, self.first_c),
            ConstraintCouple::new(last, self.last_c),
        ];
    }

    fn constraints(&self) -> Constraints {
        constraint_handle(self.couples)
    }
}

/// `myConstraints`: the two extremity couples as the handle the gradients take.
pub(crate) fn constraint_handle([first, last]: [ConstraintCouple; 2]) -> Constraints {
    let mut couples: Array1<'static, ConstraintCouple> = Array1::with_size(2);
    couples.update_lower_bound(1);
    (couples[1], couples[2]) = (first, last);
    Handle::new(HArray1::from_owned(couples))
}

/// The slot of `NCollection_Sequence::Value(Index)`: Standard_OutOfRange outside
/// `1..=Length()`.
pub(crate) fn sequence_slot<T>(sequence: &[T], index: i32) -> Result<usize, Failure> {
    usize::try_from(index - 1)
        .ok()
        .filter(|&slot| slot < sequence.len())
        .ok_or_else(|| {
            Failure::out_of_range(&format!(
                "NCollection_Sequence::Value: index {index}; expected 1..={}",
                sequence.len()
            ))
        })
}
