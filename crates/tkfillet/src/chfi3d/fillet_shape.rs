// Rust translation of ChFi3d_FilletShape.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The shape of the sections of a fillet.

/// The shape of the sections of a fillet surface (`ChFi3d_FilletShape`): rational
/// (exact circular arcs), quasi-angular (polynomial with a quasi-angular
/// parametrization) or polynomial; `FilletShape::Polynomial as i32 == 2`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FilletShape {
    /// `ChFi3d_Rational`.
    Rational = 0,
    /// `ChFi3d_QuasiAngular`.
    QuasiAngular = 1,
    /// `ChFi3d_Polynomial`.
    Polynomial = 2,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_follow_the_native_order() {
        assert_eq!(
            [
                FilletShape::Rational as i32,
                FilletShape::QuasiAngular as i32,
                FilletShape::Polynomial as i32
            ],
            [0, 1, 2]
        );
    }
}
