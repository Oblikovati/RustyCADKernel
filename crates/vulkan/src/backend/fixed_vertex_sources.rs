//! Expand only flat fixed submissions, preserving geometry and native provoking input separately.
use crate::{PrimitiveDraw, PrimitiveTopology, fixed_lighting::FixedLightingInterpolation};
pub(super) fn fixed_vertex_sources(draw: &PrimitiveDraw, indices: &[u32]) -> Vec<(usize, usize)> {
    let flat = draw
        .lighting
        .as_ref()
        .is_some_and(|state| state.fixed == Some(FixedLightingInterpolation::Flat));
    if !flat || draw.topology() != PrimitiveTopology::Triangles {
        return (0..draw.vertices.len())
            .map(|index| (index, index))
            .collect();
    }
    indices
        .chunks_exact(3)
        .enumerate()
        .flat_map(|(number, triangle)| {
            let source = draw
                .native_flat_sources
                .get(number)
                .copied()
                .unwrap_or(triangle[2]) as usize;
            triangle.iter().map(move |index| (*index as usize, source))
        })
        .collect()
}
pub(super) fn fixed_expanded_indices(draw: &PrimitiveDraw, count: usize) -> Option<Vec<u32>> {
    let flat = draw
        .lighting
        .as_ref()
        .is_some_and(|state| state.fixed == Some(FixedLightingInterpolation::Flat));
    (flat && draw.topology() == PrimitiveTopology::Triangles).then(|| (0..count as u32).collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expanded_geometry_uses_original_positions_and_logical_quad_source() {
        let mut draw = PrimitiveDraw::new(
            vec![crate::RenderVertex::new([0.; 3], [1.; 4]); 4],
            vec![0, 1, 2, 0, 2, 3],
            crate::identity_clip(),
        )
        .unwrap();
        let mut lighting = crate::RenderLighting::new(
            vec![[0., 0., 1.]; 4],
            crate::RenderMaterial::default(),
            crate::identity_clip(),
            crate::identity_clip(),
            false,
            crate::RenderShading::PhongFacet,
        )
        .unwrap();
        lighting.prepare_fixed_lighting().unwrap();
        draw.lighting = Some(lighting);
        draw.native_flat_sources = vec![3, 3];
        assert_eq!(
            fixed_vertex_sources(&draw, &draw.indices),
            [(0, 3), (1, 3), (2, 3), (0, 3), (2, 3), (3, 3)]
        );
        assert_eq!(
            fixed_expanded_indices(&draw, 6),
            Some(vec![0, 1, 2, 3, 4, 5])
        );
        draw.lighting = None;
        assert_eq!(fixed_vertex_sources(&draw, &draw.indices).len(), 4);
        assert!(fixed_expanded_indices(&draw, 4).is_none());
    }
}
