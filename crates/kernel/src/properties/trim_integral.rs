// SPDX-License-Identifier: GPL-2.0-only
//! Green's theorem along one trim: `integral F(u(t), v(t)) v'(t) dt` for the eight quantities.
//! Closed form for straight and circular trims of planes and for iso-parameter trims of
//! surfaces of revolution; adaptive Gauss-Kronrod for every other trim.
use super::accumulate::{Moments, QUANTITIES};
use super::boundary::{TrimShape, TrimUse};
use super::general::GeneralSurface;
use super::planar::{PlanarField, basis_integrals, combine, pointwise};
use super::poly::Poly;
use super::quadrature::{Quadrature, integrate, integrate_leading, moments};
use super::revolution::UField;
use super::trig_poly::TrigPoly;
use super::univariate::{TermSum, Univariate};
use rustycad_tkernel::standard::Failure;

/// The integrand family of one face.
pub(crate) enum FaceIntegrand {
    /// A plane.
    Planar(PlanarField),
    /// A cylinder or cone: polynomial profile.
    PolynomialRevolution(Box<[UField<Poly>; QUANTITIES]>),
    /// A sphere or torus: trigonometric profile.
    PeriodicRevolution(Box<[UField<TrigPoly>; QUANTITIES]>),
    /// Any other surface: nested quadrature.
    General(Box<GeneralSurface>),
}

/// The signed contribution of one trim use; `integrate_trim(&integrand, &trim)?`.
pub(crate) fn integrate_trim(
    integrand: &FaceIntegrand,
    trim: &TrimUse,
) -> Result<Moments, Failure> {
    let closed = match integrand {
        FaceIntegrand::Planar(field) => planar_closed_form(field, trim),
        FaceIntegrand::PolynomialRevolution(fields) => iso_parameter(fields.as_ref(), trim),
        FaceIntegrand::PeriodicRevolution(fields) => iso_parameter(fields.as_ref(), trim),
        FaceIntegrand::General(surface) => return general_trim(surface, trim),
    };
    if let Some(terms) = closed {
        let mut exact = Moments::default();
        exact.add_closed_form(&terms, trim.sign);
        return Ok(exact);
    }
    let numeric = integrate(&|t| sample(integrand, trim, t), trim.first, trim.last)?;
    let mut result = Moments::default();
    result.add(&moments(&numeric), trim.sign);
    Ok(result)
}

fn planar_closed_form(field: &PlanarField, trim: &TrimUse) -> Option<[TermSum; QUANTITIES]> {
    let basis = match trim.shape {
        TrimShape::Line { origin, direction } => {
            let u = Poly::linear(origin[0], direction[0]);
            let v = Poly::linear(origin[1], direction[1]);
            basis_integrals(&u, &v, trim.first, trim.last)
        }
        TrimShape::Circle { .. } => {
            let [u, v] = circle_coordinates(&trim.shape);
            basis_integrals(&u, &v, trim.first, trim.last)
        }
        TrimShape::General => return None,
    };
    Some(combine(field, &basis))
}

/// `u(t)` and `v(t)` of a circular trim as degree-one trigonometric polynomials.
fn circle_coordinates(shape: &TrimShape) -> [TrigPoly; 2] {
    let TrimShape::Circle {
        centre,
        x,
        y,
        radius,
    } = *shape
    else {
        return [TrigPoly::constant(0.0), TrigPoly::constant(0.0)];
    };
    [0, 1].map(|k| {
        let harmonic = TrigPoly::harmonic(1, radius * x[k], radius * y[k]);
        TrigPoly::constant(centre[k]).add(&harmonic)
    })
}

/// Iso-`u` lines integrate exactly; iso-`v` lines carry no `dv` and contribute exactly zero.
fn iso_parameter<V: Univariate>(
    fields: &[UField<V>; QUANTITIES],
    trim: &TrimUse,
) -> Option<[TermSum; QUANTITIES]> {
    let TrimShape::Line { origin, direction } = trim.shape else {
        return None;
    };
    if direction[1] == 0.0 {
        return Some([TermSum::default(); QUANTITIES]);
    }
    if direction[0] != 0.0 {
        return None;
    }
    let along = |t: f64| origin[1] + t * direction[1];
    let (from, to) = (along(trim.first), along(trim.last));
    Some(std::array::from_fn(|k| {
        fields[k].iso_u_integral(origin[0], from, to)
    }))
}

fn sample(integrand: &FaceIntegrand, trim: &TrimUse, t: f64) -> Result<[f64; QUANTITIES], Failure> {
    let ([u, v], [_, dv]) = trim.d1(t)?;
    Ok(match integrand {
        FaceIntegrand::Planar(field) => pointwise(field, u, v, dv),
        FaceIntegrand::PolynomialRevolution(fields) => revolution_sample(fields.as_ref(), u, v, dv),
        FaceIntegrand::PeriodicRevolution(fields) => revolution_sample(fields.as_ref(), u, v, dv),
        FaceIntegrand::General(surface) => surface.antiderivative(u, v)?.value.map(|f| f * dv),
    })
}

/// Iso-`v` trims carry no `dv`; others integrate the nested antiderivative. Channels 8..16
/// carry the inner error estimates, which ride along and are doubled as a safety margin.
fn general_trim(surface: &GeneralSurface, trim: &TrimUse) -> Result<Moments, Failure> {
    if matches!(trim.shape, TrimShape::Line { direction, .. } if direction[1] == 0.0) {
        return Ok(Moments::default());
    }
    let sample = |t: f64| -> Result<[f64; 2 * QUANTITIES], Failure> {
        let ([u, v], [_, dv]) = trim.d1(t)?;
        let inner = surface.antiderivative(u, v)?;
        Ok(std::array::from_fn(|k| match k {
            k if k < QUANTITIES => inner.value[k] * dv,
            k => inner.error[k - QUANTITIES] * dv.abs(),
        }))
    };
    let sum = integrate_leading(&sample, (trim.first, trim.last), QUANTITIES)?;
    let mut outer = moments(&Quadrature {
        value: std::array::from_fn(|k| sum.value[k]),
        error: std::array::from_fn(|k| sum.error[k] + 2.0 * sum.value[k + QUANTITIES].abs()),
    });
    outer.value = outer.value.map(|value| value * trim.sign);
    Ok(outer)
}

fn revolution_sample<V: Univariate>(
    fields: &[UField<V>; QUANTITIES],
    u: f64,
    v: f64,
    dv: f64,
) -> [f64; QUANTITIES] {
    std::array::from_fn(|k| fields[k].antiderivative_at(u, v) * dv)
}
