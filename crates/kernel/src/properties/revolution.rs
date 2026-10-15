// SPDX-License-Identifier: GPL-2.0-only
//! Cylinders, cones, spheres and tori as surfaces of revolution
//! `P(u, v) = O + r(v) (cos u X + sin u Y) + h(v) Z` with `|Pu x Pv| = s r(v)`, `s` constant,
//! and `Pu x Pv = e r (h' (cos u X + sin u Y) - r' Z)`, `e = +1` for a direct frame.
//! Every integrand is a trigonometric polynomial in `u` whose coefficients are profile
//! functions of `v`; the `u` antiderivative is exact, and so is the `v` integral along
//! iso-`u` trims (seams, generators, the edges of every revolved primitive).
use super::accumulate::{AREA, AREA_MOMENT, FLUX, FLUX_MOMENT, QUANTITIES};
use super::univariate::{TermSum, Univariate};

/// Number of `u` harmonics kept: `1, cos u, sin u, cos 2u, sin 2u`.
const MODES: usize = 5;

/// The profile of a surface of revolution; `Profile { radius, height, speed, handedness }`.
pub(crate) struct Profile<V> {
    /// `r(v)`, the distance to the axis.
    pub(crate) radius: V,
    /// `h(v)`, the height along the axis.
    pub(crate) height: V,
    /// `s = sqrt(r'^2 + h'^2)`, constant for every supported surface.
    pub(crate) speed: f64,
    /// `+1` for a direct (right-handed) frame, `-1` otherwise.
    pub(crate) handedness: f64,
}

/// The surface frame relative to the integration reference point `Q`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Frame {
    /// `O - Q`.
    pub(crate) offset: [f64; 3],
    pub(crate) x: [f64; 3],
    pub(crate) y: [f64; 3],
    pub(crate) z: [f64; 3],
}

/// `sum_m mode_m(u) * coefficient_m(v)` over the harmonic basis.
#[derive(Clone)]
pub(crate) struct UField<V> {
    modes: [V; MODES],
}

/// `(frequency, is_sine)` of each basis entry.
const BASIS: [(usize, bool); MODES] = [(0, false), (1, false), (1, true), (2, false), (2, true)];

fn mode_index(frequency: usize, sine: bool) -> usize {
    match (frequency, sine) {
        (0, _) => 0,
        _ => 2 * frequency - 1 + usize::from(sine),
    }
}

impl<V: Univariate> UField<V> {
    fn from_modes(constant: V, cosine: V, sine: V) -> Self {
        let zero = V::constant(0.0);
        Self {
            modes: [constant, cosine, sine, zero.clone(), zero],
        }
    }

    /// The product; both factors must be of `u` degree one at most.
    fn mul(&self, other: &Self) -> Self {
        let zero = V::constant(0.0);
        let mut product = Self::from_modes(zero.clone(), zero.clone(), zero);
        for (i, a) in self.modes.iter().enumerate().take(3) {
            for (j, b) in other.modes.iter().enumerate().take(3) {
                product.accumulate(BASIS[i], BASIS[j], &a.mul(b));
            }
        }
        product
    }

    /// Adds `coefficient * basis_a(u) * basis_b(u)` by the product-to-sum identities.
    fn accumulate(&mut self, (p, sp): (usize, bool), (q, sq): (usize, bool), coefficient: &V) {
        let half = coefficient.scale(0.5);
        let (sum, difference) = (p + q, p.abs_diff(q));
        // Sign of sin(p-q) when p < q, and the cos/sin sign pattern of each pair.
        let flip = if p >= q { 1.0 } else { -1.0 };
        let terms: [(usize, bool, f64); 2] = match (sp, sq) {
            (false, false) => [(sum, false, 1.0), (difference, false, 1.0)],
            (true, true) => [(sum, false, -1.0), (difference, false, 1.0)],
            (true, false) => [(sum, true, 1.0), (difference, true, flip)],
            (false, true) => [(sum, true, 1.0), (difference, true, -flip)],
        };
        for (frequency, sine, sign) in terms {
            if sine && frequency == 0 {
                continue;
            }
            let index = mode_index(frequency, sine);
            self.modes[index] = self.modes[index].add(&half.scale(sign));
        }
    }

    /// `A_m(u)`: the exact antiderivative from 0 of each basis entry at `u`.
    fn antiderivative_weights(u: f64) -> [f64; MODES] {
        BASIS.map(|(k, sine)| match (k, sine) {
            (0, _) => u,
            (k, false) => (k as f64 * u).sin() / k as f64,
            (k, true) => (1.0 - (k as f64 * u).cos()) / k as f64,
        })
    }

    /// Exact `integral_{from}^{to} F(u0, v) dv` where `dF/du` is this field.
    pub(crate) fn iso_u_integral(&self, u0: f64, from: f64, to: f64) -> TermSum {
        let weights = Self::antiderivative_weights(u0);
        let terms = self.modes.iter().zip(weights);
        terms.fold(TermSum::default(), |sum, (mode, weight)| {
            sum.plus(mode.integral(from, to).times(weight))
        })
    }

