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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/FairCurve/FairCurve_Batten.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FairCurve_Batten`: the curve of an elastic batten between two points, with
//! optional end angles, a height law and a sliding length.
mod compute;
pub(super) mod deform;
mod sliding;
pub(super) mod solve;
pub(super) mod step;

use super::FairCurveAnalysisCode;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, arrays::one_based};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point2, Vector2};

/// The batten: the requested ("new") constraints, the ones of the last computed
/// curve ("old") and that curve, a non-rational B-spline of degree 9.
///
/// ```ignore
/// let mut batten = FairCurveBatten::new(p1, p2, 1.0, 0.0)?;
/// batten.set_angle1(0.3);
/// let mut code = FairCurveAnalysisCode::Ok;
/// batten.compute(&mut code, 50, 1e-3)?;
/// ```
#[derive(Clone)]
pub struct FairCurveBatten {
    pub(super) code: FairCurveAnalysisCode,
    pub(super) old: BattenConstraints,
    pub(super) new: BattenConstraints,
    pub(super) degree: i32,
    pub(super) poles: Handle<HArray1<Point2>>,
    pub(super) flatknots: Handle<HArray1<f64>>,
    pub(super) knots: Handle<HArray1<f64>>,
    pub(super) mults: Handle<HArray1<i32>>,
}

/// One set of batten constraints (the native `Old*` or `New*` fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct BattenConstraints {
    pub(super) p1: Point2,
    pub(super) p2: Point2,
    pub(super) angle1: f64,
    pub(super) angle2: f64,
    pub(super) height: f64,
    pub(super) slope: f64,
    pub(super) sliding_factor: f64,
    pub(super) free_sliding: bool,
    pub(super) constraint_order1: i32,
    pub(super) constraint_order2: i32,
}

/// `Standard_NullValue` for confused end points.
fn confused(p1: Point2, p2: Point2) -> Failure {
    Failure::with_message(
        "Standard_NullValue",
        &format!(
            "FairCurve : P1 and P2 are confused: ({}, {}) and ({}, {}) closer than {}",
            p1.x(),
            p1.y(),
            p2.x(),
            p2.y(),
            precision::confusion()
        ),
    )
}

/// `Standard_NegativeValue` for a height that is not positive.
fn negative_height(context: &str, height: f64) -> Failure {
    Failure::with_message(
        "Standard_NegativeValue",
        &format!("FairCurve : {context}: Height = {height}; expected Height > 0"),
    )
}

