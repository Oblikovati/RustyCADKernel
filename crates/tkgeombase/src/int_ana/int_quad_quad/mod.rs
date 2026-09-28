// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of IntAna_IntQuadQuad.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna_IntQuadQuad: the analytic intersection of a cylinder or a cone with a
//! quadric, as [`Curve`] pieces. The state and the accessors are here, the
//! cylinder and cone algorithms in their modules, the curve linking in `links`.
mod cone;
mod cylinder;
mod links;
mod trig;

use super::{Curve, Quadric};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Cone3, Cylinder3, Point3};

/// The capacity of the native curve arrays (myNbMaxCurves).
const MAX_CURVES: usize = 12;

/// IntAna_IntQuadQuad: the curves (and contact points) of the intersection of a
/// cylinder or a cone with a [`Quadric`], used when IntAna_QuadQuadGeo has no
/// geometric solution.
///
/// `let inter = IntQuadQuad::cylinder(cylinder, &Quadric::from_sphere(sphere), 1e-7)?;`
#[derive(Clone, Copy, Debug)]
pub struct IntQuadQuad {
    done: bool,
    identical: bool,
    the_curve: [Curve; MAX_CURVES],
    previouscurve: [i32; MAX_CURVES],
    nextcurve: [i32; MAX_CURVES],
    nb_curves: i32,
    nbpoints: i32,
    thepoints: [Point3; 2],
    my_nb_max_curves: i32,
    my_epsilon: f64,
    my_epsilon_coeff_poly_null: f64,
}

impl Default for IntQuadQuad {
    /// IntAna_IntQuadQuad(): not done, no curve, the native epsilons (1e-8).
    fn default() -> Self {
        Self {
            done: false,
            identical: false,
            the_curve: [Curve::default(); MAX_CURVES],
            previouscurve: [0; MAX_CURVES],
            nextcurve: [0; MAX_CURVES],
            nb_curves: 0,
            nbpoints: 0,
            thepoints: [Point3::default(); 2],
            my_nb_max_curves: MAX_CURVES as i32,
            my_epsilon: 0.00000001,
            my_epsilon_coeff_poly_null: 0.00000001,
        }
    }
}

impl IntQuadQuad {
    /// IntAna_IntQuadQuad(): the empty constructor. `IntQuadQuad::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// IntAna_IntQuadQuad(gp_Cylinder, IntAna_Quadric, Tol); the tolerance is
    /// unused, as natively. `IntQuadQuad::cylinder(cyl, &quadric, 1e-7)?`.
    pub fn cylinder(cylinder: Cylinder3, quadric: &Quadric, tol: f64) -> Result<Self, Failure> {
        let mut inter: Self = Self::default();
        inter.perform_cylinder(cylinder, quadric, tol)?;
        Ok(inter)
    }

    /// IntAna_IntQuadQuad(gp_Cone, IntAna_Quadric, Tol); the tolerance is unused,
    /// as natively. `IntQuadQuad::cone(cone, &quadric, 1e-7)?`.
    pub fn cone(cone: Cone3, quadric: &Quadric, tol: f64) -> Result<Self, Failure> {
        let mut inter: Self = Self::default();
        inter.perform_cone(cone, quadric, tol)?;
        Ok(inter)
    }

    /// IsDone(): true when the computation succeeded. `inter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// IdenticalElements(): the elements are identical (never set by the
    /// algorithms); StdFail_NotDone when not done. `inter.identical_elements()?`.
    pub fn identical_elements(&self) -> Result<bool, Failure> {
        self.check_done()?;
        Ok(self.identical)
    }

    /// NbCurve(): the number of curves; StdFail_NotDone when not done,
    /// Standard_DomainError for identical elements. `inter.nb_curve()?`.
    pub fn nb_curve(&self) -> Result<i32, Failure> {
        self.check_result()?;
        Ok(self.nb_curves)
    }

    /// NbPnt(): the number of contact points (the algorithms find none), with the
    /// errors of [`Self::nb_curve`]. `inter.nb_pnt()?`.
    pub fn nb_pnt(&self) -> Result<i32, Failure> {
        self.check_result()?;
        Ok(self.nbpoints)
    }

    /// Curve(N): the curve of range `n`, with the errors of [`Self::nb_curve`] and
    /// Standard_OutOfRange outside `1..=NbCurve()`. `inter.curve(1)?`.
    pub fn curve(&self, n: i32) -> Result<&Curve, Failure> {
        Ok(&self.the_curve[self.check_curve(n, "Incorrect Curve Number")?])
    }

    /// Point(N): the contact point of range `n`, with the errors of
    /// [`Self::nb_curve`] and Standard_OutOfRange outside `1..=NbPnt()`.
    /// `inter.point(1)?`.
    pub fn point(&self, n: i32) -> Result<Point3, Failure> {
        self.check_result()?;
        if n <= 0 || n > self.nbpoints {
            return Err(Failure::out_of_range(&format!(
                "Incorrect Point Number {n}; expected 1..={}",
                self.nbpoints
            )));
        }
        Ok(self.thepoints[(n - 1) as usize])
    }

    /// Parameters(N, U1, U2): not implemented natively; the native member only
    /// prints "IntAna_IntQuadQuad::Parameters(...) is not yet implemented" on
    /// std::cout and leaves `u1` and `u2` unchanged. The port leaves them
    /// unchanged without printing. `inter.parameters(1, &mut u1, &mut u2)`.
    pub fn parameters(&self, _n: i32, _u1: &mut f64, _u2: &mut f64) {}

    fn check_done(&self) -> Result<(), Failure> {
        if self.done {
            return Ok(());
        }
        Err(Failure::with_type(
            "StdFail_NotDone",
            "IntQuadQuad Not done",
            "",
        ))
    }

    fn check_result(&self) -> Result<(), Failure> {
        self.check_done()?;
        if self.identical {
            return Err(Failure::with_type(
                "Standard_DomainError",
                "IntQuadQuad identical",
                "",
            ));
        }
        Ok(())
    }

    /// The 0-based slot of curve `n` after the done, identical and range checks.
    fn check_curve(&self, n: i32, message: &str) -> Result<usize, Failure> {
        self.check_result()?;
        if n > self.nb_curves || n <= 0 {
            return Err(Failure::out_of_range(&format!(
                "{message} {n}; expected 1..={}",
                self.nb_curves
            )));
        }
        Ok((n - 1) as usize)
    }

    /// The next free curve slot, counted in NbCurve(); its curve is then set in
    /// place (a Set*QuadValues keeps the fields it does not assign, as natively).
    /// Standard_OutOfRange when the native array of 12 curves would overflow
    /// (undefined behaviour natively).
    fn new_curve_slot(&mut self) -> Result<&mut Curve, Failure> {
        let slot: usize = self.nb_curves as usize;
        if slot >= MAX_CURVES {
            return Err(Failure::out_of_range(&format!(
                "IntAna_IntQuadQuad - curve {} exceeds the {MAX_CURVES} curve slots",
                slot + 1
            )));
        }
        self.nb_curves += 1;
        Ok(&mut self.the_curve[slot])
    }

    /// Clears the result before a Perform: done, not identical, no curve, no
    /// point, and every link of the curve slots.
    fn reset(&mut self) {
        self.done = true;
        self.identical = false;
        self.nb_curves = 0;
        self.nbpoints = 0;
        let count: usize = self.my_nb_max_curves as usize;
        self.previouscurve[..count].fill(0);
        self.nextcurve[..count].fill(0);
    }
}
