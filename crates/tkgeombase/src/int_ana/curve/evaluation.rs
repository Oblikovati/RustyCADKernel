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
// Rust translation of IntAna_Curve.cxx (Value, D1u, FindParameter, InternalValue,
// InternalUVValue) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The points of an [`Curve`]: the (U, V) of a parameter on the cylinder or the
//! cone, the point, its tangent and the projection of a point.
use super::{Curve, TrigTerms, domain_error};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The outputs of InternalUVValue: the (U, V) on the surface, the polynomial
/// `A Z² + B Z + C` at the parameter, and the signed square root of its
/// discriminant (unchanged when A vanishes).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct UvValue {
    pub u: f64,
    pub v: f64,
    pub a: f64,
    pub signe_sqrt_dis: f64,
}

/// The values at Theta that every trigonometric term needs.
struct Angles {
    cost: f64,
    sint: f64,
    sin2t: f64,
    cos2t: f64,
}

impl TrigTerms {
    /// The polynomial at Theta: `Cte + sin (Sin + sin SinSin) + cos (Cos + cos CosCos) + CosSin sin 2t`.
    fn value(&self, t: &Angles) -> f64 {
        self.cte
            + t.sint * (self.sin + t.sint * self.sin_sin)
            + t.cost * (self.cos + t.cost * self.cos_cos)
            + self.cos_sin * t.sin2t
    }

    /// The derivative in Theta whose last term is `cos 2t * cross` (the native
    /// Z2 derivative squares CosSin where the others double it).
    fn derivative(&self, t: &Angles, cross: f64) -> f64 {
        self.sin * t.cost - self.cos * t.sint
            + t.sin2t * (self.sin_sin - self.cos_cos)
            + t.cos2t * cross
    }
}

impl Curve {
    /// InternalUVValue: the (U, V) of the parameter `theta`, the second Z root
    /// beyond DomainSup for two curves; Standard_DomainError outside the domain.
    /// The discriminant is zeroed below its error estimate over the step
    /// `100 Epsilon(2 DomainSup - DomainInf)`.
    pub(super) fn internal_uv_value(
        &self,
        theta: f64,
        signe_sqrt_dis: f64,
    ) -> Result<UvValue, Failure> {
        let (inf, sup) = (self.domain_inf, self.domain_sup);
        let (rel_tolp, rel_tolm) = (1.0 + epsilon(1.0), 1.0 - epsilon(1.0));
        let dt: f64 = 100.0 * epsilon(sup + sup - inf);
        if (theta < inf * rel_tolm)
            || ((theta > sup * rel_tolp) && !self.two_curves)
            || (theta > (sup + sup - inf) * rel_tolp)
        {
            return Err(domain_error(&format!(
                "the parameter {theta} is outside [{inf}, {sup}]; expected a parameter of the domain"
            )));
        }
        let (theta, mut second) = if (theta - sup).abs() < dt {
            (sup, false)
        } else if theta > sup {
            (sup + sup - theta, true)
        } else {
            (theta, false)
        };
        if !self.two_curves {
            second = self.take_z_positive;
        }
        let angles = Angles {
            cost: theta.cos(),
            sint: theta.sin(),
            sin2t: (theta + theta).sin(),
            cos2t: (theta + theta).cos(),
        };
        let mut result = UvValue {
            u: theta,
            signe_sqrt_dis,
            ..UvValue::default()
        };
        self.solve_z(&angles, dt, second, &mut result);
        Ok(result)
    }

