// Created by: Nikolai BUKHALOV
// Copyright (c) 2015 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib_CheckCurveOnSurface.cxx
// (GeomLib_CheckCurveOnSurface_TargetFunc) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The negated square distance between a 3D curve and a curve on surface at the
//! same parameter, with its derivatives and the exact maximum of elementary pairs.
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};
use rustycad_tkmath::math::{
    Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient, MultipleVarFunctionWithHessian,
    TrigonometricRoots, Vector,
};
use std::f64::consts::PI;

/// `GeomLib_CheckCurveOnSurface_TargetFunc`: `-|C1(t) - C2(t)|^2` on `[first, last]`.
/// The evaluations fail (false) outside the range or when a curve raises (the native
/// try/catch).
pub(super) struct TargetFunc<'c> {
    my_curve1: &'c dyn Adaptor3dCurve,
    my_curve2: &'c dyn Adaptor3dCurve,
    my_first: f64,
    my_last: f64,
}

impl<'c> TargetFunc<'c> {
    pub fn new(
        c3d: &'c dyn Adaptor3dCurve,
        curve_on_surface: &'c dyn Adaptor3dCurve,
        first: f64,
        last: f64,
    ) -> Self {
        Self {
            my_curve1: c3d,
            my_curve2: curve_on_surface,
            my_first: first,
            my_last: last,
        }
    }

    pub fn first_parameter(&self) -> f64 {
        self.my_first
    }

    pub fn last_parameter(&self) -> f64 {
        self.my_last
    }

    fn check_parameter(&self, param: f64) -> bool {
        self.my_first <= param && param <= self.my_last
    }

    /// The value of the one-dimension function at `x`, None outside the range or on
    /// a curve failure.
    pub fn value_at(&self, x: f64) -> Option<f64> {
        if !self.check_parameter(x) {
            return None;
        }
        let p1: Point3 = self.my_curve1.value(x).ok()?;
        let p2: Point3 = self.my_curve2.value(x).ok()?;
        Some(-p1.square_distance(&p2))
    }

    /// The first derivative, and the second one when `second` (`Derive`).
    pub fn derive(&self, x: f64, second: bool) -> Option<(f64, f64)> {
        if !self.check_parameter(x) {
            return None;
        }
        let (p1, dc1, dcc1, p2, dc2, dcc2) = if second {
            let (p1, dc1, dcc1) = self.my_curve1.d2(x).ok()?;
            let (p2, dc2, dcc2) = self.my_curve2.d2(x).ok()?;
            (p1, dc1, dcc1, p2, dc2, dcc2)
        } else {
            let (p1, dc1) = self.my_curve1.d1(x).ok()?;
            let (p2, dc2) = self.my_curve2.d1(x).ok()?;
            (p1, dc1, Vector3::default(), p2, dc2, Vector3::default())
        };
        let vec1 = Vector3::between(p1, p2);
        let vec2: Vector3 = dc2 - dc1;
        let deriv1: f64 = -2.0 * vec1.dot(vec2);
        let deriv2: f64 = if second {
            -2.0 * (vec2.square_magnitude() + vec1.dot(dcc2 - dcc1))
        } else {
            0.0
        };
        Some((deriv1, deriv2))
    }

    /// `ElementaryMaximum`: the exact best (smallest) value and its parameter for two
    /// lines (the better end) or two circles/ellipses (the ends and the roots of the
    /// trigonometric derivative on one period from the start); None when the pair is
    /// not elementary or the roots cannot be used.
    pub fn elementary_maximum(&self) -> Result<Option<(f64, f64)>, Failure> {
        let (type1, type2) = (self.my_curve1.get_type()?, self.my_curve2.get_type()?);
        if type1 == CurveType::Line && type2 == CurveType::Line {
            let (Some(first_value), Some(last_value)) =
                (self.value_at(self.my_first), self.value_at(self.my_last))
            else {
                return Ok(None);
            };
            return Ok(Some(if first_value <= last_value {
                (first_value, self.my_first)
            } else {
                (last_value, self.my_last)
            }));
        }
        let trig = |kind: CurveType| kind == CurveType::Circle || kind == CurveType::Ellipse;
        if !trig(type1) || !trig(type2) {
            return Ok(None);
        }
        let c1 = coefficients(self.my_curve1, type1)?;
        let c2 = coefficients(self.my_curve2, type2)?;
        self.trigonometric_maximum([c1.0 - c2.0, c1.1 - c2.1, c1.2 - c2.2])
    }

