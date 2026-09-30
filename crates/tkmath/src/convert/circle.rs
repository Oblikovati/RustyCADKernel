// Rust translation of Convert_CircleToBSplineCurve.cxx and Convert_EllipseToBSplineCurve.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles and ellipses: the unit circle B-spline scaled by the radii and placed
//! in the conic frame.
use super::arrays::one_based;
use super::cos_sin::{CosAndSin, build_trimmed};
use super::cos_sin_periodic::build_periodic;
use super::{ConicToBSplineCurve, ParameterisationType};
use crate::gp::{Axis2d, Circle2, Ellipse2, Point2, Transform2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use std::f64::consts::PI;

// The radii and axes the circle and ellipse conversions share.
struct CentralConic {
    major: f64,
    minor: f64,
    x_axis: Axis2d,
    y_axis: Axis2d,
}

impl ConicToBSplineCurve {
    /// Full circle (`Convert_CircleToBSplineCurve(C, Parameterisation)`): periodic
    /// for TgtThetaOver2 and RationalC1, otherwise the arc `[0, 2 PI]`;
    /// `ConicToBSplineCurve::from_circle(&circle, ParameterisationType::TgtThetaOver2)?`.
    pub fn from_circle(
        circle: &Circle2,
        parameterisation: ParameterisationType,
    ) -> Result<Self, Failure> {
        full_conic(&circle_shape(circle), parameterisation)
    }

    /// Circle arc `[u1, u2]` (`Convert_CircleToBSplineCurve(C, U1, U2, Parameterisation)`);
    /// the opening must lie in `]0, 2 PI + PConfusion]`;
    /// `ConicToBSplineCurve::from_circle_arc(&circle, 0.0, PI, ParameterisationType::QuasiAngular)?`.
    pub fn from_circle_arc(
        circle: &Circle2,
        u1: f64,
        u2: f64,
        parameterisation: ParameterisationType,
    ) -> Result<Self, Failure> {
        check_opening("Convert_CircleToBSplineCurve", u1, u2)?;
        conic_arc(&circle_shape(circle), u1, u2, parameterisation)
    }

    /// Full ellipse (`Convert_EllipseToBSplineCurve(E, Parameterisation)`);
    /// `ConicToBSplineCurve::from_ellipse(&ellipse, ParameterisationType::RationalC1)?`.
    pub fn from_ellipse(
        ellipse: &Ellipse2,
        parameterisation: ParameterisationType,
    ) -> Result<Self, Failure> {
        full_conic(&ellipse_shape(ellipse), parameterisation)
    }

    /// Ellipse arc `[u1, u2]` (`Convert_EllipseToBSplineCurve(E, U1, U2, Parameterisation)`);
    /// `ConicToBSplineCurve::from_ellipse_arc(&ellipse, 0.0, 1.0, ParameterisationType::Polynomial)?`.
    pub fn from_ellipse_arc(
        ellipse: &Ellipse2,
        u1: f64,
        u2: f64,
        parameterisation: ParameterisationType,
    ) -> Result<Self, Failure> {
        check_opening("Convert_EllipseToBSplineCurve", u1, u2)?;
        conic_arc(&ellipse_shape(ellipse), u1, u2, parameterisation)
    }
}

fn circle_shape(circle: &Circle2) -> CentralConic {
    CentralConic {
        major: circle.radius(),
        minor: circle.radius(),
        x_axis: circle.x_axis(),
        y_axis: circle.y_axis(),
    }
}

fn ellipse_shape(ellipse: &Ellipse2) -> CentralConic {
    CentralConic {
        major: ellipse.major_radius(),
        minor: ellipse.minor_radius(),
        x_axis: ellipse.x_axis(),
        y_axis: ellipse.y_axis(),
    }
}

fn check_opening(class: &str, u1: f64, u2: f64) -> Result<(), Failure> {
    let delta: f64 = u2 - u1;
    let eps: f64 = precision::p_confusion();
    if (delta > (2.0 * PI + eps)) || (delta <= 0.0e0) {
        return Err(Failure::with_type(
            "Standard_DomainError",
            &format!("{class}: arc [{u1}, {u2}] opens by {delta}, expected ]0, 2 PI + {eps}]"),
            "",
        ));
    }
    Ok(())
}

// Parameterisations BuildCosAndSin cannot make periodic are trimmed on [0, 2 PI].
fn full_conic(
    shape: &CentralConic,
    parameterisation: ParameterisationType,
) -> Result<ConicToBSplineCurve, Failure> {
    use ParameterisationType::{RationalC1, TgtThetaOver2};
    if parameterisation != TgtThetaOver2 && parameterisation != RationalC1 {
        return place(
            shape,
            &build_trimmed(parameterisation, 0.0, 2.0 * PI)?,
            false,
        );
    }
    place(shape, &build_periodic(parameterisation)?, true)
}

fn conic_arc(
    shape: &CentralConic,
    u1: f64,
    u2: f64,
    parameterisation: ParameterisationType,
) -> Result<ConicToBSplineCurve, Failure> {
    place(shape, &build_trimmed(parameterisation, u1, u2)?, false)
}

// Scales the unit circle poles by the radii (the minor one signed by the frame
// orientation) and moves them into the conic frame.
fn place(
    shape: &CentralConic,
    cos_and_sin: &CosAndSin,
    periodic: bool,
) -> Result<ConicToBSplineCurve, Failure> {
    let mut result = ConicToBSplineCurve::from_cos_and_sin(cos_and_sin, periodic);
    let (ox, oy) = (shape.x_axis.direction(), shape.y_axis.direction());
    let trsf: Transform2 = Transform2::transformation_between(shape.x_axis, Axis2d::default());
    let value: f64 = if ox.x() * oy.y() - ox.y() * oy.x() > 0.0e0 {
        shape.minor
    } else {
        -shape.minor
    };
    result.poles = one_based(cos_and_sin.cos_numerator.len() as i32)?;
    for ii in 1..=result.nb_poles() {
        let local = Point2::new(
            shape.major * cos_and_sin.cos_numerator[ii],
            value * cos_and_sin.sin_numerator[ii],
        );
        result.poles[ii] = local.transformed(trsf);
    }
    Ok(result)
}
