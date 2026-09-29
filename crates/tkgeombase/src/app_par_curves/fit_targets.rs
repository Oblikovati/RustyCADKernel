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
// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_Function.gxx, AppParCurves_BSpFunction.gxx (shared parts)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parts `AppParCurves_Function` and `AppParCurves_BSpFunction` share (the two
//! native files repeat them): the constraint classification, the line points of
//! the constrained branch, and the optimizer callbacks.
use super::Constraints;
use super::arrays::{line_points, scratch};
use super::constraint::Constraint;
use super::line_tool::LineTool;
use super::resol_constraint::couples;
use crate::native_failure::array_failure;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::math::Matrix;

/// `Contraintes`, `Adeb`, `Afin`, `tabdim` and `PTLX`/`PTLY`/`PTLZ`.
pub(crate) struct FitTargets {
    pub(crate) contraintes: bool,
    pub(crate) adeb: i32,
    pub(crate) afin: i32,
    pub(crate) tabdim: Vec<i32>,
    pub(crate) ptl: [Matrix<'static>; 3],
}

impl FitTargets {
    /// Adeb/Afin skip the constrained extremities; a constraint on any other point
    /// makes the function constrained, and then the line points are read (the 2D
    /// points at the curve index, not the index minus the 3D curves, as natively).
    pub(crate) fn new(
        ssp: &impl LineTool,
        [first_p, last_p]: [i32; 2],
        constraints: &Constraints,
        context: &str,
    ) -> Result<Self, Failure> {
        let curves: i32 = ssp.nb_p3d()? + ssp.nb_p2d()?;
        let new = || {
            Matrix::filled(first_p, last_p, 1, curves, 0.)
                .map_err(|error| array_failure(error, context))
        };
        let mut targets: Self = Self {
            contraintes: false,
            adeb: first_p,
            afin: last_p,
            tabdim: vec![0; curves.max(0) as usize],
            ptl: [new()?, new()?, new()?],
        };
        for couple in couples(constraints)? {
            let constrained: bool = couple.constraint() >= Constraint::PassPoint;
            if couple.index() == first_p {
                targets.adeb += i32::from(constrained);
            } else if couple.index() == last_p {
                targets.afin -= i32::from(constrained);
            } else if constrained {
                targets.contraintes = true;
            }
        }
        if targets.contraintes {
            targets.read_points(ssp, [first_p, last_p], context)?;
        }
        Ok(targets)
    }

    fn read_points(
        &mut self,
        ssp: &impl LineTool,
        [first_p, last_p]: [i32; 2],
        context: &str,
    ) -> Result<(), Failure> {
        let nb3d: i32 = ssp.nb_p3d()?;
        for (j, dimension) in self.tabdim.iter_mut().enumerate() {
            *dimension = if (j as i32) < nb3d { 3 } else { 2 };
        }
        let (mut points, mut points2d) =
            (scratch::<Point3>(nb3d)?, scratch::<Point2>(ssp.nb_p2d()?)?);
        for i in first_p..=last_p {
            line_points(ssp, i, &mut points, &mut points2d)?;
            for j in 1..=self.tabdim.len() as i32 {
                if self.tabdim[(j - 1) as usize] == 3 {
                    let p: Point3 = *points
                        .value(j)
                        .map_err(|error| array_failure(error, context))?;
                    (
                        self.ptl[0][(i, j)],
                        self.ptl[1][(i, j)],
                        self.ptl[2][(i, j)],
                    ) = (p.x(), p.y(), p.z());
                } else {
                    let p: Point2 = *points2d
                        .value(j)
                        .map_err(|error| array_failure(error, context))?;
                    (self.ptl[0][(i, j)], self.ptl[1][(i, j)]) = (p.x(), p.y());
                }
            }
        }
        Ok(())
    }
}

/// The math_MultipleVarFunctionWithGradient callbacks and the BFGS criterion of a
/// parameter function with `evaluate`, `evaluate_gradient`, `evaluate_values`,
/// `nb_variables`, and the fields `failure`, `err3d`, `err2d`. A native exception
/// unwinds through math_BFGS; a callback cannot return one, so it keeps the first
/// in `failure` (see `take_failure`) and reports false.
macro_rules! impl_par_function {
    ($function:ident) => {
        impl<L: LineTool + Clone> $function<L> {
            /// The first exception raised inside an optimizer callback;
            /// `f.take_failure()`.
            pub fn take_failure(&mut self) -> Option<Failure> {
                self.failure.take()
            }

            // The callback result: false after a kept exception.
            fn keep(&mut self, result: Result<bool, Failure>) -> bool {
                if self.failure.is_some() {
                    return false;
                }
                result.unwrap_or_else(|error| {
                    self.failure = Some(error);
                    false
                })
            }
        }

        impl<L: LineTool + Clone> rustycad_tkmath::math::MultipleVarFunction for $function<L> {
            fn variables(&self) -> i32 {
                self.nb_variables()
            }

            fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
                if self.failure.is_some() {
                    return false;
                }
                let result = self.evaluate(point, value);
                self.keep(result)
            }
        }

        impl<L: LineTool + Clone> rustycad_tkmath::math::MultipleVarFunctionWithGradient
            for $function<L>
        {
            fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
                if self.failure.is_some() {
                    return false;
                }
                let result = self.evaluate_gradient(point, gradient);
                self.keep(result)
            }

            fn values(
                &mut self,
                point: &Vector<'_>,
                value: &mut f64,
                gradient: &mut Vector<'_>,
            ) -> bool {
                if self.failure.is_some() {
                    return false;
                }
                let result = self.evaluate_values(point, value, gradient);
                self.keep(result)
            }
        }

        impl<L: LineTool + Clone> super::gradient_bfgs::FitFunction for $function<L> {
            fn fit_error_3d(&self) -> f64 {
                self.err3d
            }

            fn fit_error_2d(&self) -> f64 {
                self.err2d
            }
        }
    };
}
pub(crate) use impl_par_function;
