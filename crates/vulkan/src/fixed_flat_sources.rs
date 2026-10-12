//! Preserve actual GL provoking conventions, including first-vertex GL_POLYGON shading.
use rustycad_tkservice::graphic3d::Graphic3dTypeOfPrimitiveArray as Kind;
pub(crate) fn native_flat_sources(kind: Kind, source: &[u32], triangles: &[u32]) -> Vec<u32> {
    match kind {
        Kind::Quadrangles => source.chunks_exact(4).flat_map(|q| [q[3]; 2]).collect(),
        Kind::Quadranglestrips => source
            .windows(4)
            .step_by(2)
            .flat_map(|q| [q[3]; 2])
            .collect(),
        Kind::Polygons => source
            .first()
            .map_or_else(Vec::new, |last| vec![*last; triangles.len() / 3]),
        _ => triangles.chunks_exact(3).map(|t| t[2]).collect(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn polygon_and_quad_sources_are_not_each_triangle_last_corner() {
        assert_eq!(
            native_flat_sources(Kind::Quadrangles, &[0, 1, 2, 3], &[0, 1, 2, 0, 2, 3]),
            [3, 3]
        );
        assert_eq!(
            native_flat_sources(Kind::Quadranglestrips, &[0, 1, 2, 3, 4, 5], &[0; 12]),
            [3, 3, 5, 5]
        );
        assert_eq!(
            native_flat_sources(Kind::Polygons, &[0, 1, 2, 3], &[0, 1, 2, 0, 2, 3]),
            [0, 0]
        );
        assert_eq!(
            native_flat_sources(Kind::Triangles, &[0, 1, 2], &[0, 1, 2]),
            [2]
        );
    }
}
