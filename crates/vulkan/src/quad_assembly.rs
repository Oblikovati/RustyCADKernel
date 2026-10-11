//! Keep legacy quad interpolation device-dependent without guessing from triangle geometry.
use crate::PrimitiveDraw;
use std::borrow::Cow;
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum QuadDiagonal {
    #[default]
    FirstThird,
    SecondFourth,
}
impl QuadDiagonal {
    pub(crate) fn for_vendor(vendor: u32) -> Self {
        // Paired 3d097a0 captures: llvmpipe uses 0--2, the host's radeonsi uses 1--3.
        // Other OpenGL drivers retain the canonical diagonal until their captures establish a policy.
        if vendor == 0x1002 {
            Self::SecondFourth
        } else {
            Self::FirstThird
        }
    }
}
pub(crate) fn device_quad_indices(draw: &PrimitiveDraw, diagonal: QuadDiagonal) -> Cow<'_, [u32]> {
    if draw.quad_corners.is_empty() || diagonal == QuadDiagonal::FirstThird {
        return Cow::Borrowed(&draw.indices);
    }
    Cow::Owned(
        draw.quad_corners
            .iter()
            .flat_map(|q| [q[0], q[1], q[3], q[1], q[2], q[3]])
            .collect(),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verified_native_profiles_choose_their_quad_diagonal() {
        assert_eq!(QuadDiagonal::for_vendor(0x1002), QuadDiagonal::SecondFourth);
        assert_eq!(QuadDiagonal::for_vendor(0x10005), QuadDiagonal::FirstThird);
        assert_eq!(QuadDiagonal::for_vendor(0), QuadDiagonal::FirstThird);
    }
    #[test]
    fn quad_corners_preserve_indices_and_ordinary_triangles_are_never_reinterpreted() {
        let mut draw = PrimitiveDraw::new(vec![], vec![], crate::identity_clip()).unwrap();
        draw.indices = vec![4, 3, 2, 4, 2, 1];
        assert_eq!(
            device_quad_indices(&draw, QuadDiagonal::SecondFourth).as_ref(),
            draw.indices
        );
        draw.quad_corners = vec![[4, 3, 2, 1]];
        assert_eq!(
            device_quad_indices(&draw, QuadDiagonal::SecondFourth).as_ref(),
            [4, 3, 1, 3, 2, 1]
        );
        assert_eq!(
            device_quad_indices(&draw, QuadDiagonal::FirstThird).as_ref(),
            draw.indices
        );
    }
}
