// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/ElCLib/ElCLib.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The gp-object overloads of ElCLib (Value/D1/D2/D3/DN/Parameter) as traits, so one generic
//! function serves every curve type in both dimensions.
use super::{derivatives as dn, parameters as at, plane, space};
use crate::gp::{
    Circle2, Circle3, Ellipse2, Ellipse3, Hyperbola2, Hyperbola3, Line2, Line3, Parabola2,
    Parabola3,
};
use rustycad_tkernel::standard::Failure;

/// Curves that ElCLib evaluates by value, first derivative, N-th derivative and projection.
pub trait ElementaryCurve {
    /// Point type (Point3 or Point2).
    type Point;
    /// Derivative type (Vector3 or Vector2).
    type Vector;
    /// Point at a parameter; `curve.value(0.5)`.
    fn value(&self, u: f64) -> Self::Point;
    /// Point and first derivative; `curve.d1(0.5)`.
    fn d1(&self, u: f64) -> (Self::Point, Self::Vector);
    /// N-th derivative; `curve.dn(0.5,2)`.
    fn dn(&self, u: f64, n: i32) -> Self::Vector;
    /// Parameter of a point; only ellipses and planar circles can fail; `curve.parameter(p)?`.
    fn parameter(&self, point: Self::Point) -> Result<f64, Failure>;
}
/// Curves with an ElCLib D2 overload (every conic).
pub trait SecondDerivativeCurve: ElementaryCurve {
    /// Point with first and second derivatives; `curve.d2(0.5)`.
    fn d2(&self, u: f64) -> (Self::Point, Self::Vector, Self::Vector);
}
/// Curves with an ElCLib D3 overload (circles, ellipses and hyperbolas).
pub trait ThirdDerivativeCurve: SecondDerivativeCurve {
    /// Point with the first three derivatives; `curve.d3(0.5)`.
    fn d3(&self, u: f64) -> (Self::Point, Self::Vector, Self::Vector, Self::Vector);
}

/// ElCLib::Value overloads; `elclib::value(0.5,&circle)`.
pub fn value<C: ElementaryCurve>(u: f64, curve: &C) -> C::Point {
    curve.value(u)
}
/// ElCLib::D1 overloads; `elclib::d1(0.5,&circle)`.
pub fn d1<C: ElementaryCurve>(u: f64, curve: &C) -> (C::Point, C::Vector) {
    curve.d1(u)
}
/// ElCLib::D2 overloads; `elclib::d2(0.5,&circle)`.
pub fn d2<C: SecondDerivativeCurve>(u: f64, curve: &C) -> (C::Point, C::Vector, C::Vector) {
    curve.d2(u)
}
/// ElCLib::D3 overloads; `elclib::d3(0.5,&circle)`.
pub fn d3<C: ThirdDerivativeCurve>(
    u: f64,
    curve: &C,
) -> (C::Point, C::Vector, C::Vector, C::Vector) {
    curve.d3(u)
}
/// ElCLib::DN overloads; `elclib::dn(0.5,&circle,3)`.
pub fn dn<C: ElementaryCurve>(u: f64, curve: &C, n: i32) -> C::Vector {
    curve.dn(u, n)
}
/// ElCLib::Parameter overloads; `elclib::parameter(&circle,point)?`.
pub fn parameter<C: ElementaryCurve>(curve: &C, point: C::Point) -> Result<f64, Failure> {
    curve.parameter(point)
}

/// Implements the traits for one curve type from its placement accessor and length arguments.
macro_rules! elementary_curve {
    ($curve:ty, $point:ty, $vector:ty, $place:ident, ($($len:ident),*),
     $value:path, $d1:path, $dn:path, |$c:ident, $p:ident| $parameter:expr) => {
        impl ElementaryCurve for $curve {
            type Point = $point;
            type Vector = $vector;
            fn value(&self, u: f64) -> $point {
                $value(u, self.$place() $(, self.$len())*)
            }
            fn d1(&self, u: f64) -> ($point, $vector) {
                $d1(u, self.$place() $(, self.$len())*)
            }
            fn dn(&self, u: f64, n: i32) -> $vector {
                $dn(u, self.$place() $(, self.$len())*, n)
            }
            fn parameter(&self, point: $point) -> Result<f64, Failure> {
                let ($c, $p) = (self, point);
                $parameter
            }
        }
    };
}
macro_rules! second_derivative {
    ($curve:ty, $place:ident, ($($len:ident),*), $d2:path) => {
        impl SecondDerivativeCurve for $curve {
            fn d2(&self, u: f64) -> (Self::Point, Self::Vector, Self::Vector) {
                $d2(u, self.$place() $(, self.$len())*)
            }
        }
    };
}
macro_rules! third_derivative {
    ($curve:ty, $place:ident, ($($len:ident),*), $d3:path) => {
        impl ThirdDerivativeCurve for $curve {
            fn d3(&self, u: f64) -> (Self::Point, Self::Vector, Self::Vector, Self::Vector) {
                $d3(u, self.$place() $(, self.$len())*)
            }
        }
    };
}

