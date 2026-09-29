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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_Gradient_BFGSOfTheGradient.hxx/_0.cxx (the AppParCurves_Gradient_BFGS parameter)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `AppParCurves_Gradient_BFGS` and `AppParCurves_BSpGradient_BFGS` template
//! parameters of the gradients (instantiated natively as AppDef/GeomInt/BRepApprox
//! `*_BFGSOf*` classes): math_BFGS stopping on the fit tolerances too.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Bfgs, MultipleVarFunctionWithGradient, Vector};

/// The fit errors a gradient function exposes to its BFGS criterion
/// (`MaxError3d()`, `MaxError2d()` of `AppParCurves_Function` and `_BSpFunction`).
pub trait FitFunction: MultipleVarFunctionWithGradient {
    /// `MaxError3d()` of the last evaluation.
    fn fit_error_3d(&self) -> f64;
    /// `MaxError2d()` of the last evaluation.
    fn fit_error_2d(&self) -> f64;
}

/// `AppParCurves_Gradient_BFGS`: `math_BFGS(NbVariables, Eps, NbIterations, Eps)`
/// with `IsSolutionReached` true when the minimum is stable
/// (`2 |F - Fprev| <= 1e-10 (|F| + |Fprev|) + 1e-12`) or both fit errors are within
/// the tolerances.
///
/// ```
/// use rustycad_tkgeombase::app_def::MultiLine;
/// use rustycad_tkgeombase::app_par_curves::{Function, GradientBfgs};
/// # use rustycad_tkgeombase::app_par_curves::{Constraint, ConstraintCouple};
/// # use rustycad_tkernel::ncollection::{Array1, HArray1};
/// # use rustycad_tkernel::standard::Handle;
/// # use rustycad_tkmath::gp::Point3;
/// # use rustycad_tkmath::math::Vector;
/// // Six points of the parabola y = x^2, AppDef_MultiLine read by AppDef_MyLineTool.
/// let points: Vec<Point3> = (0..6).map(|i| {
///     let t = f64::from(i) / 5.0;
///     Point3::new(t, t * t, 0.0)
/// }).collect();
/// let line = MultiLine::from_points(&Array1::from_shared(&points, 1));
/// # let ends = [ConstraintCouple::new(1, Constraint::PassPoint), ConstraintCouple::new(6, Constraint::PassPoint)];
/// # let constraints = Handle::new(HArray1::from_array(&Array1::from_shared(&ends, 1)));
/// # let mut u = Vector::with_bounds(1, 6)?;
/// # (1..=6).for_each(|i| u[i] = f64::from(i - 1) / 5.0);
/// let mut function = Function::new(&line, 1, 6, &constraints, &u, 3)?;
/// let bfgs = GradientBfgs::new(&mut function, &u, 1e-3, 1e-6, 1e-7, 200)?;
/// assert!(bfgs.is_solution_reached(&function));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct GradientBfgs {
    bfgs: Bfgs,
    tol3d: f64,
    tol2d: f64,
}

impl GradientBfgs {
    /// Builds the minimizer and runs it from `starting_point` (the native constructor
    /// performs); `GradientBfgs::new(&mut f, &u, 1e-3, 1e-6, 1e-7, 200)?`.
    pub fn new<F: FitFunction>(
        function: &mut F,
        starting_point: &Vector<'_>,
        tolerance3d: f64,
        tolerance2d: f64,
        eps: f64,
        nb_iterations: i32,
    ) -> Result<Self, Failure> {
        let mut bfgs: Bfgs = Bfgs::with_options(function.variables(), eps, nb_iterations, eps)?;
        bfgs.perform_with_criterion(function, starting_point, &mut |solver: &Bfgs, f: &mut F| {
            solution_reached(solver, f, [tolerance3d, tolerance2d])
        })?;
        Ok(Self {
            bfgs,
            tol3d: tolerance3d,
            tol2d: tolerance2d,
        })
    }

    /// `IsSolutionReached(F)`; `bfgs.is_solution_reached(&f)`.
    pub fn is_solution_reached<F: FitFunction>(&self, function: &F) -> bool {
        solution_reached(&self.bfgs, function, [self.tol3d, self.tol2d])
    }

    /// The underlying math_BFGS; `bfgs.bfgs().is_done()`.
    pub fn bfgs(&self) -> &Bfgs {
        &self.bfgs
    }
}

fn solution_reached<F: FitFunction>(solver: &Bfgs, function: &F, [tol3d, tol2d]: [f64; 2]) -> bool {
    let (minimum, previous) = (solver.current_minimum(), solver.previous_minimum());
    let stable: bool =
        2.0 * (minimum - previous).abs() <= 1.0e-10 * (minimum.abs() + previous.abs()) + 1.0e-12;
    let within: bool = function.fit_error_3d() <= tol3d && function.fit_error_2d() <= tol2d;
    stable || within
}
