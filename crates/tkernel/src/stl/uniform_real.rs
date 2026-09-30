//! libstdc++ 13 `std::generate_canonical<double, 53>` over `std::mt19937` and
//! `std::uniform_real_distribution<double>` (bits/random.tcc and bits/random.h).
//!
//! The standard fixes the engine but not how a distribution turns its draws into a
//! double, so this follows the pinned library: two 32-bit draws make one canonical value.
use super::MersenneTwister;

/// `2^32`, the range `max() - min() + 1` of `std::mt19937`, as the double `__tmp` grows.
const ENGINE_RANGE: f64 = 4_294_967_296.0;

/// `std::generate_canonical<double, 53>(engine)`: a value in `[0, 1)` from
/// `ceil(53 / 32) = 2` draws, `(d1 + d2 * 2^32) / 2^64` summed in double as libstdc++
/// does; a sum that rounds up to 1 becomes the largest double below 1.
///
/// ```
/// use rustycad_tkernel::stl::{MersenneTwister, generate_canonical};
/// let value = generate_canonical(&mut MersenneTwister::new(0));
/// assert!((0.0..1.0).contains(&value));
/// ```
pub fn generate_canonical(engine: &mut MersenneTwister) -> f64 {
    let mut sum: f64 = 0.0;
    let mut scale: f64 = 1.0;
    for _ in 0..2 {
        sum += f64::from(engine.next_u32()) * scale;
        scale *= ENGINE_RANGE;
    }
    let canonical: f64 = sum / scale;
    if canonical >= 1.0 {
        // std::nextafter(1.0, 0.0) (_GLIBCXX_USE_C99_MATH_TR1 is set on Linux).
        return 1.0 - f64::EPSILON / 2.0;
    }
    canonical
}

/// `std::uniform_real_distribution<double>(a, b)`: each draw is
/// `generate_canonical(engine) * (b - a) + a`, in that operation order.
///
/// ```
/// use rustycad_tkernel::stl::{MersenneTwister, UniformRealDistribution};
/// let mut engine = MersenneTwister::new(0);
/// let value = UniformRealDistribution::new(2.0, 3.0).sample(&mut engine);
/// assert!((2.0..3.0).contains(&value));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UniformRealDistribution {
    a: f64,
    b: f64,
}

impl UniformRealDistribution {
    /// The distribution over `[a, b)`; the native constructor only asserts `a <= b` in
    /// debug builds, so any bounds are kept; `UniformRealDistribution::new(0.0, 1.0)`.
    pub const fn new(a: f64, b: f64) -> Self {
        Self { a, b }
    }

    /// `a()`: the lower bound; `distribution.a()`.
    pub const fn a(&self) -> f64 {
        self.a
    }

    /// `b()`: the upper bound; `distribution.b()`.
    pub const fn b(&self) -> f64 {
        self.b
    }

    /// `operator()(engine)`: the next value; `distribution.sample(&mut engine)`.
    pub fn sample(&self, engine: &mut MersenneTwister) -> f64 {
        generate_canonical(engine) * (self.b - self.a) + self.a
    }
}

#[cfg(test)]
mod tests {
    use super::{MersenneTwister, UniformRealDistribution, generate_canonical};

    #[test]
    fn canonical_combines_two_draws_low_word_first() {
        let mut reference = MersenneTwister::new(0);
        let (low, high) = (reference.next_u32(), reference.next_u32());
        let expected =
            (f64::from(low) + f64::from(high) * 4_294_967_296.0) / 18_446_744_073_709_551_616.0;
        assert_eq!(generate_canonical(&mut MersenneTwister::new(0)), expected);
    }

    #[test]
    fn distribution_scales_then_shifts() {
        let canonical = generate_canonical(&mut MersenneTwister::new(7));
        let value = UniformRealDistribution::new(-2.0, 5.0).sample(&mut MersenneTwister::new(7));
        assert_eq!(value, canonical * 7.0 + -2.0);
    }
}
