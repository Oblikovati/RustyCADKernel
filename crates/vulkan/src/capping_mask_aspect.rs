//! FillModeOnly rejects source aspects, so native stencil draws use Workspace::NoneCulling.
use crate::{PrimitiveDraw, RenderFaceCulling};
pub(crate) fn apply_native_stencil_aspect(mask: &mut PrimitiveDraw) {
    // OCCT3d097a0 OpenGl_Workspace::ShouldRender rejects OpenGl_Aspects under FillModeOnly.
    mask.lighting = None;
    mask.surface_texture = None;
    mask.mesh_edges = None;
    mask.polygon_edge_style = None;
    mask.edge_background_fallback = false;
    mask.surface_texture_binding_only = false;
    mask.surface_texture_unavailable = None;
    mask.surface_cutoff = None;
    mask.native_material_cutoff = None;
    mask.surface_forced_blend = false;
    mask.face_culling = RenderFaceCulling::None;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_filter_asset_and_stencil_policy_ignore_source_alpha_texture_and_culling() {
        let trace =
            include_str!("../tests/fixtures/texture-texgen-capping/native-cap-contract.txt");
        assert!(trace.lines().any(|line| line == "MASK_ASPECT_FILTER 0 1"));
        let mut mask = textured_mask_fixture();
        apply_native_stencil_aspect(&mut mask);
        assert_eq!(mask.face_culling, RenderFaceCulling::None);
        assert_eq!(mask.surface_cutoff, None);
        assert!(!mask.surface_forced_blend);
        assert!(mask.surface_texture_unavailable.is_none());
    }
    fn textured_mask_fixture() -> PrimitiveDraw {
        let mut mask = crate::PrimitiveDraw::new(
            vec![crate::RenderVertex::new([0.; 3], [1.; 4]); 3],
            vec![0, 1, 2],
            crate::identity_clip(),
        )
        .unwrap();
        mask.surface_cutoff = Some(0.5);
        mask.surface_forced_blend = true;
        mask.face_culling = RenderFaceCulling::Back;
        mask.surface_texture_unavailable =
            Some(crate::surface_texture::SurfaceTextureUnavailable {
                resource_id: b"native-mask-input".to_vec(),
                fixed_environment: true,
                modulate: true,
                vertex_override: false,
                back_vertex: true,
            });
        mask
    }
}
