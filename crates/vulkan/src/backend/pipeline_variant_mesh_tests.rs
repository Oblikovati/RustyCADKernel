//! Mesh shader interfaces and unsampled image policies keep cache identities distinct.
use super::*;
#[test]
fn native_mesh_shader_family_has_distinct_fragment_and_geometry_interfaces() {
    let mut fragments = std::collections::BTreeSet::new();
    let mut geometries = std::collections::BTreeSet::new();
    for index in 0..8 {
        let mut kind = empty_variant(false, index & 2 != 0, false, RenderCappingPass::None);
        kind.lighting = index & 1 != 0;
        kind.surface_texture = index & 4 != 0;
        kind.mesh_edges = true;
        let [vertex, fragment, geometry] =
            super::super::mesh_edge_pipeline::edge_shader_bytes(kind);
        assert_eq!(
            vertex,
            super::super::surface_texture_pipeline::surface_shaders(kind)[0]
        );
        fragments.insert(fragment);
        geometries.insert(geometry);
    }
    assert_eq!(fragments.len(), 8);
    assert_eq!(geometries.len(), 4);
}
#[test]
fn mesh_polygon_and_unsampled_binding_policies_have_distinct_pipeline_identity() {
    let mut draw = PrimitiveDraw::new(vec![], vec![], crate::identity_clip()).unwrap();
    let ordinary = special_variant_key(&draw);
    draw.mesh_edges = Some(crate::RenderMeshEdges::new([1.; 4], 1., false).unwrap());
    let mesh = special_variant_key(&draw);
    assert!(special_variant(&draw, false).mesh_edges);
    draw.mesh_edges = None;
    draw.polygon_edge_style = Some(crate::LineStyle::new(1., 0xffff, 1).unwrap());
    let polygon = special_variant_key(&draw);
    assert!(special_variant(&draw, false).line);
    draw.surface_texture_binding_only = true;
    let binding = special_variant_key(&draw);
    assert!(!special_variant(&draw, false).surface_texture);
    let mut identities = vec![ordinary, mesh, polygon, binding];
    identities.sort_unstable();
    identities.dedup();
    assert_eq!(identities.len(), 4);
}