use crate::gp::{Point2, Point3, Vector2, Vector3};
elementary_curve!(
    Line3,
    Point3,
    Vector3,
    position,
    (),
    space::line_value,
    space::line_d1,
    dn::line_dn,
    |c, p| Ok(at::line_parameter(c.position(), p))
);
elementary_curve!(
    Circle3,
    Point3,
    Vector3,
    position,
    (radius),
    space::circle_value,
    space::circle_d1,
    dn::circle_dn,
    |c, p| Ok(at::circle_parameter(c.position(), p))
);
elementary_curve!(
    Ellipse3,
    Point3,
    Vector3,
    position,
    (major_radius, minor_radius),
    space::ellipse_value,
    space::ellipse_d1,
    dn::ellipse_dn,
    |c, p| at::ellipse_parameter(c.position(), c.major_radius(), c.minor_radius(), p)
);
elementary_curve!(
    Hyperbola3,
    Point3,
    Vector3,
    position,
    (major_radius, minor_radius),
    space::hyperbola_value,
    space::hyperbola_d1,
    dn::hyperbola_dn,
    |c, p| Ok(at::hyperbola_parameter(
        c.position(),
        c.major_radius(),
        c.minor_radius(),
        p
    ))
);
elementary_curve!(
    Parabola3,
    Point3,
    Vector3,
    position,
    (focal),
    space::parabola_value,
    space::parabola_d1,
    dn::parabola_dn,
    |c, p| Ok(at::parabola_parameter(c.position(), p))
);
elementary_curve!(
    Line2,
    Point2,
    Vector2,
    position,
    (),
    plane::line_value_2d,
    plane::line_d1_2d,
    dn::line_dn_2d,
    |c, p| Ok(at::line_parameter_2d(c.position(), p))
);
elementary_curve!(
    Circle2,
    Point2,
    Vector2,
    axis,
    (radius),
    plane::circle_value_2d,
    plane::circle_d1_2d,
    dn::circle_dn_2d,
    |c, p| at::circle_parameter_2d(c.axis(), p)
);
elementary_curve!(
    Ellipse2,
    Point2,
    Vector2,
    axis,
    (major_radius, minor_radius),
    plane::ellipse_value_2d,
    plane::ellipse_d1_2d,
    dn::ellipse_dn_2d,
    |c, p| at::ellipse_parameter_2d(c.axis(), c.major_radius(), c.minor_radius(), p)
);
elementary_curve!(
    Hyperbola2,
    Point2,
    Vector2,
    axis,
    (major_radius, minor_radius),
    plane::hyperbola_value_2d,
    plane::hyperbola_d1_2d,
    dn::hyperbola_dn_2d,
    |c, p| Ok(at::hyperbola_parameter_2d(
        c.axis(),
        c.major_radius(),
        c.minor_radius(),
        p
    ))
);
elementary_curve!(
    Parabola2,
    Point2,
    Vector2,
    axis,
    (focal),
    plane::parabola_value_2d,
    plane::parabola_d1_2d,
    dn::parabola_dn_2d,
    |c, p| Ok(at::parabola_parameter_2d(c.axis(), p))
);

second_derivative!(Circle3, position, (radius), space::circle_d2);
second_derivative!(
    Ellipse3,
    position,
    (major_radius, minor_radius),
    space::ellipse_d2
);
second_derivative!(
    Hyperbola3,
    position,
    (major_radius, minor_radius),
    space::hyperbola_d2
);
second_derivative!(Parabola3, position, (focal), space::parabola_d2);
second_derivative!(Circle2, axis, (radius), plane::circle_d2_2d);
second_derivative!(
    Ellipse2,
    axis,
    (major_radius, minor_radius),
    plane::ellipse_d2_2d
);
second_derivative!(
    Hyperbola2,
    axis,
    (major_radius, minor_radius),
    plane::hyperbola_d2_2d
);
second_derivative!(Parabola2, axis, (focal), plane::parabola_d2_2d);

third_derivative!(Circle3, position, (radius), space::circle_d3);
third_derivative!(
    Ellipse3,
    position,
    (major_radius, minor_radius),
    space::ellipse_d3
);
third_derivative!(
    Hyperbola3,
    position,
    (major_radius, minor_radius),
    space::hyperbola_d3
);
third_derivative!(Circle2, axis, (radius), plane::circle_d3_2d);
third_derivative!(
    Ellipse2,
    axis,
    (major_radius, minor_radius),
    plane::ellipse_d3_2d
);
third_derivative!(
    Hyperbola2,
    axis,
    (major_radius, minor_radius),
    plane::hyperbola_d3_2d
);
