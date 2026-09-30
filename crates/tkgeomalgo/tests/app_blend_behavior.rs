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

//! Behavior of the AppBlend port beyond the numeric probe, which compares the
//! template with a native instance on the same probe sections.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::app_blend::*;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// `n` line points: Point(i) is the section index i.
struct Line(i32);

impl TheLine for Line {
    type Point = i32;
    fn nb_points(&self) -> i32 {
        self.0
    }
    fn point(&self, index: i32) -> i32 {
        index
    }
}

/// Straight polynomial sections `(j, t, 0)`, j = 0..2, translated along Y by `t`.
struct Translated {
    n: i32,
}

impl Translated {
    fn t(&self, i: i32) -> f64 {
        f64::from(i - 1) / f64::from(self.n - 1)
    }
}

impl TheSectionGenerator for Translated {
    type Point = i32;
    fn get_shape(&self) -> Result<[i32; 4], Failure> {
        Ok([3, 2, 2, 1])
    }
    fn knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        (knots[1], knots[2]) = (0.0, 1.0);
        Ok(())
    }
    fn mults(&self, mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
        (mults[1], mults[2]) = (3, 3);
        Ok(())
    }
    fn section_d1(&mut self, p: &i32, s: &mut AppBlendSection) -> Result<bool, Failure> {
        self.section(p, s)?;
        for j in 1..=3 {
            (s.d_poles[j], s.d_weights[j]) = (Vector3::new(0.0, 1.0, 0.0), 0.0);
        }
        s.d_poles2d[1] = Vector2::new(1.0, 0.0);
        Ok(true)
    }
    fn section(&mut self, p: &i32, s: &mut AppBlendSection) -> Result<(), Failure> {
        let t: f64 = self.t(*p);
        for j in 1..=3 {
            (s.poles[j], s.weights[j]) = (Point3::new(f64::from(j - 1), t, 0.0), 1.0);
        }
        s.poles2d[1] = Point2::new(t, 0.5);
        Ok(())
    }
    fn parameter(&self, p: &i32) -> Result<f64, Failure> {
        Ok(10.0 * self.t(*p))
    }
}

#[test]
fn queries_before_a_computation_are_not_done() {
    let approx = AppBlendAppSurf::default();
    assert!(!approx.is_done());
    assert_eq!(
        approx.u_degree().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(approx.tol_curve_on_surf(3), 0.0);
    let mut weights = AppBlendAppSurf::new(3, 8, 1e-4, 1e-5, 5, false);
    let error = weights.set_criterium_weight(0.1, -0.2, 0.3).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DomainError");
    assert_eq!(weights.criterium_weight(), [0.4, 0.2, 0.4]);
}

#[test]
fn a_translated_section_sweeps_a_plane() {
    let mut f = Translated { n: 6 };
    let mut approx = AppBlendAppSurf::new(3, 8, 1e-6, 1e-6, 5, true);
    // Native quirk: the Bezier pieces with known parameters fail in
    // AppDef_Compute::SplineValue.
    let error = approx.perform(Some(&Line(6)), &mut f, false).unwrap_err();
    assert_eq!(error.exception_type(), "StdFail_NotDone");
    approx.perform(Some(&Line(6)), &mut f, true).unwrap();
    assert!(approx.is_done());
    let [udeg, _, nb_u, _, nb_uk, _] = approx.surf_shape().unwrap();
    assert_eq!((udeg, nb_u, nb_uk), (2, 3, 2));
    // Known parameters: the V knots run over the section parameters.
    let knots = approx.surf_v_knots().unwrap();
    assert_eq!((knots[knots.lower()], knots[knots.upper()]), (0.0, 10.0));
    let poles = approx.surf_poles().unwrap();
    assert!(poles.iter().all(|p| p.z().abs() <= 1e-12));
    assert_eq!(approx.nb_curves2d().unwrap(), 1);
    assert_eq!(
        approx.curve2d(2).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn pieces_need_a_positive_size_and_a_line() {
    let mut f = Translated { n: 6 };
    let mut approx = AppBlendAppSurf::new(3, 8, 1e-6, 1e-6, 5, false);
    let error = approx
        .perform_by_pieces(Some(&Line(6)), &mut f, 0)
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DivideByZero");
    approx.perform_by_pieces(Some(&Line(6)), &mut f, 3).unwrap();
    assert!(approx.is_done());
    approx.perform(None::<&Line>, &mut f, true).unwrap();
    assert!(!approx.is_done());
}

#[test]
fn the_context_switches_start_false() {
    assert!(!get_context_spline_approx() && !get_context_approx_with_no_tgt());
    set_context_spline_approx(true);
    assert!(get_context_spline_approx());
    set_context_spline_approx(false);
}