    // The derivative of the square distance in trigonometric form, and its roots.
    fn trigonometric_maximum(
        &self,
        [offset, cos_coeff, sin_coeff]: [Xyz; 3],
    ) -> Result<Option<(f64, f64)>, Failure> {
        let cos_cos: f64 = 2.0 * cos_coeff.dot(sin_coeff);
        let cos_sin: f64 = 0.5 * (sin_coeff.square_modulus() - cos_coeff.square_modulus());
        let cos: f64 = offset.dot(sin_coeff);
        let sin: f64 = -offset.dot(cos_coeff);
        let constant: f64 = -cos_coeff.dot(sin_coeff);
        // Exact zero is required here: a tolerance could classify a varying distance
        // as constant.
        if cos_cos == 0.0 && cos_sin == 0.0 && cos == 0.0 && sin == 0.0 && constant == 0.0 {
            return Ok(self
                .value_at(self.my_first)
                .map(|value| (value, self.my_first)));
        }
        let period: f64 = 2.0 * PI;
        let first: f64 = elclib::in_period(self.my_first, 0.0, period);
        let shift: f64 = first - self.my_first;
        let last: f64 = first + std_min(self.my_last - self.my_first, period);
        let scale: f64 = [cos_sin, cos, sin, constant]
            .iter()
            .fold(cos_cos.abs(), |m, v| std_max(m, v.abs()));
        let roots = TrigonometricRoots::new(
            cos_cos / scale,
            cos_sin / scale,
            cos / scale,
            sin / scale,
            constant / scale,
            first,
            last,
        );
        if !roots.is_done()
            || roots.infinite_roots()
            || (self.my_last - self.my_first >= period && roots.solutions()? == 0)
        {
            return Ok(None);
        }
        let mut best: (f64, f64) = (f64::MAX, self.my_first);
        let mut candidates: Vec<f64> = vec![self.my_first, self.my_last];
        for index in 1..=roots.solutions()? {
            candidates.push(roots.value(index)? - shift);
        }
        for parameter in candidates {
            let Some(value) = self.value_at(parameter) else {
                return Ok(None);
            };
            if value < best.0 {
                best = (value, parameter);
            }
        }
        Ok(Some(best))
    }
}

// The origin and the cosine and sine coefficients of a circle or an ellipse.
fn coefficients(curve: &dyn Adaptor3dCurve, kind: CurveType) -> Result<(Xyz, Xyz, Xyz), Failure> {
    if kind == CurveType::Circle {
        let circle = curve.circle()?;
        return Ok((
            circle.location().xyz(),
            circle.x_axis().direction().xyz() * circle.radius(),
            circle.y_axis().direction().xyz() * circle.radius(),
        ));
    }
    let ellipse = curve.ellipse()?;
    Ok((
        ellipse.location().xyz(),
        ellipse.x_axis().direction().xyz() * ellipse.major_radius(),
        ellipse.y_axis().direction().xyz() * ellipse.minor_radius(),
    ))
}

impl MultipleVarFunction for TargetFunc<'_> {
    fn variables(&self) -> i32 {
        1
    }

    fn value(&mut self, x: &Vector<'_>, value: &mut f64) -> bool {
        let Some(at) = x.value(1).ok().and_then(|x1| self.value_at(*x1)) else {
            return false;
        };
        *value = at;
        true
    }
}

impl MultipleVarFunctionWithGradient for TargetFunc<'_> {
    fn gradient(&mut self, x: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        let Some((d1, _)) = x.value(1).ok().and_then(|x1| self.derive(*x1, false)) else {
            return false;
        };
        gradient.change_value(1).map(|g| *g = d1).is_ok()
    }

    fn values(&mut self, x: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(x, value) && self.gradient(x, gradient)
    }
}

impl MultipleVarFunctionWithHessian for TargetFunc<'_> {
    fn values_hessian(
        &mut self,
        x: &Vector<'_>,
        value: &mut f64,
        gradient: &mut Vector<'_>,
        hessian: &mut Matrix<'_>,
    ) -> bool {
        if !self.value(x, value) {
            return false;
        }
        let Some((d1, d2)) = x.value(1).ok().and_then(|x1| self.derive(*x1, true)) else {
            return false;
        };
        gradient.change_value(1).map(|g| *g = d1).is_ok()
            && hessian.change_value(1, 1).map(|h| *h = d2).is_ok()
    }
}
