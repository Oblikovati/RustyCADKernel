//! Expand only flat fixed submissions, preserving geometry and native provoking input separately.
use crate::{PrimitiveDraw, PrimitiveTopology, fixed_lighting::FixedLightingInterpolation};
pub(super) fn fixed_vertex_sources(draw: &PrimitiveDraw, indices: &[u32]) -> Vec<(usize, usize)> {
    if !uses_fixed_flat_vertices(draw) {
        return (0..draw.vertices.len())
            .map(|index| (index, index))
            .collect();
    }
    expanded_provoking_sources(draw, indices)
}
fn expanded_provoking_sources(draw: &PrimitiveDraw, indices: &[u32]) -> Vec<(usize, usize)> {
    indices
        .as_chunks::<3>()
        .0
        .iter()
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
    uses_fixed_flat_vertices(draw).then(|| (0..count as u32).collect())
}
fn uses_fixed_flat_vertices(draw: &PrimitiveDraw) -> bool {
    draw.topology() == PrimitiveTopology::Triangles
        && draw
            .lighting
            .as_ref()
            .is_some_and(|state| state.fixed == Some(FixedLightingInterpolation::Flat))
}

#[cfg(test)]
pub(super) fn fixed_test_lighting(
    count: usize,
    shading: crate::RenderShading,
) -> crate::RenderLighting {
    let mut lighting = crate::RenderLighting::new(
        vec![[0., 0., 1.]; count],
        crate::RenderMaterial::default(),
        crate::identity_clip(),
        crate::identity_clip(),
        false,
        shading,
    )
    .unwrap();
    lighting.prepare_fixed_lighting().unwrap();
    lighting
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixed_quad_draw() -> PrimitiveDraw {
        let mut draw = PrimitiveDraw::new(
            vec![crate::RenderVertex::new([0.; 3], [1.; 4]); 4],
            vec![0, 1, 2, 0, 2, 3],
            crate::identity_clip(),
        )
        .unwrap();
        draw.lighting = Some(fixed_test_lighting(4, crate::RenderShading::PhongFacet));
        draw.native_flat_sources = vec![3, 3];
        draw
    }
    #[test]
    fn expanded_geometry_uses_original_positions_and_logical_quad_source() {
        let draw = fixed_quad_draw();
        assert_eq!(
            fixed_vertex_sources(&draw, &draw.indices),
            [(0, 3), (1, 3), (2, 3), (0, 3), (2, 3), (3, 3)]
        );
        assert_eq!(
            fixed_expanded_indices(&draw, 6),
            Some(vec![0, 1, 2, 3, 4, 5])
        );
    }
    #[test]
    fn ordinary_geometry_keeps_original_indices_and_vertices() {
        let mut draw = fixed_quad_draw();
        draw.lighting = None;
        assert_eq!(fixed_vertex_sources(&draw, &draw.indices).len(), 4);
        assert!(fixed_expanded_indices(&draw, 4).is_none());
    }
}
