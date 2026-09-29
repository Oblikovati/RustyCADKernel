// Created: 2009-02-02
//
// Copyright (c) 2009-2013 OPEN CASCADE SAS
//
// This file is part of commercial software by OPEN CASCADE SAS,
// furnished in accordance with the terms and conditions of the contract
// and with the inclusion of this copyright notice.
// This file or any part thereof may not be provided or otherwise
// made available to any third party.
//
// No ownership title to the software is transferred hereby.
//
// OPEN CASCADE SAS makes no representation or warranties with respect to the
// performance of this software, and specifically disclaims any responsibility
// for any damages, special or consequential, connected with its use.
// Rust translation of ModelingData/TKGeomBase/Geom2dConvert/Geom2dConvert_PPoint.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dConvert_PPoint`: a curve parameter with its point and first derivative.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkmath::gp::{Point2, Vector2, Xy};

/// A point of a curve with its parameter and tangent; two points are equal when
/// their parameters are within Precision::PConfusion().
///
/// `let p = PPoint::from_adaptor(0.5, &adaptor)?; let d = p.dist(&PPoint::default());`
#[derive(Clone, Copy, Debug)]
pub struct PPoint {
    my_parameter: f64,
    my_point: Point2,
    my_d1: Vector2,
}

impl Default for PPoint {
    /// The empty point: parameter RealLast(), point and derivative at the origin
    /// (`Geom2dConvert_PPoint()`).
    fn default() -> Self {
        Self {
            my_parameter: f64::MAX,
            my_point: Point2::new(0.0, 0.0),
            my_d1: Vector2::new(0.0, 0.0),
        }
    }
}

impl PPoint {
    /// A point from its values (`Geom2dConvert_PPoint(theParameter, thePoint, theD1)`);
    /// `PPoint::new(0.5, Xy::new(1.0, 2.0), Xy::new(0.0, 1.0))`.
    pub fn new(parameter: f64, point: Xy, d1: Xy) -> Self {
        Self {
            my_parameter: parameter,
            my_point: Point2::new(point.x(), point.y()),
            my_d1: Vector2::new(d1.x(), d1.y()),
        }
    }

    /// The point and derivative of `adaptor` at `parameter`; the adaptor's failure
    /// propagates (`Geom2dConvert_PPoint(theParameter, theAdaptor)`);
    /// `PPoint::from_adaptor(0.5, &adaptor)?`.
    pub fn from_adaptor(parameter: f64, adaptor: &dyn Adaptor2dCurve2d) -> Result<Self, Failure> {
        let (point, d1): (Point2, Vector2) = adaptor.d1(parameter)?;
        Ok(Self {
            my_parameter: parameter,
            my_point: point,
            my_d1: d1,
        })
    }

    /// The distance between the two points (`Dist`); `a.dist(&b)`.
    pub fn dist(&self, other: &PPoint) -> f64 {
        self.my_point.distance(other.my_point)
    }

    /// The parameter (`Parameter()`); `p.parameter()`.
    pub fn parameter(&self) -> f64 {
        self.my_parameter
    }

    /// The point location (`Point()`); `p.point()`.
    pub fn point(&self) -> Xy {
        self.my_point.xy()
    }

    /// The first derivative (`D1()`); `p.d1()`.
    pub fn d1(&self) -> Xy {
        self.my_d1.xy()
    }

    /// Replaces the first derivative (`SetD1`); `p.set_d1(Xy::new(1.0, 0.0))`.
    pub fn set_d1(&mut self, d1: Xy) {
        self.my_d1.set_xy(d1);
    }
}

/// `operator==` compares the parameters within Precision::PConfusion(); `operator!=`
/// is its own test (`> PConfusion()`), so both are false for a NaN parameter.
#[allow(clippy::partialeq_ne_impl)] // The native operator!= is not the negation of operator==.
impl PartialEq for PPoint {
    fn eq(&self, other: &PPoint) -> bool {
        (self.my_parameter - other.parameter()).abs() <= precision::p_confusion()
    }
    fn ne(&self, other: &PPoint) -> bool {
        (self.my_parameter - other.parameter()).abs() > precision::p_confusion()
    }
}