    /// `F(u, v)` where `dF/du` is this field and `F(0, v) = 0`.
    pub(crate) fn antiderivative_at(&self, u: f64, v: f64) -> f64 {
        let weights = Self::antiderivative_weights(u);
        let terms = self.modes.iter().zip(weights);
        terms.fold(0.0, |sum, (mode, weight)| sum + weight * mode.eval(v))
    }
}

/// The eight integrands of the accumulate layout for this surface and frame.
/// Example: `revolution_fields(&cylinder_profile, &frame)[AREA]` is `R` in mode 0.
pub(crate) fn revolution_fields<V: Univariate>(
    profile: &Profile<V>,
    frame: &Frame,
) -> [UField<V>; QUANTITIES] {
    let density = constant_field(profile.radius.scale(profile.speed));
    let flux = flux_field(profile, frame);
    let position: [UField<V>; 3] = std::array::from_fn(|axis| position_field(profile, frame, axis));
    std::array::from_fn(|index| match index {
        AREA => density.clone(),
        FLUX => flux.clone(),
        i if i < FLUX => density.mul(&position[i - AREA_MOMENT]),
        i => flux.mul(&position[i - FLUX_MOMENT]),
    })
}

fn constant_field<V: Univariate>(value: V) -> UField<V> {
    UField::from_modes(value, V::constant(0.0), V::constant(0.0))
}

/// Component `axis` of `P - Q = (O - Q) + r (cos u X + sin u Y) + h Z`.
fn position_field<V: Univariate>(profile: &Profile<V>, frame: &Frame, axis: usize) -> UField<V> {
    let along = profile.height.scale(frame.z[axis]);
    UField::from_modes(
        V::constant(frame.offset[axis]).add(&along),
        profile.radius.scale(frame.x[axis]),
        profile.radius.scale(frame.y[axis]),
    )
}

/// `(P - Q) . (Pu x Pv) = e r [h' (d.X cos u + d.Y sin u) + r h' - h r' - (d.Z) r']`.
fn flux_field<V: Univariate>(profile: &Profile<V>, frame: &Frame) -> UField<V> {
    let (r, h) = (&profile.radius, &profile.height);
    let (dr, dh) = (r.derivative(), h.derivative());
    let dot = |axis: [f64; 3]| (0..3).map(|k| frame.offset[k] * axis[k]).sum::<f64>();
    let er = r.scale(profile.handedness);
    let swept = r.mul(&dh).add(&h.mul(&dr).scale(-1.0));
    let constant = er.mul(&swept.add(&dr.scale(-dot(frame.z))));
    let lateral = er.mul(&dh);
    UField::from_modes(
        constant,
        lateral.scale(dot(frame.x)),
        lateral.scale(dot(frame.y)),
    )
}

#[cfg(test)]
mod tests {
    use super::super::poly::Poly;
    use super::super::trig_poly::TrigPoly;
    use super::*;
    use std::f64::consts::PI;

    fn unit_frame() -> Frame {
        Frame {
            offset: [0.0; 3],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            z: [0.0, 0.0, 1.0],
        }
    }

    fn value(field: &UField<Poly>, u: f64) -> f64 {
        let modes = field.modes.iter().zip(BASIS);
        modes.fold(0.0, |sum, (mode, (k, sine))| {
            let angle = k as f64 * u;
            let basis = if sine { angle.sin() } else { angle.cos() };
            sum + basis * mode.eval(0.0)
        })
    }

    #[test]
    fn product_to_sum_matches_pointwise_products() {
        let a = UField::from_modes(
            Poly::constant(1.0),
            Poly::constant(2.0),
            Poly::constant(-3.0),
        );
        let b = UField::from_modes(
            Poly::constant(0.5),
            Poly::constant(-1.0),
            Poly::constant(4.0),
        );
        let product = a.mul(&b);
        for u in [-1.0, 0.2, 2.5] {
            assert!((value(&product, u) - value(&a, u) * value(&b, u)).abs() < 1e-14);
        }
    }

    #[test]
    fn sphere_area_and_volume_from_one_seam() {
        let profile = Profile {
            radius: TrigPoly::harmonic(1, 2.0, 0.0),
            height: TrigPoly::harmonic(1, 0.0, 2.0),
            speed: 2.0,
            handedness: 1.0,
        };
        let fields = revolution_fields(&profile, &unit_frame());
        // The seam at u = 2 pi traversed upwards closes the domain [0, 2pi] x [-pi/2, pi/2].
        let area = fields[AREA].iso_u_integral(2.0 * PI, -PI / 2.0, PI / 2.0);
        assert!((area.value - 16.0 * PI).abs() < 1e-13);
        let flux = fields[FLUX].iso_u_integral(2.0 * PI, -PI / 2.0, PI / 2.0);
        assert!((flux.value / 3.0 - 32.0 * PI / 3.0).abs() < 1e-12);
        assert!((fields[AREA].antiderivative_at(PI, 0.0) - 2.0 * PI * 2.0).abs() < 1e-14);
    }
}
