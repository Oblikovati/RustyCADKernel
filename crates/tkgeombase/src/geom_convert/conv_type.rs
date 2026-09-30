// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_ConvType.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomConvert_ConvType`: how the canonical recognizers choose among the
//! candidate analytic curves and surfaces.

/// The conversion strategy of CurveToAnaCurve and SurfToAnaSurf, in the native
/// enumerator order (`GeomConvert_Target`, `GeomConvert_Simplest`,
/// `GeomConvert_MinGap`).
///
/// `let kind = ConvType::MinGap; assert_eq!(kind as i32, 2);`
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ConvType {
    /// Only the target type is tried.
    Target,
    /// The first candidate within the tolerance, simplest first.
    Simplest,
    /// The candidate of smallest gap within the tolerance.
    MinGap,
}

impl ConvType {
    /// The enumerator of native value `value` (0, 1 or 2), None otherwise;
    /// `ConvType::from_native(1) == Some(ConvType::Simplest)`.
    pub fn from_native(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Target),
            1 => Some(Self::Simplest),
            2 => Some(Self::MinGap),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enumerators_keep_the_native_values() {
        assert_eq!(ConvType::Target as i32, 0);
        assert_eq!(ConvType::Simplest as i32, 1);
        assert_eq!(ConvType::MinGap as i32, 2);
        assert_eq!(ConvType::from_native(2), Some(ConvType::MinGap));
        assert_eq!(ConvType::from_native(3), None);
    }
}
