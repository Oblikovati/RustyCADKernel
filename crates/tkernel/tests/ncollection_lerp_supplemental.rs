//! NCollection_Lerp: `(1 - t) * start + t * end` for any scalar-multipliable value.
use rustycad_tkernel::ncollection::Lerp;
use std::ops::{Add, Mul};

/// A two-component value that records nothing but supports the operators the template needs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Pair(f64, f64);
impl Add for Pair {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Pair(self.0 + other.0, self.1 + other.1)
    }
}
impl Mul<Pair> for f64 {
    type Output = Pair;
    fn mul(self, pair: Pair) -> Pair {
        Pair(self * pair.0, self * pair.1)
    }
}

#[test]
fn scalars_use_the_native_expression() {
    let lerp = Lerp::new(0.1, 0.7);
    assert_eq!(lerp.interpolate(0.3), (1.0 - 0.3) * 0.1 + 0.3 * 0.7);
    assert_eq!(lerp.interpolate(0.0), 0.1);
    assert_eq!(lerp.interpolate(1.0), 0.7);
    assert_eq!(Lerp::interpolated(2.0, 4.0, 0.5), 3.0);
    assert_eq!(Lerp::new(2.0, 4.0).interpolate(2.0), 6.0);
}

#[test]
fn default_and_init_replace_both_ends() {
    let mut lerp = Lerp::<f64>::default();
    assert_eq!(lerp.interpolate(0.5), 0.0);
    lerp.init(-1.0, 1.0);
    assert_eq!(lerp, Lerp::new(-1.0, 1.0));
    assert_eq!(lerp.interpolate(0.75), 0.5);
}

#[test]
fn composite_values_interpolate_componentwise() {
    let lerp = Lerp::new(Pair(0.0, 10.0), Pair(4.0, 20.0));
    assert_eq!(lerp.interpolate(0.25), Pair(1.0, 12.5));
    assert_eq!(
        Lerp::interpolated(Pair(1.0, 1.0), Pair(3.0, 5.0), 0.5),
        Pair(2.0, 3.0)
    );
}
