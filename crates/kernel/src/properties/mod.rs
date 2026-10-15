// SPDX-License-Identifier: GPL-2.0-only
//! Martensite ADR0073: analytic mass properties of owned B-rep faces and bodies.
//!
//! Faces on planes, cylinders, cones, spheres and tori integrate in closed form when
//! their trims are straight or circular in the parameter plane (every primitive and
//! extruded profile made of lines and arcs). Other trims on those surfaces use an
//! adaptive Gauss-Kronrod rule with its error estimate; free-form surfaces nest that rule
//! for the `u` antiderivative. Faces whose trims do not close in the parameter plane fall
//! back to the ported `BRepGProp` with an infinite bound. Every value carries its method.
mod accumulate;
mod body;
mod boundary;
mod face;
mod general;
mod measured;
mod native;
mod planar;
mod poly;
mod quadrature;
mod revolution;
mod surface;
mod trig_poly;
mod trim_integral;
mod univariate;

#[cfg(test)]
mod body_tests;
#[cfg(test)]
mod face_tests;

pub use body::body_properties;
pub use face::face_properties;
pub use measured::{
    BodyProperties, FaceProperties, IntegrationMethod, LoopRole, LoopSummary, Measured,
    MeasuredPoint,
};