impl FairCurveBatten {
    /// A batten from `p1` to `p2` of middle `height` and `slope`, initialized by the
    /// straight line raised to degree 9. Confused points raise `Standard_NullValue`,
    /// a height that is not positive `Standard_NegativeValue`;
    /// `FairCurveBatten::new(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0), 1.0, 0.0)?`.
    pub fn new(p1: Point2, p2: Point2, height: f64, slope: f64) -> Result<Self, Failure> {
        if p1.is_equal(p2, precision::confusion()) {
            return Err(confused(p1, p2));
        }
        if height <= 0.0 {
            return Err(negative_height("Height is not positive", height));
        }
        let constraints = BattenConstraints {
            p1,
            p2,
            angle1: 0.0,
            angle2: 0.0,
            height,
            slope,
            sliding_factor: 1.0,
            free_sliding: false,
            constraint_order1: 1,
            constraint_order2: 1,
        };
        let (poles, knots, mults, flatknots) = straight_line(p1, p2, 9)?;
        Ok(Self {
            code: FairCurveAnalysisCode::Ok,
            old: constraints,
            new: constraints,
            degree: 9,
            poles: Handle::new(HArray1::from_owned(poles)),
            flatknots: Handle::new(HArray1::from_owned(flatknots)),
            knots: Handle::new(HArray1::from_owned(knots)),
            mults: Handle::new(HArray1::from_owned(mults)),
        })
    }
    /// Frees the sliding (its factor is then computed); `batten.set_free_sliding(true)`.
    pub fn set_free_sliding(&mut self, free_sliding: bool) {
        self.new.free_sliding = free_sliding;
    }
    /// The constraint order at P1: 0 (none), 1 (angle) or 2 (curvature too);
    /// `batten.set_constraint_order1(0)`.
    pub fn set_constraint_order1(&mut self, constraint_order: i32) {
        self.new.constraint_order1 = constraint_order;
    }
    /// The constraint order at P2; `batten.set_constraint_order2(0)`.
    pub fn set_constraint_order2(&mut self, constraint_order: i32) {
        self.new.constraint_order2 = constraint_order;
    }
    /// Moves P1; the end angles follow the rotation of P1P2. Confused points raise
    /// `Standard_NullValue`; `batten.set_p1(Point2::new(0.0, 0.1))?`.
    pub fn set_p1(&mut self, p1: Point2) -> Result<(), Failure> {
        if p1.is_equal(self.new.p2, precision::confusion()) {
            return Err(confused(p1, self.new.p2));
        }
        self.angles(p1, self.new.p2)?;
        self.new.p1 = p1;
        Ok(())
    }
    /// Moves P2 like `set_p1`; `batten.set_p2(Point2::new(2.0, 0.0))?`.
    pub fn set_p2(&mut self, p2: Point2) -> Result<(), Failure> {
        if self.new.p1.is_equal(p2, precision::confusion()) {
            return Err(confused(self.new.p1, p2));
        }
        self.angles(self.new.p1, p2)?;
        self.new.p2 = p2;
        Ok(())
    }
    /// `Angles(P1, P2)`: turns the requested angles by the rotation of P1P2.
    fn angles(&mut self, p1: Point2, p2: Point2) -> Result<(), Failure> {
        let v_old = Vector2::between(self.new.p1, self.new.p2);
        let v_new = Vector2::between(p1, p2);
        let dangle: f64 = v_old.angle(v_new)?;
        self.new.angle1 -= dangle;
        self.new.angle2 += dangle;
        Ok(())
    }
    /// The angle at P1 from P1P2; `batten.set_angle1(0.2)`.
    pub fn set_angle1(&mut self, angle1: f64) {
        self.new.angle1 = angle1;
    }
    /// The angle at P2 from P2P1; `batten.set_angle2(0.2)`.
    pub fn set_angle2(&mut self, angle2: f64) {
        self.new.angle2 = angle2;
    }
    /// The middle height; not positive raises `Standard_NegativeValue`;
    /// `batten.set_height(2.0)?`.
    pub fn set_height(&mut self, height: f64) -> Result<(), Failure> {
        if height <= 0.0 {
            return Err(negative_height("Height is no positive", height));
        }
        self.new.height = height;
        Ok(())
    }
    /// The geometric slope of the height; `batten.set_slope(0.1)`.
    pub fn set_slope(&mut self, slope: f64) {
        self.new.slope = slope;
    }
    /// The ratio of the active length to the reference length;
    /// `batten.set_sliding_factor(1.2)`.
    pub fn set_sliding_factor(&mut self, sliding_factor: f64) {
        self.new.sliding_factor = sliding_factor;
    }
    /// The free-sliding flag of the computed curve; `batten.get_free_sliding()`.
    pub fn get_free_sliding(&self) -> bool {
        self.old.free_sliding
    }
    /// The constraint order at P1 of the computed curve; `batten.get_constraint_order1()`.
    pub fn get_constraint_order1(&self) -> i32 {
        self.old.constraint_order1
    }
    /// The constraint order at P2 of the computed curve; `batten.get_constraint_order2()`.
    pub fn get_constraint_order2(&self) -> i32 {
        self.old.constraint_order2
    }
    /// P1 of the computed curve; `batten.get_p1()`.
    pub fn get_p1(&self) -> Point2 {
        self.old.p1
    }
    /// P2 of the computed curve; `batten.get_p2()`.
    pub fn get_p2(&self) -> Point2 {
        self.old.p2
    }
    /// The angle at P1 of the computed curve; `batten.get_angle1()`.
    pub fn get_angle1(&self) -> f64 {
        self.old.angle1
    }
    /// The angle at P2 of the computed curve; `batten.get_angle2()`.
    pub fn get_angle2(&self) -> f64 {
        self.old.angle2
    }
    /// The height of the computed curve; `batten.get_height()`.
    pub fn get_height(&self) -> f64 {
        self.old.height
    }
    /// The slope of the computed curve; `batten.get_slope()`.
    pub fn get_slope(&self) -> f64 {
        self.old.slope
    }
    /// The sliding factor of the computed curve; `batten.get_sliding_factor()`.
    pub fn get_sliding_factor(&self) -> f64 {
        self.old.sliding_factor
    }
    /// The computed curve; `batten.curve()?`.
    pub fn curve(&self) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        let poles = self.poles.get().expect("batten poles").array1();
        let knots = self.knots.get().expect("batten knots").array1();
        let mults = self.mults.get().expect("batten mults").array1();
        let curve = Geom2dBSplineCurve::new(&poles, &knots, &mults, self.degree, false)?;
        Ok(Handle::new(curve))
    }
}

/// The segment P1P2 raised to `degree`: poles, knots, multiplicities, flat knots.
type SplineArrays = (
    Array1<'static, Point2>,
    Array1<'static, f64>,
    Array1<'static, i32>,
    Array1<'static, f64>,
);

fn straight_line(p1: Point2, p2: Point2, degree: i32) -> Result<SplineArrays, Failure> {
    let mut ipoles: Array1<'static, Point2> = one_based(2)?;
    (ipoles[1], ipoles[2]) = (p1, p2);
    let mut iknots: Array1<'static, f64> = one_based(2)?;
    (iknots[1], iknots[2]) = (0.0, 1.0);
    let mut imults: Array1<'static, i32> = one_based(2)?;
    (imults[1], imults[2]) = (2, 2);
    let mut npoles: Array1<'static, Point2> = one_based(degree + 1)?;
    let (mut nknots, mut nmults) = (one_based(2)?, one_based(2)?);
    // The native call passes a 2-entry weight array with `NoWeights()` input; a
    // non-rational elevation never writes it.
    bsplclib::increase_degree(
        1,
        degree,
        false,
        &ipoles,
        None,
        &iknots,
        &imults,
        &mut npoles,
        None,
        &mut nknots,
        &mut nmults,
    )?;
    let length: i32 = bsplclib::knot_sequence_length(&nmults, degree, false)?;
    let mut flat: Array1<'static, f64> = one_based(length)?;
    bsplclib::knot_sequence_with_degree(&nknots, &nmults, degree, false, &mut flat)?;
    Ok((npoles, nknots, nmults, flat))
}