    /// The root V of `A Z² + B Z + C` at the angles: `-C/B` (or 0) when A vanishes
    /// within Precision::PConfusion(), else `(-B ± sqrt(disc)) / 2A`.
    fn solve_z(&self, t: &Angles, dt: f64, second: bool, result: &mut UvValue) {
        let a: f64 = self.z2.value(t);
        let da: f64 = self.z2.derivative(t, self.z2.cos_sin * self.z2.cos_sin);
        let b: f64 = self.z1.value(t);
        let db: f64 = self.z1.derivative(t, self.z1.cos_sin + self.z1.cos_sin);
        let c: f64 = self.z0.value(t);
        let dc: f64 = self.z0.derivative(t, self.z0.cos_sin + self.z0.cos_sin);
        let mut discriminant: f64 = b * b - 4.0 * a * c;
        // The discriminant error over the step dt: |d(Disc)/dt| dt.
        let tol_d: f64 = 2.0 * dt * (b * db - 2.0 * (a * dc + c * da)).abs();
        if discriminant < tol_d {
            discriminant = 0.0;
        }
        result.a = a;
        if a.abs() > precision::p_confusion() || a.is_nan() {
            result.signe_sqrt_dis = if second {
                discriminant.sqrt()
            } else {
                -discriminant.sqrt()
            };
            result.v = (-b + result.signe_sqrt_dis) / (a + a);
        } else if b.abs() <= precision::p_confusion() {
            result.v = 0.0;
        } else {
            result.v = -c / b;
        }
    }

    /// InternalValue(U, V): the point of the cylinder, the cone (V is the radius,
    /// converted to the generatrix length) or the sphere, V clamped to ±100000;
    /// the origin for another quadric.
    pub(super) fn internal_value(&self, u: f64, v: f64) -> Point3 {
        // Native quirk: V is bounded to ±100000 (a NaN stays NaN, as natively).
        let v: f64 = v.clamp(-100000.0, 100000.0);
        match self.typequadric {
            SurfaceType::Cone => {
                let generatrix: f64 = (v - self.rcyl) / self.angle.sin();
                elslib::cone_value(u, generatrix, self.ax3, self.rcyl, self.angle)
            }
            SurfaceType::Cylinder => elslib::cylinder_value(u, v, self.ax3, self.rcyl),
            SurfaceType::Sphere => elslib::sphere_value(u, v, self.ax3, self.rcyl),
            _ => Point3::new(0.0, 0.0, 0.0),
        }
    }

    /// Value(Theta): the point of parameter `theta`; Standard_DomainError outside
    /// the domain. `curve.value(0.5)?`.
    pub fn value(&self, theta: f64) -> Result<Point3, Failure> {
        let uv: UvValue = self.internal_uv_value(theta, 0.0)?;
        Ok(self.internal_value(uv.u, uv.v))
    }

    /// D1u(Theta, P, V): stores the point in `point` and returns false when the
    /// tangent cannot be computed (|A| < 1e-7 or a discriminant root below 1e-10);
    /// otherwise stores in `vector` the finite difference over
    /// `(DomainSup - DomainInf) 1e-6` (backwards at the end of the domain) and
    /// returns true. `curve.d1u(0.5, &mut p, &mut v)?`.
    pub fn d1u(
        &self,
        theta: f64,
        point: &mut Point3,
        vector: &mut Vector3,
    ) -> Result<bool, Failure> {
        let uv: UvValue = self.internal_uv_value(theta, 0.0)?;
        *point = self.value(theta)?;
        if uv.a.abs() < 1.0e-7 || uv.signe_sqrt_dis.abs() < 1.0e-10 {
            return Ok(false);
        }
        let (inf, sup) = (self.domain_inf, self.domain_sup);
        let mut dtheta: f64 = (sup - inf) * 1.0e-6;
        let mut theta2: f64 = theta + dtheta;
        if (theta2 < inf)
            || ((theta2 > sup) && !self.two_curves)
            || (theta2 > (sup + sup - inf + 1.0e-14))
        {
            dtheta = -dtheta;
            theta2 = theta + dtheta;
        }
        let next: Point3 = self.value(theta2)?;
        let inverse: f64 = 1.0 / dtheta;
        vector.set_coordinates(
            (next.x() - point.x()) * inverse,
            (next.y() - point.y()) * inverse,
            (next.z() - point.z()) * inverse,
        );
        Ok(true)
    }
}
