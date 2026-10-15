// SPDX-License-Identifier: GPL-2.0-only
//! Planes `P(u, v) = O + u X + v Y`, unit density, normal `N = X x Y`.
//! Each integrand's `u` antiderivative is `c1 u + c2 u^2 + c3 u v`, so Green's theorem
//! reduces every quantity to the three trim integrals of `u v'`, `u^2 v'` and `u v v'`.
use super::accumulate::{AREA, AREA_MOMENT, FLUX, FLUX_MOMENT, QUANTITIES};
use super::revolution::Frame;
use super::univariate::{TermSum, Univariate};

/// `[c1, c2, c3]` of `F = c1 u + c2 u^2 + c3 u v` for each quantity.
pub(crate) type PlanarField = [[f64; 3]; QUANTITIES];

/// The eight antiderivative coefficient triples for a plane in `frame`.
/// Example: `planar_field(&frame)[AREA] == [1.0, 0.0, 0.0]`.
pub(crate) fn planar_field(frame: &Frame) -> PlanarField {
    let normal = cross(frame.x, frame.y);
    let flux: f64 = (0..3).map(|k| frame.offset[k] * normal[k]).sum();
    let moment = |axis: usize| [frame.offset[axis], 0.5 * frame.x[axis], frame.y[axis]];
    std::array::from_fn(|index| match index {
        AREA => [1.0, 0.0, 0.0],
        FLUX => [flux, 0.0, 0.0],
        i if i < FLUX => moment(i - AREA_MOMENT),
        i => moment(i - FLUX_MOMENT).map(|c| c * flux),
    })
}

/// Exact trim integrals `[u v', u^2 v', u v v']` for `u(t), v(t)` over `[from, to]`.
/// Example: `basis_integrals(&Poly::linear(1.0, 0.0), &Poly::linear(0.0, 1.0), 0.0, 1.0)`.
pub(crate) fn basis_integrals<V: Univariate>(u: &V, v: &V, from: f64, to: f64) -> [TermSum; 3] {
    let speed = v.derivative();
    let first = u.mul(&speed);
    [
        first.integral(from, to),
        u.mul(&first).integral(from, to),
        v.mul(&first).integral(from, to),
    ]
}

/// The quantities from the three basis integrals; `combine(&field, &basis)`.
pub(crate) fn combine(field: &PlanarField, basis: &[TermSum; 3]) -> [TermSum; QUANTITIES] {
    field.map(|coefficients| {
        (0..3).fold(TermSum::default(), |sum, k| {
            sum.plus(basis[k].times(coefficients[k]))
        })
    })
}

/// `F(u, v) * dv` pointwise, for trims integrated numerically; `pointwise(&field, u, v, dv)`.
pub(crate) fn pointwise(field: &PlanarField, u: f64, v: f64, dv: f64) -> [f64; QUANTITIES] {
    field.map(|[c1, c2, c3]| (c1 * u + c2 * u * u + c3 * u * v) * dv)
}

/// `a x b`.
pub(crate) fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::super::poly::Poly;
    use super::super::trig_poly::TrigPoly;
    use super::*;
    use std::f64::consts::PI;

    fn frame(offset: [f64; 3]) -> Frame {
        Frame {
            offset,
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            z: [0.0, 0.0, 1.0],
        }
    }

    #[test]
    fn unit_square_boundary_gives_area_and_centroid() {
        let field = planar_field(&frame([0.0, 0.0, -2.0]));
        // Counter-clockwise unit square; only the two vertical sides carry dv.
        let right = combine(
            &field,
            &basis_integrals(&Poly::constant(1.0), &Poly::linear(0.0, 1.0), 0.0, 1.0),
        );
        let left = combine(
            &field,
            &basis_integrals(&Poly::constant(0.0), &Poly::linear(1.0, -1.0), 0.0, 1.0),
        );
        let total = |k: usize| right[k].value + left[k].value;
        assert_eq!(total(AREA), 1.0);
        assert_eq!(total(AREA_MOMENT), 0.5);
        assert_eq!(total(AREA_MOMENT + 1), 0.5);
        assert_eq!(total(FLUX), -2.0);
    }

    #[test]
    fn circle_trim_gives_disc_area_and_moment() {
        let field = planar_field(&frame([0.0; 3]));
        let u = TrigPoly::constant(3.0).add(&TrigPoly::harmonic(1, 2.0, 0.0));
        let v = TrigPoly::harmonic(1, 0.0, 2.0);
        let terms = combine(&field, &basis_integrals(&u, &v, 0.0, 2.0 * PI));
        assert!((terms[AREA].value - 4.0 * PI).abs() < 1e-13);
        assert!((terms[AREA_MOMENT].value - 12.0 * PI).abs() < 1e-12);
        assert!(terms[AREA_MOMENT + 1].value.abs() < 1e-12);
        assert_eq!(pointwise(&field, 2.0, 1.0, 1.0)[AREA], 2.0);
        assert_eq!(cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), [0.0, 0.0, 1.0]);
    }
}
