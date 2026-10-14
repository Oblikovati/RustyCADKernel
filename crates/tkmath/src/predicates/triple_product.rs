// SPDX-License-Identifier: GPL-2.0-only
//! Exact transversality of supplied vectors; no source, surface chart or body certificate.
use super::{NonFiniteCoordinate, exact_coordinate, exact_determinant};
use num_bigint::BigInt;
use std::cmp::Ordering;

/// Compare det(first, second, third) with zero without rounded cross/dot products.
/// A nonzero sign proves independence of the supplied binary64 vectors only.
/// Martensite extrusion consumers must separately bind the actual support plane,
/// placements, curve charts and required body validation (issue #10).
/// ```
/// use rustycad_tkmath::predicates::compare_scalar_triple_product_3d;
/// assert_eq!(compare_scalar_triple_product_3d([1.,0.,0.], [0.,1.,0.], [0.,0.,1.])?, std::cmp::Ordering::Greater);
/// # Ok::<(), rustycad_tkmath::predicates::NonFiniteCoordinate>(())
/// ```
pub fn compare_scalar_triple_product_3d(
    first: [f64; 3],
    second: [f64; 3],
    third: [f64; 3],
) -> Result<Ordering, NonFiniteCoordinate> {
    let first = exact_vector3(first, 0)?;
    let second = exact_vector3(second, 1)?;
    let third = exact_vector3(third, 2)?;
    Ok(exact_triple_product(&first, &second, &third).cmp(&BigInt::from(0)))
}
fn exact_vector3(vector: [f64; 3], index: usize) -> Result<[BigInt; 3], NonFiniteCoordinate> {
    Ok([
        exact_coordinate(vector[0], index, 0)?,
        exact_coordinate(vector[1], index, 1)?,
        exact_coordinate(vector[2], index, 2)?,
    ])
}
fn exact_triple_product(first: &[BigInt; 3], second: &[BigInt; 3], third: &[BigInt; 3]) -> BigInt {
    let cross: [BigInt; 3] = std::array::from_fn(|axis| {
        let next = (axis + 1) % 3;
        let last = (axis + 2) % 3;
        exact_determinant(
            &[second[next].clone(), second[last].clone()],
            &[third[next].clone(), third[last].clone()],
        )
    });
    &first[0] * &cross[0] + &first[1] * &cross[1] + &first[2] * &cross[2]
}

#[cfg(test)]
mod tests {
    use super::compare_scalar_triple_product_3d as sign;
    use std::cmp::Ordering::{Equal, Greater, Less};

    #[test]
    fn supplied_plane_vectors_distinguish_transverse_and_in_plane_extrusions() {
        assert_eq!(sign([1., 0., 0.], [0., 1., 0.], [0., 0., 1.]), Ok(Greater));
        assert_eq!(sign([0., 1., 0.], [1., 0., 0.], [0., 0., 1.]), Ok(Less));
        assert_eq!(sign([1., 0., 0.], [0., 1., 0.], [3., -4., -0.]), Ok(Equal));
        assert_eq!(
            sign([2., 3., 5.], [7., 11., 13.], [-17., -27., -29.]),
            Ok(Equal)
        );
    }
    #[test]
    fn independent_oblique_rational_determinant_is_negative() {
        // FLINT QQ expectation -78 is retained in scalar-triple-product-exact.json.
        assert_eq!(
            sign([2., 3., 5.], [7., 11., 13.], [17., 19., 23.]),
            Ok(Less)
        );
    }
    #[test]
    fn exact_determinant_survives_cancellation_of_rounded_float_products() {
        // FLINT QQ expectation 1 is retained in scalar-triple-product-exact.json.
        let n = 4_503_599_627_370_496.;
        assert_eq!(n * n - (n - 1.) * (n + 1.), 0.);
        assert_eq!(
            sign([n, n - 1., 0.], [n + 1., n, 0.], [0., 0., 1.]),
            Ok(Greater)
        );
        assert_eq!(
            sign([n, n - 1., 0.], [n + 1., n, 0.], [0., 0., -1.]),
            Ok(Less)
        );
    }
    #[test]
    fn exact_sign_survives_underflow_and_overflow_at_every_finite_power_of_two() {
        let bits = (0..52)
            .map(|shift| 1u64 << shift)
            .chain((1..=2046).map(|e| e << 52));
        for bits in bits {
            let s = f64::from_bits(bits);
            assert_eq!(sign([s, 0., 0.], [0., s, 0.], [0., 0., s]), Ok(Greater));
            assert_eq!(sign([s, 0., 0.], [0., s, 0.], [0., 0., -s]), Ok(Less));
        }
        assert_eq!(sign([-0.; 3], [1.; 3], [2.; 3]), Ok(Equal));
    }
    #[test]
    fn nonfinite_vector_components_preserve_actual_position_and_payload() {
        for position in 0..9 {
            for invalid in [
                f64::from_bits(0x7ff8_0000_0000_1234),
                f64::INFINITY,
                f64::NEG_INFINITY,
            ] {
                assert_rejected_vector_component(position, invalid);
            }
        }
    }
    fn assert_rejected_vector_component(position: usize, invalid: f64) {
        let mut vectors = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
        vectors[position / 3][position % 3] = invalid;
        let error = sign(vectors[0], vectors[1], vectors[2]).unwrap_err();
        assert_eq!((error.point, error.axis), (position / 3, position % 3));
        assert_eq!(error.value.to_bits(), invalid.to_bits());
        assert!(
            error
                .to_string()
                .contains("expected a finite binary64 coordinate")
        );
    }
    #[test]
    fn degenerate_vectors_do_not_hide_later_nonfinite_input() {
        let error = sign([0.; 3], [0., f64::INFINITY, f64::NAN], [f64::NAN; 3]).unwrap_err();
        assert_eq!((error.point, error.axis), (1, 1));
        assert_eq!(error.value, f64::INFINITY);
    }
}
